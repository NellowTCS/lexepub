use crate::error::{LexEpubError, Result};
use async_zip::base::read::seek::ZipFileReader;
use bytes::Bytes;
use std::path::Path;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Low-level EPUB extractor that handles file operations
#[derive(Clone)]
pub struct EpubExtractor {
    data_source: EpubDataSource,
}

#[derive(Clone)]
enum EpubDataSource {
    FilePath(std::path::PathBuf),
    Bytes(Bytes),
}

impl EpubExtractor {
    /// Open EPUB from file path
    pub async fn open(path: std::path::PathBuf) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::FilePath(path),
        })
    }

    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::Bytes(data),
        })
    }

    /// Read a specific file from EPUB
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        match &self.data_source {
            EpubDataSource::FilePath(file_path) => self.read_file_from_path(file_path, path).await,
            EpubDataSource::Bytes(bytes) => self.read_file_from_bytes(bytes, path).await,
        }
    }

    /// Read file from EPUB file path
    async fn read_file_from_path(&self, file_path: &Path, path: &str) -> Result<Vec<u8>> {
        let file = tokio::fs::File::open(file_path)
            .await
            .map_err(LexEpubError::Io)?;
        let reader = BufReader::new(file).compat();
        let mut archive = ZipFileReader::new(reader)
            .await
            .map_err(LexEpubError::Zip)?;

        self.extract_file_from_archive(&mut archive, path).await
    }

    /// Read file from EPUB bytes
    async fn read_file_from_bytes(&self, data: &Bytes, path: &str) -> Result<Vec<u8>> {
        let cursor = std::io::Cursor::new(data.as_ref());
        let reader = BufReader::new(cursor).compat();
        let mut archive = ZipFileReader::new(reader)
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
