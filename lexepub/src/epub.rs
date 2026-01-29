use crate::error::Result;
use crate::metadata::EpubMetadata;
use crate::parser::{ChapterParser, ParsedChapter};
use bytes::Bytes;
use std::path::Path;

// Re-export the main types for convenience
pub use crate::reader::LexEpub;

/// Additional utilities for working with EPUB files
impl LexEpub {
    /// Open an EPUB from a file path
    pub async fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open(path).await
    }

    /// Parse all chapters and return them as a vector
    pub async fn parse_all_chapters(&mut self) -> Result<Vec<ParsedChapter>> {
        self.chapters().await
    }

    /// Extract only text content from all chapters (fastest??)
    pub async fn extract_text_only(&mut self) -> Result<Vec<String>> {
        let parser = ChapterParser::new().text_only();
        self.set_chapter_parser(parser);

        let chapters = self.chapters().await?;
        Ok(chapters.into_iter().map(|c| c.content).collect())
    }

    /// Extract chapters with AST for advanced processing
    pub async fn extract_with_ast(&mut self) -> Result<Vec<ParsedChapter>> {
        let parser = ChapterParser::new().with_ast();
        self.set_chapter_parser(parser);

        self.chapters().await
    }

    /// Get metadata as a convenient struct
    pub async fn get_metadata(&mut self) -> Result<EpubMetadata> {
        let metadata_ref = self.metadata().await?;
        Ok(metadata_ref.clone())
    }

    /// Check if EPUB has a cover image
    pub async fn has_cover(&mut self) -> Result<bool> {
        let cover = self.cover_image().await?;
        Ok(cover.is_some())
    }

    /// Get total word count across all chapters
    pub async fn total_word_count(&mut self) -> Result<usize> {
        let chapters = self.chapters().await?;
        Ok(chapters.iter().map(|c| c.word_count).sum())
    }

    /// Get total character count across all chapters
    pub async fn total_char_count(&mut self) -> Result<usize> {
        let chapters = self.chapters().await?;
        Ok(chapters.iter().map(|c| c.char_count).sum())
    }
}

/// Convenience function for quick text extraction
pub async fn extract_text_from_epub<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut epub = LexEpub::open(path).await?;
    epub.extract_text_only().await
}

/// Convenience function for quick text extraction from bytes
pub async fn extract_text_from_bytes(data: Bytes) -> Result<Vec<String>> {
    let mut epub = LexEpub::from_bytes(data).await?;
    epub.extract_text_only().await
}

/// Convenience function for quick metadata extraction
pub async fn extract_metadata<P: AsRef<Path>>(path: P) -> Result<EpubMetadata> {
    let mut epub = LexEpub::open(path).await?;
    epub.get_metadata().await
}
