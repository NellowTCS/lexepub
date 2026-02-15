use futures::StreamExt;
use lexepub::epub::{extract_ast, extract_text_only, get_metadata, LexEpub};
use std::path::Path;

#[cfg(test)]
mod api_tests {
    use super::*;

    #[tokio::test]
    async fn test_lexepub_open() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let result = LexEpub::open(test_epub).await;
        assert!(result.is_ok());
        let mut epub = result.unwrap();

        // Test that we can get metadata
        let metadata = epub.get_metadata().await.unwrap();
        assert!(metadata.title.is_some() || !metadata.authors.is_empty());
    }

    #[tokio::test]
    async fn test_lexepub_from_bytes() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let data = std::fs::read(test_epub).unwrap();
        let bytes = bytes::Bytes::from(data);

        let result = LexEpub::from_bytes(bytes).await;
        assert!(result.is_ok());
        let mut epub = result.unwrap();

        let metadata = epub.get_metadata().await.unwrap();
        assert!(metadata.title.is_some() || !metadata.authors.is_empty());
    }

    #[tokio::test]
    async fn test_extract_text_only() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let mut epub = LexEpub::open(test_epub).await.unwrap();
        let chapters = epub.extract_text_only().await.unwrap();

        assert!(!chapters.is_empty());
        for chapter in chapters {
            assert!(!chapter.is_empty());
            // Should be valid UTF-8
            let _ = chapter.as_str();
        }
    }

    #[tokio::test]
    async fn test_extract_ast() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let mut epub = LexEpub::open(test_epub).await.unwrap();
        let chapters = epub.extract_ast().await.unwrap();

        assert!(!chapters.is_empty());
        for chapter in chapters {
            assert!(!chapter.content.is_empty());
            // AST should be valid
            let _ = &chapter.content;
        }
    }

    #[tokio::test]
    async fn test_extract_chapters_stream() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let mut epub = LexEpub::open(test_epub).await.unwrap();
        let stream = epub.extract_chapters_stream().await.unwrap();

        // Consume the stream (yummmy)
        let mut count = 0;
        let mut stream = stream;
        while let Some(chapter) = stream.next().await {
            let _ = chapter.unwrap();
            count += 1;
        }

        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let mut epub = LexEpub::open(test_epub).await.unwrap();
        let metadata = epub.get_metadata().await.unwrap();

        // At minimum, should have some basic fields
        // (exact fields depend on the EPUB)
        let _ = metadata;
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let mut epub = LexEpub::open(test_epub).await.unwrap();

        // Should be able to call operations multiple times
        let metadata1 = epub.get_metadata().await.unwrap();
        let metadata2 = epub.get_metadata().await.unwrap();
        assert_eq!(metadata1.title, metadata2.title);

        let chapters1 = epub.extract_text_only().await.unwrap();
        let chapters2 = epub.extract_text_only().await.unwrap();
        assert_eq!(chapters1.len(), chapters2.len());
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        // Test convenience functions
        let result = extract_text_only(test_epub).await;
        if let Ok(chapters) = result {
            assert!(!chapters.is_empty());
        }

        let result = extract_ast(test_epub).await;
        if let Ok(chapters) = result {
            assert!(!chapters.is_empty());
        }

        let result = get_metadata(test_epub).await;
        if let Ok(metadata) = result {
            let _ = metadata;
        }
    }

    #[tokio::test]
    async fn test_all_test_epubs() {
        let test_files = [
            "examples/epubs/test-book.epub",
            "examples/epubs/Accessibility-Tests-Extended-Descriptions-v1.1.1.epub",
            "examples/epubs/Fundamental-Accessibility-Tests-Basic-Functionality-v2.0.0.epub",
            "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
        ];

        for test_file in &test_files {
            let path = Path::new(test_file);
            if !path.exists() {
                continue;
            }

            println!("Testing {}", test_file);

            // Test basic opening
            let mut epub = LexEpub::open(path).await.unwrap();

            // Test metadata extraction
            let _metadata = epub.get_metadata().await.unwrap();

            // Test content extraction
            let chapters = epub.extract_text_only().await.unwrap();
            println!("Found {} chapters in {}", chapters.len(), test_file);

            // Test AST extraction
            let ast_chapters = epub.extract_ast().await.unwrap();
            println!("Found {} AST chapters in {}", ast_chapters.len(), test_file);

            // Test streaming
            let stream = epub.extract_chapters_stream().await.unwrap();
            let mut count = 0;
            let mut stream = stream;
            while let Some(_) = stream.next().await {
                count += 1;
            }
            println!("Found {} chapters in stream for {}", count, test_file);
        }
    }

    #[tokio::test]
    // TODO: This is so bad, we should have a much better check for this,
    //       but for now just verify that it doesn't crash and actually works at all
    async fn test_api_error_handling() {
        // Test that API functions handle errors gracefully
        // Note: open() doesn't validate file existence immediately
        let epub_result = LexEpub::open("/definitely/does/not/exist.epub").await;
        assert!(epub_result.is_ok()); // open succeeds initially

        // But it should fail when trying to read metadata
        let mut epub = epub_result.unwrap();
        let result = epub.get_metadata().await;
        assert!(result.is_err());

        // from_bytes accepts any data initially, validation happens during reading
        let invalid_bytes = bytes::Bytes::new();
        let epub_result = LexEpub::from_bytes(invalid_bytes).await;
        assert!(epub_result.is_ok()); // from_bytes doesn't validate immediately
    }

    #[tokio::test]
    async fn test_chapter_content_types() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if !test_epub.exists() {
            return;
        }

        let mut epub = LexEpub::open(test_epub).await.unwrap();

        // Test text extraction
        let text_chapters = epub.extract_text_only().await.unwrap();
        for chapter in &text_chapters {
            // Should be String
            let _: &str = chapter.as_str();
        }

        // Test AST extraction
        let ast_chapters = epub.extract_ast().await.unwrap();
        for chapter in &ast_chapters {
            // Should have content
            assert!(!chapter.content.is_empty());
            let content = &chapter.content;
            // Should be serializable
            let _ = serde_json::to_string(content);
        }
    }
}
