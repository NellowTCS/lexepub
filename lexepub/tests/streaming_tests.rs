use futures::StreamExt;
use lexepub::epub::extract_text_only;
use lexepub::prelude::*;
use std::path::Path;

#[test]
fn test_stream_matches_eager_extraction() {
    futures::executor::block_on(async {
        let test_files = [
            "examples/epubs/test-book.epub",
            "examples/epubs/Accessibility-Tests-Extended-Descriptions-v1.1.1.epub",
            "examples/epubs/Fundamental-Accessibility-Tests-Basic-Functionality-v2.0.0.epub",
            "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
        ];

        for f in &test_files {
            let path = Path::new(f);
            if !path.exists() {
                continue;
            }

            // eager extraction
            let eager = extract_text_only(path).await.unwrap();

            // streaming extraction
            let mut epub = LexEpub::open(path).await.unwrap();
            let mut stream = epub.extract_chapters_stream().await.unwrap();

            let mut streamed = Vec::new();
            while let Some(ch) = stream.next().await {
                let parsed = ch.unwrap();
                streamed.push(parsed.content);
            }

            assert_eq!(eager, streamed, "streamed != eager for {}", f);
        }
    });
}

#[test]
fn test_partial_stream_consumption_then_full_extract() {
    futures::executor::block_on(async {
        let path = Path::new(
            "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
        );
        if !path.exists() {
            return;
        }

        let mut epub = LexEpub::open(path).await.unwrap();
        let mut stream = epub.extract_chapters_stream().await.unwrap();

        // consume only the first chapter
        if let Some(first) = stream.next().await {
            let first = first.unwrap();
            assert!(!first.content.is_empty());
        }

        // full eager extraction should still succeed and contain at least one chapter
        let all = epub.extract_text_only().await.unwrap();
        assert!(!all.is_empty());
    });
}

#[test]
fn test_chapterstream_type_is_stream() {
    // compile-time trait check
    fn _assert_stream<S: futures::Stream<Item = lexepub::Result<lexepub::ParsedChapter>>>() {}
    _assert_stream::<lexepub::ChapterStream>();
}
