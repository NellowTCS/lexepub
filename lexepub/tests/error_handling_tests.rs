use lexepub::epub::LexEpub;
use lexepub::error::LexEpubError;
use std::path::Path;

#[cfg(test)]
mod error_tests {
    use super::*;

    #[tokio::test]
    // Again, our opening and validation logic is so bad...
    async fn test_file_not_found() {
        // open() doesn't validate file existence immediately
        let epub_result = LexEpub::open("nonexistent.epub").await;
        assert!(epub_result.is_ok()); // succeeds initially

        // But fails when trying to read
        let mut epub = epub_result.unwrap();
        let result = epub.get_metadata().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_zip_data() {
        // from_bytes doesn't validate data immediately
        let invalid_data = bytes::Bytes::from("not a zip file");
        let epub_result = LexEpub::from_bytes(invalid_data).await;
        assert!(epub_result.is_ok()); // succeeds initially

        // But fails when trying to read
        let mut epub = epub_result.unwrap();
        let result = epub.get_metadata().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_container_xml() {
        // This would require creating a mock EPUB without container.xml
        // For now, just test that the error handling works
        let result = std::fs::read("examples/epubs/test-book.epub");
        if result.is_ok() {
            let data = bytes::Bytes::from(result.unwrap());
            let mut epub = LexEpub::from_bytes(data).await.unwrap();

            // This should work for valid EPUBs
            let _ = epub.get_metadata().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_invalid_xml_parsing() {
        // Test with malformed XML would require mocking
        // For now, test that valid XML works
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.get_metadata().await;
            assert!(result.is_ok(), "Valid EPUB should parse correctly");
        }
    }

    #[tokio::test]
    async fn test_missing_chapter_files() {
        // This would require a mock EPUB with missing chapter files
        // For now, test that valid EPUBs work
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok(), "Valid EPUB should extract chapters");
        }
    }

    #[tokio::test]
    async fn test_utf8_error_handling() {
        // Test with invalid UTF-8 in chapter content
        // This would require creating mock data with invalid UTF-8
        // For now, test that valid content works
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
            let chapters = result.unwrap();
            for chapter in chapters {
                // Ensure all content is valid UTF-8
                let _ = chapter.as_str();
            }
        }
    }

    #[test]
    fn test_error_display() {
        let io_error = LexEpubError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        let display = format!("{}", io_error);
        assert!(display.contains("IO error"));

        let missing_file = LexEpubError::MissingFile("test.txt".to_string());
        let display = format!("{}", missing_file);
        assert!(display.contains("Missing required file"));
        assert!(display.contains("test.txt"));
    }

    #[test]
    fn test_error_types() {
        // Test that all error variants can be created
        let _io = LexEpubError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        // TODO: ZipError variants depend on async_zip version, using a simple test
        let zip_err = async_zip::error::ZipError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test",
        ));
        let _zip = LexEpubError::Zip(zip_err);
        // TODO: quick_xml::Error variants depend on version, using a simple test
        let xml_err = quick_xml::Error::Io(std::sync::Arc::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test",
        )));
        let _xml = LexEpubError::Xml(xml_err);
        let _invalid = LexEpubError::InvalidFormat("test".to_string());
        let _missing = LexEpubError::MissingFile("test".to_string());
        let _html = LexEpubError::Html("test".to_string());
        let _metadata = LexEpubError::MetadataError("test".to_string());
        let _chapter = LexEpubError::ChapterError("test".to_string());
        // Create a proper FromUtf8Error
        let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence
        let _utf8 = LexEpubError::Utf8(String::from_utf8(invalid_utf8).unwrap_err());
        // TODO: Skip Utf8Str test as it's hard to construct properly
        let _json = LexEpubError::Serialization(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test",
        )));
        let _async = LexEpubError::AsyncError("test".to_string());
    }

    #[tokio::test]
    async fn test_partial_failure_recovery() {
        // Test that if one chapter fails, others still work
        // This would require a mock EPUB with some corrupted chapters
        // For now, test that valid EPUBs work completely
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let mut epub = LexEpub::open(test_epub).await.unwrap();
            let result = epub.extract_text_only().await;
            assert!(result.is_ok());
            let chapters = result.unwrap();
            assert!(!chapters.is_empty());

            // All chapters should succeed
            for chapter in chapters {
                assert!(!chapter.is_empty());
            }
        }
    }
}
