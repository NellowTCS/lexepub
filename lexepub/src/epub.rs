use crate::core::chapter::{Chapter, ChapterStream, ParsedChapter};
use crate::core::container::ContainerParser;
use crate::core::extractor::EpubExtractor;
use crate::core::html_parser::extract_text_content;
use crate::core::opf_parser::OpfParser;
use crate::error::{LexEpubError, Result};
use bytes::Bytes;
use std::path::Path;

/// Main EPUB processing struct
pub struct LexEpub {
    extractor: EpubExtractor,
    metadata: Option<EpubMetadata>,
    chapters: Option<Vec<ParsedChapter>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub version: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub languages: Vec<String>,
    pub subjects: Vec<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<String>,
    pub identifiers: Vec<String>,
    pub rights: Option<String>,
    pub contributors: Vec<String>,
    pub spine: Vec<String>,
    pub has_cover: bool,
    pub cover_image_format: Option<String>,
    pub chapter_count: usize,
}

impl EpubMetadata {
    /// Validates the metadata per EPUB standards (requires title, language, and identifier)
    pub fn validate(&self) -> std::result::Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.as_ref().is_none_or(|t| t.trim().is_empty()) {
            errors.push("Missing required field: title".to_string());
        }
        if self.languages.is_empty() {
            errors.push("Missing required field: language".to_string());
        }
        if self.identifiers.is_empty() {
            errors.push("Missing required field: identifier".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl LexEpub {
    /// Open an EPUB from a file path
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let extractor = EpubExtractor::open(path.as_ref().to_path_buf()).await?;
        Ok(Self {
            extractor,
            metadata: None,
            chapters: None,
        })
    }

    /// Synchronous wrapper for `open` (used by FFI and sync callers)
    pub fn open_sync<P: AsRef<Path>>(path: P) -> Result<Self> {
        futures::executor::block_on(LexEpub::open(path))
    }

    /// Create an EPUB from bytes
    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        let extractor = EpubExtractor::from_bytes(data).await?;
        Ok(Self {
            extractor,
            metadata: None,
            chapters: None,
        })
    }

    /// Create an EPUB from an async reader (streaming, does not copy the whole
    /// archive into memory). Useful for SD/LittleFS/flash-backed readers.
    pub async fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: futures::AsyncBufRead + futures::AsyncSeek + Unpin + Send + 'static,
    {
        let extractor = EpubExtractor::from_reader(reader)?;
        Ok(Self {
            extractor,
            metadata: None,
            chapters: None,
        })
    }

    /// Create an EPUB from a blocking reader by wrapping it with
    /// `futures::io::AllowStdIo` (convenience for platforms with sync FS APIs).
    pub fn from_sync_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read + std::io::Seek + Send + 'static,
    {
        let extractor = EpubExtractor::from_sync_reader(reader)?;
        Ok(Self {
            extractor,
            metadata: None,
            chapters: None,
        })
    }

    /// Extract only text content from all chapters
    pub async fn extract_text_only(&mut self) -> Result<Vec<String>> {
        let chapters = self.extract_chapters().await?;
        Ok(chapters.into_iter().map(|c| c.content).collect())
    }

    /// Extract chapters with AST for advanced processing
    pub async fn extract_ast(&mut self) -> Result<Vec<ParsedChapter>> {
        self.extract_chapters().await
    }

    /// Extract chapters as a stream for memory-efficient processing
    pub async fn extract_chapters_stream(&mut self) -> Result<ChapterStream> {
        // Build a streaming ChapterStream that reads each chapter lazily from
        // the archive via the extractor.

        // Get OPF location
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        // Parse OPF for spine and manifest
        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let spine = opf_parser.parse_spine(&opf_data)?;
        let metadata = opf_parser.parse_metadata(&opf_data)?;

        // Resolve full paths for spine entries and return a streaming iterator
        let mut entries = Vec::new();
        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));

        for item_id in spine {
            if let Some(href) = metadata.manifest.get(&item_id) {
                let full_path = opf_base.join(&href.0);
                let full_path_str = full_path.to_string_lossy().to_string();
                entries.push(full_path_str);
            }
        }

        Ok(ChapterStream::from_extractor(
            self.extractor.clone(),
            entries,
        ))
    }

    /// Get metadata
    pub async fn get_metadata(&mut self) -> Result<EpubMetadata> {
        if let Some(ref metadata) = self.metadata {
            return Ok(metadata.clone());
        }

        // Get OPF location
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        // Parse OPF metadata
        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let opf_metadata = opf_parser.parse_metadata(&opf_data)?;

        let epub_metadata = EpubMetadata {
            title: opf_metadata.title,
            version: opf_metadata.version,
            authors: opf_metadata.creators,
            description: opf_metadata.description,
            languages: opf_metadata.languages,
            subjects: opf_metadata.subjects,
            publisher: opf_metadata.publisher,
            publication_date: opf_metadata.date,
            identifiers: opf_metadata.identifiers,
            rights: opf_metadata.rights,
            contributors: opf_metadata.contributors,
            spine: opf_metadata.spine.clone(),
            has_cover: opf_metadata.cover_image_id.is_some(),
            cover_image_format: opf_metadata
                .cover_image_id
                .as_ref()
                .and_then(|id| opf_metadata.manifest.get(id).map(|(_, mime)| mime.clone())),
            chapter_count: opf_metadata.spine.len(),
        };

        self.metadata = Some(epub_metadata.clone());
        Ok(epub_metadata)
    }

    /// Validates the metadata against basic EPUB standard requirements
    pub async fn validate_metadata(&mut self) -> Result<()> {
        let metadata = self.get_metadata().await?;
        metadata
            .validate()
            .map_err(crate::error::LexEpubError::ValidationError)
    }

    pub fn validate_metadata_sync(&mut self) -> Result<()> {
        futures::executor::block_on(self.validate_metadata())
    }

    /// Get total word count across all chapters
    pub async fn total_word_count(&mut self) -> Result<usize> {
        let chapters = self.extract_chapters().await?;
        Ok(chapters.iter().map(|c| c.word_count).sum())
    }

    /// Synchronous wrapper for `total_word_count`
    pub fn total_word_count_sync(&mut self) -> Result<usize> {
        futures::executor::block_on(self.total_word_count())
    }

    /// Get total character count across all chapters
    pub async fn total_char_count(&mut self) -> Result<usize> {
        let chapters = self.extract_chapters().await?;
        Ok(chapters.iter().map(|c| c.char_count).sum())
    }

    /// Synchronous wrapper for `total_char_count`
    pub fn total_char_count_sync(&mut self) -> Result<usize> {
        futures::executor::block_on(self.total_char_count())
    }

    /// Check if the EPUB has a cover image
    pub async fn has_cover(&mut self) -> Result<bool> {
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let cover_id = opf_parser.get_cover_image_id(&opf_data)?;

        Ok(cover_id.is_some())
    }

    /// Extract the cover image bytes from the EPUB
    pub async fn cover_image(&mut self) -> Result<Vec<u8>> {
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let metadata = opf_parser.parse_metadata(&opf_data)?;

        let cover_id = metadata
            .cover_image_id
            .ok_or_else(|| LexEpubError::MissingFile("No cover image found in EPUB".to_string()))?;

        let cover_href = metadata.manifest.get(&cover_id).ok_or_else(|| {
            LexEpubError::MissingFile(format!("Cover image item '{}' not in manifest", cover_id))
        })?;

        // Resolve the cover href relative to the OPF file's directory
        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let full_path = opf_base.join(&cover_href.0);
        let full_path_str = full_path.to_string_lossy();

        self.extractor.read_file(&full_path_str).await
    }

    /// Stream the cover image bytes directly to a given parameter implementing futures::AsyncWrite.
    /// This avoids allocating a buffer for the entire image in memory.
    pub async fn cover_image_to_writer<W: futures::AsyncWrite + Unpin + Send>(
        &mut self,
        writer: &mut W,
    ) -> Result<u64> {
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let metadata = opf_parser.parse_metadata(&opf_data)?;

        let cover_id = metadata
            .cover_image_id
            .ok_or_else(|| LexEpubError::MissingFile("No cover image found in EPUB".to_string()))?;

        let cover_href = metadata.manifest.get(&cover_id).ok_or_else(|| {
            LexEpubError::MissingFile(format!("Cover image item '{}' not in manifest", cover_id))
        })?;

        // Resolve the cover href relative to the OPF file's directory
        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let full_path = opf_base.join(&cover_href.0);
        let full_path_str = full_path.to_string_lossy();

        self.extractor.read_file_to_writer(&full_path_str, writer).await
    }

    pub async fn extract_with_ast(&mut self) -> Result<Vec<ParsedChapter>> {
        self.extract_ast().await
    }

    // Internal method to extract chapters
    async fn extract_chapters(&mut self) -> Result<Vec<ParsedChapter>> {
        if let Some(ref chapters) = self.chapters {
            return Ok(chapters.clone());
        }

        // Get OPF location
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        // Parse OPF for spine and manifest
        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let spine = opf_parser.parse_spine(&opf_data)?;
        let metadata = opf_parser.parse_metadata(&opf_data)?;

        // Extract chapters
        let mut chapters = Vec::new();
        // Get the base directory of the OPF file for resolving relative hrefs
        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));

        for item_id in spine {
            if let Some(href) = metadata.manifest.get(&item_id) {
                // Resolve the href relative to the OPF file's directory
                let full_path = opf_base.join(&href.0);
                let full_path_str = full_path.to_string_lossy();
                match self.extractor.read_file(&full_path_str).await {
                    Ok(content) => {
                        // Parse HTML content
                        let chapter = Chapter {
                            href: full_path_str.to_string(),
                            id: item_id,
                            media_type: "application/xhtml+xml".to_string(), // TODO: Assume XHTML
                            content,
                        };

                        let parser = crate::core::html_parser::ChapterParser::new().with_both();
                        let parsed_chapter = parser.parse_chapter(chapter)?;

                        chapters.push(parsed_chapter);
                    }
                    Err(_) => {
                        // Skip chapters that can't be read
                        continue;
                    }
                }
            }
        }

        // Cache chapters only when `lowmem` feature is not enabled
        // low-memory targets should avoid keeping the entire chapter list in
        // memory.
        #[cfg(not(feature = "lowmem"))]
        {
            self.chapters = Some(chapters.clone());
        }

        Ok(chapters)
    }
}

/// Convenience function for quick text extraction
pub async fn extract_text_only<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut epub = LexEpub::open(path).await?;
    epub.extract_text_only().await
}

/// Convenience function for AST extraction
pub async fn extract_ast<P: AsRef<Path>>(path: P) -> Result<Vec<ParsedChapter>> {
    let mut epub = LexEpub::open(path).await?;
    epub.extract_ast().await
}

/// Convenience function for metadata extraction
pub async fn get_metadata<P: AsRef<Path>>(path: P) -> Result<EpubMetadata> {
    let mut epub = LexEpub::open(path).await?;
    epub.get_metadata().await
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnalysisReport {
    pub metadata: Option<EpubMetadata>,
    pub chapter_count: usize,
    pub total_words: usize,
    pub total_chars: usize,
    pub first_chapter_preview: Option<String>,
}

/// Analyze an EPUB from an async reader (streaming, does not require full-copy).
pub async fn analyze_reader<R>(reader: R) -> Result<AnalysisReport>
where
    R: futures::AsyncRead + futures::AsyncSeek + Unpin + Send + 'static,
{
    use async_zip::base::read::seek::ZipFileReader;
    use futures::AsyncReadExt;

    let reader = futures::io::BufReader::new(reader);
    let mut archive = ZipFileReader::new(reader)
        .await
        .map_err(LexEpubError::Zip)?;

    // helper to read an entry by path
    async fn read_entry<Rdr>(archive: &mut ZipFileReader<Rdr>, path: &str) -> Result<Vec<u8>>
    where
        Rdr: futures::AsyncBufRead + futures::AsyncSeek + Unpin,
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
                crate::error::LexEpubError::MissingFile(format!(
                    "File '{}' not found in EPUB",
                    path
                ))
            })?;

        let mut entry_reader = archive
            .reader_without_entry(entry_index)
            .await
            .map_err(LexEpubError::Zip)?;
        let mut buf = Vec::new();
        entry_reader
            .read_to_end(&mut buf)
            .await
            .map_err(LexEpubError::Io)?;
        Ok(buf)
    }

    // Read container.xml
    let container_data = read_entry(&mut archive, "META-INF/container.xml").await?;
    let mut container_parser = ContainerParser::new();
    let opf_path = container_parser
        .parse_container(&container_data)?
        .rootfile_path;

    // Read OPF
    let opf_data = read_entry(&mut archive, &opf_path).await?;
    let mut opf_parser = OpfParser::new();
    let spine = opf_parser.parse_spine(&opf_data)?;
    let metadata = opf_parser.parse_metadata(&opf_data)?;

    // Extract chapter data
    let mut chapters_parsed = Vec::new();
    let opf_base = std::path::Path::new(&opf_path)
        .parent()
        .unwrap_or(std::path::Path::new(""));

    for item_id in spine {
        if let Some(href) = metadata.manifest.get(&item_id) {
            let full_path = opf_base.join(&href.0);
            let full_path_str = full_path.to_string_lossy();
            if let Ok(content) = read_entry(&mut archive, &full_path_str).await {
                let html_content = String::from_utf8_lossy(&content);
                let text_content = extract_text_content(&html_content)?;
                let word_count = text_content.split_whitespace().count();
                let char_count = text_content.chars().count();

                chapters_parsed.push((text_content, word_count, char_count));
            }
        }
    }

    let chapter_count = chapters_parsed.len();
    let total_words: usize = chapters_parsed.iter().map(|(_, w, _)| *w).sum();
    let total_chars: usize = chapters_parsed.iter().map(|(_, _, c)| *c).sum();
    let first_chapter_preview = chapters_parsed
        .first()
        .map(|(s, _, _)| s.chars().take(300).collect::<String>());

    let epub_metadata = EpubMetadata {
        title: metadata.title,
        version: metadata.version,
        authors: metadata.creators,
        description: metadata.description,
        languages: metadata.languages,
        subjects: metadata.subjects,
        publisher: metadata.publisher,
        publication_date: metadata.date,
        identifiers: metadata.identifiers,
        rights: metadata.rights,
        contributors: metadata.contributors,
        spine: metadata.spine.clone(),
        has_cover: metadata.cover_image_id.is_some(),
        cover_image_format: metadata
            .cover_image_id
            .as_ref()
            .and_then(|id| metadata.manifest.get(id).map(|(_, mime)| mime.clone())),
        chapter_count: metadata.spine.len(),
    };

    Ok(AnalysisReport {
        metadata: Some(epub_metadata),
        chapter_count,
        total_words,
        total_chars,
        first_chapter_preview,
    })
}

/// Analyze from a blocking reader (wraps with `AllowStdIo`)
pub fn analyze_sync_reader<R>(reader: R) -> Result<AnalysisReport>
where
    R: std::io::Read + std::io::Seek + Send + 'static,
{
    let allow = futures::io::AllowStdIo::new(reader);
    futures::executor::block_on(analyze_reader(allow))
}

/// Convenience: analyze an EPUB from a file path (streaming from disk).
pub async fn analyze_path<P: AsRef<Path>>(path: P) -> Result<AnalysisReport> {
    let file = std::fs::File::open(path.as_ref()).map_err(LexEpubError::Io)?;
    let reader = futures::io::AllowStdIo::new(file);
    analyze_reader(reader).await
}
