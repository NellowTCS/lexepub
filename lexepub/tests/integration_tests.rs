use lexepub::epub::{extract_text_only, get_metadata, LexEpub};
use std::path::Path;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn get_test_epub_path() -> Option<&'static Path> {
        let path = Path::new("examples/epubs/test-book.epub");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[tokio::test]
    async fn test_full_epub_processing_pipeline() {
        let test_epub = match get_test_epub_path() {
            Some(path) => path,
            None => return, // Skip test if no test EPUB available
        };

        // Open EPUB
        let mut epub = LexEpub::open(test_epub).await.unwrap();

        // Extract metadata
        let metadata = epub.get_metadata().await.unwrap();
        assert!(
            metadata.title.is_some() && !metadata.title.as_ref().unwrap().is_empty(),
            "Title should not be empty"
        );
        assert!(
            !metadata.authors.is_empty(),
            "Should have at least one author"
        );
        assert!(
            !metadata.languages.is_empty(),
            "Should have at least one language"
        );

        // Extract text content
        let chapters = epub.extract_text_only().await.unwrap();
        assert!(!chapters.is_empty(), "Should have chapters");

        // Verify word and character counts
        let total_words = epub.total_word_count().await.unwrap();
        let total_chars = epub.total_char_count().await.unwrap();
        assert!(total_words > 0, "Should have words");
        assert!(
            total_chars > total_words as usize,
            "Characters should exceed words"
        );

        // Test AST extraction
        let ast_chapters = epub.extract_ast().await.unwrap();
        assert_eq!(
            ast_chapters.len(),
            chapters.len(),
            "AST chapters should match text chapters"
        );

        for ast_chapter in &ast_chapters {
            // AST parsing not yet implemented, so ast will be None
            assert!(
                !ast_chapter.content.is_empty(),
                "Content should not be empty"
            );
        }
    }

    #[tokio::test]
    async fn test_convenience_functions_integration() {
        let test_epub = match get_test_epub_path() {
            Some(path) => path,
            None => return,
        };

        // Test extract_text_from_epub
        let chapters = extract_text_only(test_epub).await.unwrap();
        assert!(!chapters.is_empty());

        // Test extract_metadata
        let metadata = get_metadata(test_epub).await.unwrap();
        assert!(metadata.title.is_some());

        // Verify consistency between methods
        let mut epub = LexEpub::open(test_epub).await.unwrap();
        let direct_metadata = epub.get_metadata().await.unwrap();
        let direct_chapters = epub.extract_text_only().await.unwrap();

        assert_eq!(metadata.title, direct_metadata.title);
        assert_eq!(chapters, direct_chapters);
    }

    #[tokio::test]
    async fn test_chapter_streaming() {
        let test_epub = match get_test_epub_path() {
            Some(path) => path,
            None => return,
        };

        let mut epub = LexEpub::open(test_epub).await.unwrap();

        // Test streaming chapters
        let mut stream = epub.extract_chapters_stream().await.unwrap();
        let mut count = 0;
        let mut total_words = 0;

        use futures::StreamExt;
        while let Some(result) = stream.next().await {
            let chapter = result.unwrap();
            count += 1;
            total_words += chapter.word_count;
            assert!(!chapter.content.is_empty());
        }

        assert!(count > 0, "Should have chapters");
        assert!(total_words > 0, "Should have words");
    }

    // TODO: uncomment this test once has_cover() and cover_image() methods are implemented
    // #[tokio::test]
    // async fn test_cover_image_handling() {
    //     let test_epub = match get_test_epub_path() {
    //         Some(path) => path,
    //         None => return,
    //     };

    //     let mut epub = LexEpub::open(test_epub).await.unwrap();

    //     // Test cover detection
    //     let has_cover = epub.has_cover().await.unwrap();

    //     // Test cover extraction (may or may not have a cover)
    //     let cover_result = epub.cover_image().await;
    //     assert!(cover_result.is_ok());

    //     let cover_data = cover_result.unwrap();
    //     if has_cover {
    //         assert!(cover_data.is_some(), "Should have cover data if has_cover is true");
    //     }
    // }

    #[tokio::test]
    async fn test_error_handling() {
        // Test with non-existent file (open succeeds initially, fails on read)
        let epub_result = LexEpub::open("nonexistent.epub").await;
        assert!(epub_result.is_ok());
        let mut epub = epub_result.unwrap();
        let result = epub.get_metadata().await;
        assert!(result.is_err());

        // Test with invalid data (from_bytes succeeds initially, fails on read)
        let epub_result = LexEpub::from_bytes(bytes::Bytes::from("invalid")).await;
        assert!(epub_result.is_ok());
        let mut epub = epub_result.unwrap();
        let result = epub.get_metadata().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parser_configuration() {
        let test_epub = match get_test_epub_path() {
            Some(path) => path,
            None => return,
        };

        let mut epub = LexEpub::open(test_epub).await.unwrap();

        // Test text-only parsing
        let text_chapters = epub.extract_text_only().await.unwrap();
        assert!(!text_chapters.is_empty());

        // Reset and test AST parsing
        let ast_chapters = epub.extract_ast().await.unwrap();
        assert!(!ast_chapters.is_empty());

        // Verify AST structure
        for chapter in ast_chapters {
            if let Some(ast) = chapter.ast {
                assert_ast_structure(&ast);
            }
        }
    }

    fn assert_ast_structure(ast: &lexepub::AstNode) {
        match ast {
            lexepub::AstNode::Element { tag, children, .. } => {
                assert!(!tag.is_empty(), "Tag should not be empty");
                for child in children {
                    assert_ast_structure(child);
                }
            }
            lexepub::AstNode::Text { content } => {
                // Text content can be empty (whitespace)
                let _ = content;
            }
            lexepub::AstNode::Comment { content } => {
                // Comments can be empty
                let _ = content;
            }
        }
    }

    #[tokio::test]
    async fn test_memory_efficiency() {
        let test_epub = match get_test_epub_path() {
            Some(path) => path,
            None => return,
        };

        // Test that we can process the same EPUB multiple times
        for _ in 0..3 {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let _metadata = epub.get_metadata().await.unwrap();
            let _chapters = epub.extract_text_only().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_large_content_handling() {
        let test_epub = match get_test_epub_path() {
            Some(path) => path,
            None => return,
        };

        let mut epub = LexEpub::open(test_epub).await.unwrap();
        let chapters = epub.extract_text_only().await.unwrap();

        // Verify that all chapters have reasonable content
        for (i, chapter) in chapters.iter().enumerate() {
            assert!(!chapter.is_empty(), "Chapter {} should not be empty", i);
            // Basic sanity check for content length
            assert!(
                chapter.len() < 100000,
                "Chapter {} seems unreasonably large",
                i
            );
        }
    }
}
