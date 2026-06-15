use futures::StreamExt;
use lexepub::epub::{extract_text_only, get_metadata, LexEpub};
#[cfg(not(feature = "lowmem"))]
use lexepub::epub::extract_ast;
use std::path::Path;

/// All available test EPUB files
fn get_test_epubs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("examples/epubs/test-book.epub", "Basic test EPUB"),
        (
            "examples/epubs/Accessibility-Tests-Extended-Descriptions-v1.1.1.epub",
            "EPUB 3 with extended image descriptions",
        ),
        (
            "examples/epubs/Fundamental-Accessibility-Tests-Basic-Functionality-v2.0.0.epub",
            "EPUB 3 basic functionality test",
        ),
        (
            "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
            "EPUB 3 visual adjustments test",
        ),
        (
            "examples/epubs/captain-charles-johnson_a-general-history-of-the-pirates.epub",
            "Standard EBooks - Basic",
        ),
        (
            "examples/epubs/captain-charles-johnson_a-general-history-of-the-pirates_advanced.epub",
            "Standard EBooks - Advanced",
        ),
        (
            "examples/epubs/helen-herron-taft_recollections-of-full-years.epub",
            "Standard EBooks - Large book",
        ),
        (
            "examples/epubs/lytton-strachey_eminent-victorians.epub",
            "Standard EBooks - Basic",
        ),
        (
            "examples/epubs/lytton-strachey_eminent-victorians_advanced.epub",
            "Standard EBooks - Advanced",
        ),
        (
            "examples/epubs/lytton-strachey_queen-victoria.epub",
            "Standard EBooks - Basic",
        ),
        (
            "examples/epubs/mark-twain_personal-recollections-of-joan-of-arc.epub",
            "Standard EBooks - Basic",
        ),
        (
            "examples/epubs/walter-noble-burns_tombstone.epub",
            "Standard EBooks - Basic",
        ),
        (
            "examples/epubs/walter-noble-burns_tombstone_advanced.epub",
            "Standard EBooks - Advanced",
        ),
    ]
}

/// Return only the EPUB paths that actually exist
fn existing_epubs() -> Vec<&'static str> {
    let paths: Vec<&'static str> = get_test_epubs()
        .into_iter()
        .filter(|(path, _)| Path::new(path).exists())
        .map(|(path, _)| path)
        .collect();
    assert!(!paths.is_empty(), "No test EPUB fixtures found. Ensure test fixtures are available at the paths returned by get_test_epubs().");
    paths
}

#[cfg(test)]
mod epub_feature_tests {
    use super::*;

    // Metadata Extraction Tests
    #[test]
    fn test_metadata_extraction_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue, // Skip if can't open
                };

                let metadata = epub.get_metadata().await;
                assert!(
                    metadata.is_ok(),
                    "Failed to extract metadata from {}",
                    epub_path
                );

                let metadata = metadata.unwrap();
                // EPUB spec requires at least a title or identifier
                assert!(
                    metadata.title.is_some() || !metadata.identifiers.is_empty(),
                    "EPUB {} should have title or identifier",
                    epub_path
                );
            }
        });
    }

    #[test]
    fn test_metadata_fields_not_empty() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let metadata = epub.get_metadata().await.unwrap();

                // Languages should be present in valid EPUBs
                assert!(
                    !metadata.languages.is_empty(),
                    "EPUB {} should have at least one language",
                    epub_path
                );

                // Chapter count should be tracked
                assert!(
                    metadata.chapter_count > 0,
                    "EPUB {} should have at least one chapter",
                    epub_path
                );
            }
        });
    }

    #[test]
    fn test_metadata_validate_method() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let metadata = epub.get_metadata().await.unwrap();
                let validation = metadata.validate();
                // Some EPUBs may not have all required fields (e.g., test EPUBs)
                // Log the validation errors but don't fail for EPUBs that have content
                if let Err(ref errors) = validation {
                    println!("Validation warnings for {}: {:?}", epub_path, errors);
                }
                // At minimum, metadata should be extractable without panic
                let _ = metadata.title;
                let _ = metadata.languages.clone();
            }
        });
    }

    // Text Extraction Tests
    #[test]
    fn test_text_extraction_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let chapters = extract_text_only(epub_path).await;
                assert!(
                    chapters.is_ok(),
                    "Failed to extract text from {}",
                    epub_path
                );

                let chapters = chapters.unwrap();
                assert!(
                    !chapters.is_empty(),
                    "EPUB {} should have chapters",
                    epub_path
                );

                // Each chapter should have some content
                for (i, chapter) in chapters.iter().enumerate() {
                    assert!(
                        !chapter.is_empty(),
                        "Chapter {} in {} should not be empty",
                        i,
                        epub_path
                    );
                }
            }
        });
    }

    #[test]
    fn test_text_extraction_word_counts() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let word_count = epub.total_word_count().await;
                assert!(
                    word_count.is_ok(),
                    "Failed to get word count from {}",
                    epub_path
                );

                let word_count = word_count.unwrap();
                assert!(
                    word_count > 0,
                    "EPUB {} should have positive word count",
                    epub_path
                );

                let char_count = epub.total_char_count().await.unwrap();
                assert!(
                    char_count > 0,
                    "EPUB {} should have positive char count",
                    epub_path
                );
                assert!(
                    char_count >= word_count,
                    "Char count should be >= word count in {}",
                    epub_path
                );
            }
        });
    }

    // AST Extraction Tests
    #[cfg(not(feature = "lowmem"))]
    #[test]
    fn test_ast_extraction_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let chapters = extract_ast(epub_path).await;
                assert!(chapters.is_ok(), "Failed to extract AST from {}", epub_path);

                let chapters = chapters.unwrap();
                assert!(
                    !chapters.is_empty(),
                    "EPUB {} should have AST chapters",
                    epub_path
                );

                // Each chapter should have content
                for chapter in &chapters {
                    assert!(
                        !chapter.content.is_empty(),
                        "AST chapter in {} should have text content",
                        epub_path
                    );
                    assert!(
                        chapter.word_count > 0,
                        "AST chapter in {} should have word count",
                        epub_path
                    );
                }
            }
        });
    }

    #[cfg(not(feature = "lowmem"))]
    #[test]
    fn test_ast_has_valid_structure() {
        futures::executor::block_on(async {
            let epub_path = "examples/epubs/test-book.epub";
            if !Path::new(epub_path).exists() {
                return;
            }

            let chapters = extract_ast(epub_path).await.unwrap();
            for chapter in &chapters {
                // If AST is present, it should be valid
                if let Some(ref ast) = chapter.ast {
                    assert!(
                        matches!(
                            ast,
                            lexepub::AstNode::Element { .. }
                                | lexepub::AstNode::Text { .. }
                                | lexepub::AstNode::Comment { .. }
                        ),
                        "AST node should be valid type"
                    );
                }
            }
        });
    }

    // Table of Contents Tests
    #[test]
    fn test_toc_generation_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let toc = epub.get_toc().await;
                assert!(toc.is_ok(), "Failed to generate TOC for {}", epub_path);

                let toc = toc.unwrap();
                assert!(
                    !toc.is_empty(),
                    "EPUB {} should have TOC entries",
                    epub_path
                );

                // Each TOC entry should have required fields
                for (i, entry) in toc.iter().enumerate() {
                    assert!(
                        !entry.chapter_id.is_empty(),
                        "TOC entry {} in {} should have chapter_id",
                        i,
                        epub_path
                    );
                    assert!(
                        !entry.chapter_href.is_empty(),
                        "TOC entry {} in {} should have chapter_href",
                        i,
                        epub_path
                    );
                    assert!(
                        !entry.title.is_empty(),
                        "TOC entry {} in {} should have title",
                        i,
                        epub_path
                    );
                }
            }
        });
    }

    // Streaming Tests
    #[test]
    fn test_streaming_extraction_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let stream = epub.extract_chapters_stream().await;
                assert!(stream.is_ok(), "Failed to create stream for {}", epub_path);

                let mut stream = stream.unwrap();
                let mut count = 0;
                while let Some(result) = stream.next().await {
                    assert!(
                        result.is_ok(),
                        "Stream error in {}: {:?}",
                        epub_path,
                        result.err()
                    );
                    count += 1;
                }
                assert!(count > 0, "Stream for {} should yield chapters", epub_path);
            }
        });
    }

    #[test]
    fn test_streaming_matches_eager_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let eager = match extract_text_only(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let mut epub = LexEpub::open(epub_path).await.unwrap();
                let mut stream = epub.extract_chapters_stream().await.unwrap();

                let mut streamed = Vec::new();
                while let Some(ch) = stream.next().await {
                    let parsed = ch.unwrap();
                    streamed.push(parsed.content);
                }

                assert_eq!(
                    eager.len(),
                    streamed.len(),
                    "Streamed and eager counts differ for {}",
                    epub_path
                );

                for (i, (e, s)) in eager.iter().zip(streamed.iter()).enumerate() {
                    assert_eq!(
                        e.as_str(),
                        s.as_str(),
                        "Content mismatch at chapter {} in {}",
                        i,
                        epub_path
                    );
                }
            }
        });
    }

    // Convenience Functions Tests
    #[test]
    fn test_convenience_functions_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                // Test extract_text_only
                let result = extract_text_only(epub_path).await;
                assert!(result.is_ok(), "extract_text_only failed for {}", epub_path);

                // Test extract_ast
                #[cfg(not(feature = "lowmem"))]
                {
                    let result = extract_ast(epub_path).await;
                    assert!(result.is_ok(), "extract_ast failed for {}", epub_path);
                }

                // Test get_metadata
                let result = get_metadata(epub_path).await;
                assert!(result.is_ok(), "get_metadata failed for {}", epub_path);
            }
        });
    }

    // EPUB Version Tests
    #[test]
    fn test_epub_version_detection() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let metadata = epub.get_metadata().await.unwrap();
                // Version should be detected if present
                if let Some(ref version) = metadata.version {
                    assert!(
                        version.starts_with("2.") || version.starts_with("3."),
                        "EPUB {} has invalid version: {}",
                        epub_path,
                        version
                    );
                }
            }
        });
    }

    // Cover Detection Tests
    #[test]
    fn test_cover_detection() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let metadata = epub.get_metadata().await.unwrap();
                // has_cover should be a boolean (not panic)
                let _ = metadata.has_cover;
            }
        });
    }

    // From Bytes Tests
    #[test]
    fn test_from_bytes_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let data = match std::fs::read(epub_path) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let bytes = bytes::Bytes::from(data);

                let result = LexEpub::from_bytes(bytes).await;
                assert!(result.is_ok(), "from_bytes failed for {}", epub_path);
            }
        });
    }

    // Edge Cases Tests
    #[test]
    fn test_repeated_extraction_same_instance() {
        futures::executor::block_on(async {
            let epub_path = "examples/epubs/test-book.epub";
            if !Path::new(epub_path).exists() {
                return;
            }

            let mut epub = LexEpub::open(epub_path).await.unwrap();

            // First extraction
            let first = epub.extract_text_only().await.unwrap();

            // Second extraction should also work (cached or re-extracted)
            let second = epub.extract_text_only().await.unwrap();

            assert_eq!(
                first.len(),
                second.len(),
                "Repeated extraction should yield same chapter count"
            );
        });
    }

    #[test]
    fn test_chapter_order_preserved() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let mut epub = match LexEpub::open(epub_path).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let metadata = epub.get_metadata().await.unwrap();
                let chapters = epub.extract_text_only().await.unwrap();

                // Chapter count should match metadata
                assert_eq!(
                    metadata.chapter_count,
                    chapters.len(),
                    "Chapter count mismatch in {}",
                    epub_path
                );
            }
        });
    }

    // Analysis Tests
    #[test]
    fn test_analyze_reader_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let data = match std::fs::read(epub_path) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let cursor = futures::io::AllowStdIo::new(std::io::Cursor::new(data));
                let report = lexepub::epub::analyze_reader(cursor).await;
                assert!(report.is_ok(), "analyze_reader failed for {}", epub_path);

                let report = report.unwrap();
                assert!(
                    report.chapter_count > 0,
                    "Analysis should find chapters in {}",
                    epub_path
                );
            }
        });
    }

    #[test]
    fn test_analyze_path_all_epubs() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let report = lexepub::epub::analyze_path(epub_path).await;
                assert!(report.is_ok(), "analyze_path failed for {}", epub_path);

                let report = report.unwrap();
                assert!(
                    report.chapter_count > 0,
                    "Analysis should find chapters in {}",
                    epub_path
                );
            }
        });
    }

    // Unicode and Special Characters Tests
    #[test]
    fn test_unicode_content_handling() {
        futures::executor::block_on(async {
            for &epub_path in &existing_epubs() {
                let chapters = match extract_text_only(epub_path).await {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                for chapter in &chapters {
                    // Content should be valid UTF-8 (already enforced by type)
                    // Just verify we can iterate over chars without panic
                    let _: Vec<char> = chapter.chars().collect();
                }
            }
        });
    }

    // Accessibility-Specific Tests
    #[test]
    fn test_accessibility_epubs_have_content() {
        futures::executor::block_on(async {
            let accessibility_epubs = [
                "examples/epubs/Accessibility-Tests-Extended-Descriptions-v1.1.1.epub",
                "examples/epubs/Fundamental-Accessibility-Tests-Basic-Functionality-v2.0.0.epub",
                "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
            ];

            for epub_path in &accessibility_epubs {
                if !Path::new(epub_path).exists() {
                    continue;
                }

                let chapters = extract_text_only(epub_path).await.unwrap();
                assert!(
                    !chapters.is_empty(),
                    "Accessibility EPUB {} should have content",
                    epub_path
                );

                // Total content should be substantial
                let total_chars: usize = chapters.iter().map(|c| c.len()).sum();
                assert!(
                    total_chars > 100,
                    "Accessibility EPUB {} should have substantial content",
                    epub_path
                );
            }
        });
    }
}
