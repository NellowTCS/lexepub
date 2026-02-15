use bytes::Bytes;
use lexepub::epub::LexEpub;

/// This entire file needs a lot of work, but for now it's just a placeholder for
/// various edge case tests that we should add eventually.

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_epub() {
        // Test with completely empty data
        let empty_data = Bytes::new();
        let epub_result = LexEpub::from_bytes(empty_data).await;
        assert!(epub_result.is_ok()); // from_bytes succeeds initially
                                      // (uh... yeah i don't think it should do that but it does, we should fix this eventually)

        // But should fail when trying to read
        let mut epub = epub_result.unwrap();
        let result = epub.get_metadata().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_minimal_epub() {
        // Test with minimal valid EPUB structure
        // TODO: This would require creating a minimal valid EPUB
        // For now, test that existing EPUBs work
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let result = LexEpub::open(test_epub).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_special_characters() {
        // Test EPUB with Unicode characters in filenames and content
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let chapters = epub.extract_text_only().await.unwrap();

            // Check that Unicode content is handled properly
            for chapter in chapters {
                // Should not panic on any Unicode content
                let _ = chapter.as_str();
            }
        }
    }

    #[tokio::test]
    async fn test_epub_with_empty_chapters() {
        // Test EPUB with chapters that have no content
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let chapters = epub.extract_text_only().await.unwrap();

            // Ensure no chapters are completely empty (though empty chapters might be valid somehow...)
            let has_content = chapters.iter().any(|c| !c.is_empty());
            assert!(has_content, "At least one chapter should have content");
        }
    }

    #[tokio::test]
    async fn test_epub_with_nested_html() {
        // Test EPUB with deeply nested HTML structures
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let ast_chapters = epub.extract_ast().await.unwrap();

            // Check that nested structures are parsed correctly
            for chapter in ast_chapters {
                // Should handle arbitrary nesting depth
                assert!(!chapter.content.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_epub_with_broken_html() {
        // Test EPUB with malformed HTML
        // TODO: This would require creating mock data
        // For now, test that valid HTML works
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_ast().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_missing_manifest_items() {
        // Test EPUB where some manifest items are missing
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_duplicate_ids() {
        // Test EPUB with duplicate IDs in manifest/spine
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_circular_references() {
        // Test EPUB with circular references in spine
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_very_long_filenames() {
        // Test EPUB with extremely long filenames
        // TODO: This would require creating a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_binary_content_in_chapters() {
        // Test EPUB with binary data embedded in HTML
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_multiple_rootfiles() {
        // Test EPUB with multiple rootfiles in container.xml
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.get_metadata().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_no_spine() {
        // Test EPUB with no spine element
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_epub_with_empty_metadata() {
        // Test EPUB with minimal metadata
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let metadata = epub.get_metadata().await.unwrap();
            // TODO: Metadata might be empty but should not crash. Why? Because ePubs are a
            // complete mess and we should be able to handle even really bad ones without crashing.
            // (also I'm kinda tired of Rust for now and just want to get something merged, we can improve this later)
            let _ = metadata;
        }
    }

    #[tokio::test]
    async fn test_epub_with_compressed_content() {
        // Test EPUB with different compression methods
        let test_files = ["examples/epubs/test-book.epub"];

        for test_file in &test_files {
            let path = std::path::Path::new(test_file);
            if !path.exists() {
                continue;
            }

            let mut epub = LexEpub::open(path).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok(), "Should handle compression in {}", test_file);
        }
    }

    #[tokio::test]
    async fn test_epub_with_large_manifest() {
        // Test EPUB with many files in manifest
        // TODO: This would require a mock EPUB
        let test_epub = std::path::Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
        }
    }
}
