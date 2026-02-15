use crate::core::chapter::{Chapter, ChapterStream, ParsedChapter};
use crate::core::container::ContainerParser;
use crate::core::extractor::EpubExtractor;
use crate::core::html_parser::extract_text_content;
use crate::core::opf_parser::OpfParser;
use crate::error::Result;
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
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub languages: Vec<String>,
    pub subjects: Vec<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub identifiers: Vec<String>,
    pub rights: Option<String>,
    pub contributors: Vec<String>,
    // TODO: add spine field (Vec<String>) for chapter order
    // TODO: add has_cover field (bool) for cover image presence
    // TODO: add chapter_count field (usize) for number of chapters
    // TODO: rename date to publication_date for API consistency
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

    /// Create an EPUB from bytes
    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        let extractor = EpubExtractor::from_bytes(data).await?;
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
        let chapters = self.extract_chapters().await?;
        Ok(ChapterStream::new(chapters))
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
            authors: opf_metadata.creators,
            description: opf_metadata.description,
            languages: opf_metadata.languages,
            subjects: opf_metadata.subjects,
            publisher: opf_metadata.publisher,
            date: opf_metadata.date,
            identifiers: opf_metadata.identifiers,
            rights: opf_metadata.rights,
            contributors: opf_metadata.contributors,
        };

        self.metadata = Some(epub_metadata.clone());
        Ok(epub_metadata)
    }

    /// Get total word count across all chapters
    pub async fn total_word_count(&mut self) -> Result<usize> {
        let chapters = self.extract_chapters().await?;
        Ok(chapters.iter().map(|c| c.word_count).sum())
    }

    /// Get total character count across all chapters
    pub async fn total_char_count(&mut self) -> Result<usize> {
        let chapters = self.extract_chapters().await?;
        Ok(chapters.iter().map(|c| c.char_count).sum())
    }

    // TODO: implement has_cover() method, check OPF manifest for cover image
    // TODO: implement cover_image() method, extract cover image data from EPUB
    // TODO: implement extract_with_ast() method as alias for extract_ast() for API consistency? or just use one method name?

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
                let full_path = opf_base.join(href);
                let full_path_str = full_path.to_string_lossy();
                match self.extractor.read_file(&full_path_str).await {
                    Ok(content) => {
                        // Parse HTML content
                        let html_content = String::from_utf8_lossy(&content);
                        let text_content = extract_text_content(&html_content)?;
                        let word_count = text_content.split_whitespace().count();
                        let char_count = text_content.chars().count();

                        let chapter = Chapter {
                            href: full_path_str.to_string(),
                            id: item_id,
                            media_type: "application/xhtml+xml".to_string(), // TODO: Assume XHTML
                            content,
                        };

                        let parsed_chapter = ParsedChapter {
                            chapter_info: chapter,
                            content: text_content,
                            ast: None, // TODO: implement AST parsing, use ChapterParser::with_ast() instead of extract_text_content
                            word_count,
                            char_count,
                        };

                        chapters.push(parsed_chapter);
                    }
                    Err(_) => {
                        // Skip chapters that can't be read
                        continue;
                    }
                }
            }
        }

        self.chapters = Some(chapters.clone());
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
