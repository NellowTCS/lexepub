#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use lexepub::core::extractor::EpubExtractor;
    use std::path::Path;

    #[tokio::test]
    async fn test_extractor_creation() {
        let extractor = EpubExtractor::open(Path::new("test.epub").to_path_buf()).await;
        assert!(extractor.is_ok());

        let data = Bytes::from("test data");
        let extractor = EpubExtractor::from_bytes(data).await;
        assert!(extractor.is_ok());
    }

    #[tokio::test]
    async fn test_read_missing_file() {
        let extractor = EpubExtractor::open(Path::new("nonexistent.epub").to_path_buf())
            .await
            .unwrap();
        let result = extractor.read_file("missing.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_file_from_test_epub() {
        let test_epub = Path::new("examples/epubs/test-book.epub");
        if test_epub.exists() {
            let extractor = EpubExtractor::open(test_epub.to_path_buf()).await.unwrap();

            // Try to read container.xml
            let result = extractor.read_file("META-INF/container.xml").await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert!(!data.is_empty());
            assert!(String::from_utf8(data).unwrap().contains("container"));
        }
    }
}
