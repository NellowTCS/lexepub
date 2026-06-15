use crate::error::{LexEpubError, Result};
use async_zip::base::read::seek::ZipFileReader;
use bytes::Bytes;
use futures::io::{AllowStdIo, BufReader as FuturesBufReader, Cursor as FuturesCursor};
use futures::lock::Mutex as AsyncMutex;
use std::path::Path;
use std::sync::Arc;

// Trait-object helper: combine AsyncBufRead + AsyncSeek + Unpin into one
// object-safe trait so we can store boxed streaming readers.
trait AsyncReadSeek: futures::AsyncBufRead + futures::AsyncSeek + Unpin {}
impl<T: futures::AsyncBufRead + futures::AsyncSeek + Unpin> AsyncReadSeek for T {}

/// Concrete ZIP archive type for file-backed EPUBs.
type FileArchive = ZipFileReader<FuturesBufReader<AllowStdIo<std::fs::File>>>;

/// Low-level EPUB extractor that handles file operations. The extractor can
/// operate from a file path, an in-memory byte buffer, or a streaming reader
/// (async or sync wrapped with `AllowStdIo`).
pub struct EpubExtractor {
    data_source: EpubDataSource,
    /// Lazily-opened ZipFileReader for the FilePath variant. Opened on the
    /// first `read_file` call and reused for all subsequent reads, avoiding
    /// redundant ZIP central directory reads that fragment the heap.
    archive: Arc<AsyncMutex<Option<FileArchive>>>,
}

enum EpubDataSource {
    FilePath(std::path::PathBuf),
    Bytes(Bytes),
    /// A boxed async reader protected by an Arc<AsyncMutex<...>> so the
    /// extractor can be cloned and shared across tasks.
    Reader(Arc<AsyncMutex<Box<dyn AsyncReadSeek + Send + 'static>>>),
}

impl Clone for EpubExtractor {
    fn clone(&self) -> Self {
        Self {
            data_source: match &self.data_source {
                EpubDataSource::FilePath(p) => EpubDataSource::FilePath(p.clone()),
                EpubDataSource::Bytes(b) => EpubDataSource::Bytes(b.clone()),
                EpubDataSource::Reader(r) => EpubDataSource::Reader(r.clone()),
            },
            archive: self.archive.clone(),
        }
    }
}

impl EpubExtractor {
    /// Open EPUB from file path
    pub async fn open(path: std::path::PathBuf) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::FilePath(path),
            archive: Arc::new(AsyncMutex::new(None)),
        })
    }

    /// Create extractor from in-memory bytes
    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::Bytes(data),
            archive: Arc::new(AsyncMutex::new(None)),
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
            data_source: EpubDataSource::Reader(Arc::new(AsyncMutex::new(Box::new(reader)))),
            archive: Arc::new(AsyncMutex::new(None)),
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
            data_source: EpubDataSource::Reader(Arc::new(AsyncMutex::new(Box::new(buf)))),
            archive: Arc::new(AsyncMutex::new(None)),
        })
    }

    /// Read a specific file from EPUB
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        match &self.data_source {
            EpubDataSource::FilePath(_file_path) => self.read_file_with_cache(path).await,
            EpubDataSource::Bytes(bytes) => self.read_file_from_bytes(bytes, path).await,
            EpubDataSource::Reader(_) => self.read_file_from_reader(path).await,
        }
    }

    /// Stream a specific file from EPUB directly to an AsyncWrite destination
    pub async fn read_file_to_writer<W: futures::AsyncWrite + Unpin + Send>(
        &self,
        path: &str,
        writer: &mut W,
    ) -> Result<u64> {
        match &self.data_source {
            EpubDataSource::FilePath(file_path) => {
                self.read_file_from_path_to_writer(file_path, path, writer)
                    .await
            }
            EpubDataSource::Bytes(bytes) => {
                self.read_file_from_bytes_to_writer(bytes, path, writer)
                    .await
            }
            EpubDataSource::Reader(_) => self.read_file_from_reader_to_writer(path, writer).await,
        }
    }

    /// Lock the archive, initializing it if this is the first call.
    /// The returned guard keeps the archive alive for the caller's scope.
    async fn lock_archive(&self) -> Result<futures::lock::MutexGuard<'_, Option<FileArchive>>> {
        let file_path = match &self.data_source {
            EpubDataSource::FilePath(p) => p.clone(),
            _ => {
                return Err(LexEpubError::MissingFile(
                    "lock_archive called on non-FilePath extractor".into(),
                ))
            }
        };
        let mut guard = self.archive.lock().await;
        if guard.is_none() {
            let file = std::fs::File::open(&file_path).map_err(LexEpubError::Io)?;
            let allow = AllowStdIo::new(file);
            let reader = FuturesBufReader::new(allow);
            let archive = ZipFileReader::new(reader)
                .await
                .map_err(LexEpubError::Zip)?;
            *guard = Some(archive);
        }
        Ok(guard)
    }

    /// Read from FilePath using a lazily-cached ZipFileReader.
    ///
    /// First call opens the file and creates the archive; subsequent calls
    /// reuse it, eliminating repeated ZIP central directory reads.
    async fn read_file_with_cache(&self, path: &str) -> Result<Vec<u8>> {
        let mut guard = self.lock_archive().await?;
        let archive = guard.as_mut().unwrap();
        self.extract_file_from_archive(archive, path).await
    }

    /// Read a ZIP entry in small chunks, feeding each chunk to a callback.
    ///
    /// Never allocates more than `chunk_size` bytes contiguously, the key
    /// property for heap-fragmented embedded targets.
    pub async fn read_entry_chunked(
        &self,
        path: &str,
        chunk_size: usize,
        callback: &mut dyn FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        let mut guard = self.lock_archive().await?;
        let archive = guard.as_mut().unwrap();

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

        let mut buf = vec![0u8; chunk_size];
        use futures::AsyncReadExt;
        loop {
            let n = entry_reader
                .read(&mut buf)
                .await
                .map_err(LexEpubError::Io)?;
            if n == 0 {
                break;
            }
            callback(&buf[..n])?;
        }
        Ok(())
    }

    /// Read file from EPUB bytes (no caching, Bytes is cheap to clone and
    /// typically used in test/one-shot scenarios).
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

        // Read the uncompressed size from the ZIP entry before borrowing the
        // archive mutably via reader_without_entry, so we can pre-allocate the
        // output buffer to its exact size.
        let entry_size = entries.get(entry_index).and_then(|e| {
            let s = e.uncompressed_size();
            if s > 0 && s < 10 * 1024 * 1024 {
                Some(s as usize)
            } else {
                None
            }
        });

        let mut entry_reader = archive
            .reader_without_entry(entry_index)
            .await
            .map_err(LexEpubError::Zip)?;

        let mut file_data = match entry_size {
            Some(size) => Vec::with_capacity(size),
            None => Vec::new(),
        };

        use futures::AsyncReadExt;
        entry_reader
            .read_to_end(&mut file_data)
            .await
            .map_err(LexEpubError::Io)?;

        Ok(file_data)
    }
}

impl EpubExtractor {
    async fn read_file_from_path_to_writer<W: futures::AsyncWrite + Unpin + Send>(
        &self,
        file_path: &Path,
        path: &str,
        writer: &mut W,
    ) -> Result<u64> {
        let file = std::fs::File::open(file_path).map_err(LexEpubError::Io)?;
        let allow = AllowStdIo::new(file);
        let reader = FuturesBufReader::new(allow);
        let mut archive = ZipFileReader::new(reader)
            .await
            .map_err(LexEpubError::Zip)?;
        self.extract_file_from_archive_to_writer(&mut archive, path, writer)
            .await
    }

    async fn read_file_from_bytes_to_writer<W: futures::AsyncWrite + Unpin + Send>(
        &self,
        data: &Bytes,
        path: &str,
        writer: &mut W,
    ) -> Result<u64> {
        let cursor = FuturesCursor::new(data.as_ref());
        let reader = FuturesBufReader::new(cursor);
        let mut archive = ZipFileReader::new(reader)
            .await
            .map_err(LexEpubError::Zip)?;
        self.extract_file_from_archive_to_writer(&mut archive, path, writer)
            .await
    }

    async fn read_file_from_reader_to_writer<W: futures::AsyncWrite + Unpin + Send>(
        &self,
        path: &str,
        writer: &mut W,
    ) -> Result<u64> {
        let mut guard = match &self.data_source {
            EpubDataSource::Reader(m) => m.lock().await,
            _ => unreachable!(),
        };
        let reader_ref: &mut (dyn AsyncReadSeek + '_) = &mut *guard;
        let mut archive = ZipFileReader::new(reader_ref)
            .await
            .map_err(LexEpubError::Zip)?;
        self.extract_file_from_archive_to_writer(&mut archive, path, writer)
            .await
    }

    async fn extract_file_from_archive_to_writer<R, W>(
        &self,
        archive: &mut ZipFileReader<R>,
        path: &str,
        writer: &mut W,
    ) -> Result<u64>
    where
        R: futures::AsyncBufRead + futures::AsyncSeek + Unpin,
        W: futures::AsyncWrite + Unpin + Send,
    {
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

        futures::io::copy(&mut entry_reader, writer)
            .await
            .map_err(LexEpubError::Io)
    }
}
