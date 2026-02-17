use crate::error::{LexEpubError, Result};
use async_zip::base::read::seek::ZipFileReader;
use bytes::Bytes;
use futures::io::{AllowStdIo, BufReader as FuturesBufReader, Cursor as FuturesCursor};
use futures::lock::Mutex as AsyncMutex;
use std::path::Path;

// Trait-object helper: combine AsyncBufRead + AsyncSeek + Unpin into one
// object-safe trait so we can store boxed streaming readers.
trait AsyncReadSeek: futures::AsyncBufRead + futures::AsyncSeek + Unpin {}
impl<T: futures::AsyncBufRead + futures::AsyncSeek + Unpin> AsyncReadSeek for T {}

/// Low-level EPUB extractor that handles file operations. The extractor can
/// operate from a file path, an in-memory byte buffer, or a streaming reader
/// (async or sync wrapped with `AllowStdIo`).
pub struct EpubExtractor {
    data_source: EpubDataSource,
}

enum EpubDataSource {
    FilePath(std::path::PathBuf),
    Bytes(Bytes),
    /// A boxed async reader protected by an async Mutex so multiple `read_file`
    /// calls can borrow it sequentially.
    Reader(AsyncMutex<Box<dyn AsyncReadSeek + Send + 'static>>),
}

impl EpubExtractor {
    /// Open EPUB from file path
    pub async fn open(path: std::path::PathBuf) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::FilePath(path),
        })
    }

    /// Create extractor from in-memory bytes
    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::Bytes(data),
        })
    }

    /// Create extractor from an async reader (streaming, does not copy whole
    /// archive into memory). The reader will be stored and used for subsequent
    /// extraction calls.
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: futures::AsyncBufRead + futures::AsyncSeek + Unpin + Send + 'static,
    {
        Ok(Self {
            data_source: EpubDataSource::Reader(AsyncMutex::new(Box::new(reader))),
        })
    }

    /// Create extractor from a blocking (sync) reader by wrapping it with
    /// `futures::io::AllowStdIo`. Useful for platforms that expose blocking
    /// file handles.
    pub fn from_sync_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read + std::io::Seek + Send + 'static,
    {
        let allow = AllowStdIo::new(reader);
        let buf = FuturesBufReader::new(allow);
        Ok(Self {
            data_source: EpubDataSource::Reader(AsyncMutex::new(Box::new(buf))),
        })
    }

    /// Read a specific file from EPUB
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        match &self.data_source {
            EpubDataSource::FilePath(file_path) => self.read_file_from_path(file_path, path).await,
            EpubDataSource::Bytes(bytes) => self.read_file_from_bytes(bytes, path).await,
            EpubDataSource::Reader(_) => self.read_file_from_reader(path).await,
        }
    }

    /// Read file from EPUB file path
    async fn read_file_from_path(&self, file_path: &Path, path: &str) -> Result<Vec<u8>> {
        // Stream the EPUB file from disk without reading it entirely into memory.
        // Wrap the blocking std::fs::File with futures::io::AllowStdIo so it implements
        // the futures AsyncRead + AsyncSeek traits required by async_zip.
        let file = std::fs::File::open(file_path).map_err(LexEpubError::Io)?;
        let allow = AllowStdIo::new(file);
        let reader = FuturesBufReader::new(allow);
        let mut archive = ZipFileReader::new(reader)
            .await
            .map_err(LexEpubError::Zip)?;

        self.extract_file_from_archive(&mut archive, path).await
    }

    /// Read file from EPUB bytes
    async fn read_file_from_bytes(&self, data: &Bytes, path: &str) -> Result<Vec<u8>> {
        let cursor = FuturesCursor::new(data.as_ref());
        let reader = FuturesBufReader::new(cursor);
        let mut archive = ZipFileReader::new(reader)
            .await
            .map_err(LexEpubError::Zip)?;

        self.extract_file_from_archive(&mut archive, path).await
    }

    /// Read file from a stored async reader
    async fn read_file_from_reader(&self, path: &str) -> Result<Vec<u8>> {
        // Acquire the async mutex and create a ZipFileReader over a mutable
        // reference to the boxed reader. Keep the guard alive for the
        // duration of the archive usage so the borrowed reference stays valid.
        let mut guard = match &self.data_source {
            EpubDataSource::Reader(m) => m.lock().await,
            _ => unreachable!(),
        };

        // Make the reference explicit to help type inference for ZipFileReader.
        let reader_ref: &mut (dyn AsyncReadSeek + '_) = &mut *guard;
        let mut archive = ZipFileReader::new(reader_ref)
            .await
            .map_err(LexEpubError::Zip)?;

        self.extract_file_from_archive(&mut archive, path).await
    }

    /// Extract specific file from ZIP archive
    async fn extract_file_from_archive<R>(
        &self,
        archive: &mut ZipFileReader<R>,
        path: &str,
    ) -> Result<Vec<u8>>
    where
        R: futures::AsyncBufRead + futures::AsyncSeek + Unpin,
    {
        // Find entry by filename
        let entries = archive.file().entries();
        let entry_index = entries
            .iter()
            .enumerate()
            .find_map(|(i, entry)| {
                entry
                    .filename()
                    .as_str()
                    .ok()
                    .and_then(|filename| (filename == path).then_some(i))
            })
            .ok_or_else(|| {
                LexEpubError::MissingFile(format!("File '{}' not found in EPUB", path))
            })?;

        let mut entry_reader = archive
            .reader_without_entry(entry_index)
            .await
            .map_err(LexEpubError::Zip)?;

        let mut file_data = Vec::new();

        use futures::AsyncReadExt;
        entry_reader
            .read_to_end(&mut file_data)
            .await
            .map_err(LexEpubError::Io)?;

        Ok(file_data)
    }
}
