use futures::StreamExt;
use lexepub::epub::LexEpub;
use std::path::Path;
use std::time::Instant;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_content_handling() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if !test_epub.exists() {
                return; // Skip if test file doesn't exist
            }

            let mut epub = LexEpub::open(test_epub).await.unwrap();

            // Test text extraction performance
            let start = Instant::now();
            let chapters = epub.extract_text_only().await.unwrap();
            let duration = start.elapsed();

            println!("Text extraction took: {:?}", duration);
            assert!(
                duration.as_millis() < 5000,
                "Text extraction should complete within 5 seconds"
            );

            // Test AST extraction performance
            let start = Instant::now();
            let ast_chapters = epub.extract_ast().await.unwrap();
            let duration = start.elapsed();

            println!("AST extraction took: {:?}", duration);
            assert!(
                duration.as_millis() < 5000,
                "AST extraction should complete within 5 seconds"
            );

            // Verify content was extracted
            assert!(!chapters.is_empty());
            assert!(!ast_chapters.is_empty());
            assert_eq!(chapters.len(), ast_chapters.len());
        });
    }

    #[test]
    fn test_memory_efficiency() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if !test_epub.exists() {
                return;
            }

            let mut epub = LexEpub::open(test_epub).await.unwrap();

            // Test streaming extraction to avoid loading everything into memory
            // (I feel like i've said that comment like 10 times already haha)
            let start = Instant::now();
            let stream = epub.extract_chapters_stream().await.unwrap();
            let duration = start.elapsed();

            println!("Stream creation took: {:?}", duration);
            assert!(
                duration.as_millis() < 1000,
                "Stream creation should be fast"
            );

            // Consume the stream
            let start = Instant::now();
            let mut count = 0;
            let mut stream = stream;
            while let Some(chapter) = stream.next().await {
                let _ = chapter.unwrap();
                count += 1;
            }
            let duration = start.elapsed();

            println!(
                "Stream consumption took: {:?} for {} chapters",
                duration, count
            );
            assert!(count > 0, "Should have at least one chapter");
        });
    }

    #[test]
    fn test_concurrent_access() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if !test_epub.exists() {
                return;
            }

            // Test multiple concurrent operations on the same EPUB
            let epub_data = std::fs::read(test_epub).unwrap();
            let epub_bytes = bytes::Bytes::from(epub_data);

            let handles: Vec<_> = (0..3)
                .map(|_| {
                    let bytes = epub_bytes.clone();
                    std::thread::spawn(move || {
                        futures::executor::block_on(async move {
                            let mut epub = LexEpub::from_bytes(bytes).await.unwrap();
                            epub.extract_text_only().await.unwrap()
                        })
                    })
                })
                .collect();

            let start = Instant::now();
            let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
            let duration = start.elapsed();

            println!("Concurrent processing took: {:?}", duration);

            // All should succeed
            for chapters in results {
                assert!(!chapters.is_empty());
            }
        });
    }

    #[test]
    fn test_metadata_extraction_speed() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if !test_epub.exists() {
                return;
            }

            let mut epub = LexEpub::open(test_epub).await.unwrap();

            let start = Instant::now();
            let metadata = epub.get_metadata().await.unwrap();
            let duration = start.elapsed();

            println!("Metadata extraction took: {:?}", duration);
            assert!(
                duration.as_millis() < 1000,
                "Metadata extraction should be fast"
            );

            // Verify metadata is complete
            assert!(metadata.title.is_some() || !metadata.authors.is_empty());
        });
    }

    #[test]
    fn test_chapter_access_patterns() {
        futures::executor::block_on(async {
            let test_epub = Path::new("examples/epubs/test-book.epub");
            if !test_epub.exists() {
                return;
            }

            let mut epub = LexEpub::open(test_epub).await.unwrap();

            // Test random access to chapters
            let chapters = epub.extract_text_only().await.unwrap();
            let num_chapters = chapters.len();

            if num_chapters > 1 {
                let start = Instant::now();
                for i in 0..num_chapters {
                    let chapter = &chapters[i];
                    assert!(!chapter.is_empty());
                }
                let duration = start.elapsed();

                println!(
                    "Chapter access took: {:?} for {} chapters",
                    duration, num_chapters
                );
                assert!(duration.as_millis() < 1000, "Chapter access should be fast");
            }
        });
    }

    #[test]
    fn test_large_epub_handling() {
        futures::executor::block_on(async {
            // Test with the largest available test EPUB
            let test_files = [
                "examples/epubs/Accessibility-Tests-Extended-Descriptions-v1.1.1.epub",
                "examples/epubs/Fundamental-Accessibility-Tests-Basic-Functionality-v2.0.0.epub",
                "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
                "examples/epubs/test-book.epub",
            ];

            for test_file in &test_files {
                let path = Path::new(test_file);
                if !path.exists() {
                    continue;
                }

                let file_size = std::fs::metadata(path).unwrap().len();
                println!("Testing {} ({} bytes)", test_file, file_size);

                let start = Instant::now();
                let mut epub = LexEpub::open(path).await.unwrap();
                let metadata = epub.get_metadata().await.unwrap();
                let duration = start.elapsed();

                println!("Metadata extraction took: {:?}", duration);
                assert!(
                    duration.as_millis() < 2000,
                    "Metadata extraction should be reasonably fast"
                );

                // Only test content extraction for smaller files to avoid timeouts
                if file_size < 10_000_000 {
                    // 10MB limit
                    let start = Instant::now();
                    let chapters = epub.extract_text_only().await.unwrap();
                    let duration = start.elapsed();

                    println!(
                        "Content extraction took: {:?} for {} chapters",
                        duration,
                        chapters.len()
                    );
                    assert!(
                        duration.as_millis() < 10000,
                        "Content extraction should complete within 10 seconds"
                    );
                    assert!(!chapters.is_empty());
                }

                // Verify basic metadata
                assert!(metadata.title.is_some() || !metadata.authors.is_empty());
            }
        });
    }
}
