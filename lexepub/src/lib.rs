pub mod core;
pub mod epub;
pub mod error;

#[cfg(feature = "c-ffi")]
pub mod ffi;

// Re-export core modules for internal use
pub use core::chapter::{AstNode, Chapter, ChapterStream, ParsedChapter};
pub use core::container::ContainerParser;
pub use core::extractor::EpubExtractor;
pub use core::html_parser::ChapterParser;
pub use core::opf_parser::OpfParser;

// Re-export main API
pub use epub::{extract_ast, extract_text_only, get_metadata, LexEpub};
pub use error::{LexEpubError, Result};

// Re-export metadata types
pub use epub::EpubMetadata;

/// Re-export common types
pub mod prelude {
    pub use crate::core::chapter::{AstNode, Chapter, ChapterStream, ParsedChapter};
    pub use crate::core::extractor::EpubExtractor;
    pub use crate::core::html_parser::ChapterParser;
    pub use crate::epub::EpubMetadata;
    pub use crate::epub::LexEpub;
    pub use crate::error::{LexEpubError, Result};
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_open_epub() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if test_epub.exists() {
                let result = LexEpub::open(test_epub).await;
                assert!(
                    result.is_ok(),
                    "Failed to open test EPUB: {:?}",
                    result.err()
                );
            }
        });
    }

    #[test]
    fn test_extract_metadata() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if test_epub.exists() {
                let mut epub = LexEpub::open(test_epub).await.unwrap();
                let metadata = epub.get_metadata().await;
                assert!(
                    metadata.is_ok(),
                    "Failed to extract metadata: {:?}",
                    metadata.err()
                );

                let metadata = metadata.unwrap();
                // Just check that we got some metadata, don't assume specific content
                assert!(
                    !metadata.languages.is_empty()
                        || metadata.title.is_some()
                        || !metadata.authors.is_empty(),
                    "Should have some metadata"
                );
            }
        });
    }

    #[test]
    fn test_extract_text() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if test_epub.exists() {
                let mut epub = LexEpub::open(test_epub).await.unwrap();
                let chapters = epub.extract_text_only().await;
                assert!(
                    chapters.is_ok(),
                    "Failed to extract text: {:?}",
                    chapters.err()
                );

                let chapters = chapters.unwrap();
                assert!(!chapters.is_empty(), "Should have chapters");
            }
        });
    }

    #[test]
    fn test_convenience_functions() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if test_epub.exists() {
                // Test extract_text_only
                let result = extract_text_only(test_epub).await;
                assert!(
                    result.is_ok(),
                    "extract_text_only failed: {:?}",
                    result.err()
                );

                // Test extract_ast
                let result = extract_ast(test_epub).await;
                assert!(result.is_ok(), "extract_ast failed: {:?}", result.err());

                // Test get_metadata
                let result = get_metadata(test_epub).await;
                assert!(result.is_ok(), "get_metadata failed: {:?}", result.err());
            }
        });
    }

    #[test]
    fn test_chapter_parsing() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if test_epub.exists() {
                let mut epub = LexEpub::open(test_epub).await.unwrap();

                // Test text-only parsing
                let text_chapters = epub.extract_text_only().await.unwrap();
                assert!(!text_chapters.is_empty());

                // Test AST parsing
                let ast_chapters = epub.extract_ast().await.unwrap();
                assert!(!ast_chapters.is_empty());

                // Verify AST structure (TODO: currently not implemented)
                for chapter in &ast_chapters {
                    // AST parsing not yet implemented
                    let _ = chapter;
                }
            }
        });
    }

    #[test]
    fn test_word_and_char_counts() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if test_epub.exists() {
                let mut epub = LexEpub::open(test_epub).await.unwrap();

                let word_count = epub.total_word_count().await.unwrap();
                let char_count = epub.total_char_count().await.unwrap();

                assert!(word_count > 0, "Word count should be greater than 0");
                assert!(char_count > 0, "Character count should be greater than 0");
                assert!(
                    char_count > word_count,
                    "Character count should be greater than word count"
                );
            }
        });
    }
}
