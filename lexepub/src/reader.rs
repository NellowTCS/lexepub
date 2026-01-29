use crate::error::{LexEpubError, Result};
use crate::metadata::{EpubMetadata, MetadataParser};
use crate::parser::{Chapter, ChapterParser, ParsedChapter};
use async_zip::base::read::seek::ZipFileReader;
use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use std::path::Path;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Low-level EPUB extractor that handles file operations
#[derive(Clone)]
pub struct EpubExtractor {
    data_source: EpubDataSource,
    chapter_parser: Option<ChapterParser>,
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
            chapter_parser: Some(ChapterParser::new().text_only()),
        })
    }

    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        Ok(Self {
            data_source: EpubDataSource::Bytes(data),
            chapter_parser: Some(ChapterParser::new().text_only()),
        })
    }

    /// Set chapter parser
    pub fn set_chapter_parser(&mut self, parser: ChapterParser) {
        self.chapter_parser = Some(parser);
    }

    /// Read and parse EPUB metadata
    pub async fn read_metadata(&self) -> Result<EpubMetadata> {
        let container_data = self.read_file("META-INF/container.xml").await?;
        let mut metadata_parser = MetadataParser::new();
        let container_info = metadata_parser.parse_container(&container_data)?;

        let opf_data = self.read_file(&container_info.rootfile_path).await?;
        let metadata = metadata_parser.parse_opf(&opf_data)?;

        Ok(metadata)
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

    /// Read all chapters from metadata
    pub async fn read_chapters(&self, metadata: &EpubMetadata) -> Result<Vec<ParsedChapter>> {
        let mut chapters = Vec::with_capacity(metadata.spine.len());

        for spine_item in &metadata.spine {
            let href = metadata.manifest.get(spine_item).ok_or_else(|| {
                LexEpubError::MissingFile(format!(
                    "Spine item '{}' not found in manifest",
                    spine_item
                ))
            })?;

            let chapter_data = self.read_file(href).await?;

            let chapter = Chapter {
                href: href.clone(),
                id: spine_item.clone(),
                media_type: "application/xhtml+xml".to_string(), // Default for EPUB
                content: chapter_data,
            };

            let parsed_chapter = self
                .chapter_parser
                .as_ref()
                .unwrap()
                .parse_chapter(chapter)?;
            chapters.push(parsed_chapter);
        }

        Ok(chapters)
    }

    /// Stream chapters one by one (memory efficient)
    pub fn chapters_stream<'a>(
        &'a self,
        metadata: &'a EpubMetadata,
    ) -> impl Stream<Item = Result<ParsedChapter>> + 'a {
        futures::stream::iter(metadata.spine.iter()).then(move |spine_item| async move {
            let href = metadata.manifest.get(spine_item).ok_or_else(|| {
                LexEpubError::MissingFile(format!(
                    "Spine item '{}' not found in manifest",
                    spine_item
                ))
            })?;

            let chapter_data = self.read_file(href).await?;

            let chapter = Chapter {
                href: href.clone(),
                id: spine_item.clone(),
                media_type: "application/xhtml+xml".to_string(),
                content: chapter_data,
            };

            self.chapter_parser.as_ref().unwrap().parse_chapter(chapter)
        })
    }

    /// Get cover image data if available
    pub async fn cover_image(&self, metadata: &EpubMetadata) -> Result<Option<Vec<u8>>> {
        if let Some(cover_path) = &metadata.cover_image {
            let cover_data = self.read_file(cover_path).await?;
            Ok(Some(cover_data))
        } else {
            Ok(None)
        }
    }
}

pub struct LexEpub {
    extractor: EpubExtractor,
    metadata: Option<EpubMetadata>,
}

impl LexEpub {
    /// Open an EPUB file for streaming extraction
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let extractor = EpubExtractor::open(path.as_ref().to_path_buf()).await?;

        Ok(Self {
            extractor,
            metadata: None,
        })
    }

    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        let extractor = EpubExtractor::from_bytes(data).await?;

        Ok(Self {
            extractor,
            metadata: None,
        })
    }

    /// Extract and parse EPUB metadata
    pub async fn metadata(&mut self) -> Result<&EpubMetadata> {
        if self.metadata.is_none() {
            self.metadata = Some(self.extractor.read_metadata().await?);
        }
        Ok(self.metadata.as_ref().expect("metadata was just set"))
    }

    /// Create chapters stream
    pub async fn chapters_stream(
        &mut self,
    ) -> Result<impl Stream<Item = Result<ParsedChapter>> + 'static> {
        let metadata = self.metadata().await?.clone();
        let extractor = self.extractor.clone();

        Ok(
            futures::stream::iter(metadata.spine.into_iter()).then(move |spine_item| {
                let manifest = metadata.manifest.clone();
                let extractor = extractor.clone();
                async move {
                    let href = manifest.get(&spine_item).ok_or_else(|| {
                        LexEpubError::MissingFile(format!(
                            "Spine item '{}' not found in manifest",
                            spine_item
                        ))
                    })?;

                    let chapter_data = extractor.read_file(href).await?;

                    let chapter = Chapter {
                        href: href.clone(),
                        id: spine_item.clone(),
                        media_type: "application/xhtml+xml".to_string(),
                        content: chapter_data,
                    };

                    extractor
                        .chapter_parser
                        .as_ref()
                        .ok_or(LexEpubError::ChapterError(
                            "No chapter parser available".to_string(),
                        ))?
                        .parse_chapter(chapter)
                }
            }),
        )
    }

    /// Get all chapters at once
    pub async fn chapters(&mut self) -> Result<Vec<ParsedChapter>> {
        let metadata = self.metadata().await?;
        let metadata = metadata.clone();
        self.extractor.read_chapters(&metadata).await
    }

    /// Get cover image data if available
    pub async fn cover_image(&mut self) -> Result<Option<Vec<u8>>> {
        let metadata = self.metadata().await?;
        let metadata = metadata.clone();
        self.extractor.cover_image(&metadata).await
    }

    /// Set chapter parser
    pub fn set_chapter_parser(&mut self, parser: ChapterParser) {
        self.extractor.set_chapter_parser(parser);
    }
}
