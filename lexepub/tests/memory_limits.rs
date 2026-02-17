use lexepub::epub::LexEpub;
use std::path::Path;

fn read_rss_kb() -> Option<usize> {
    // Parse /proc/self/status VmRSS (Linux only)
    if let Ok(s) = std::fs::read_to_string("/proc/self/status") {
        for line in s.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return Some(kb);
                    }
                }
            }
        }
    }
    None
}

#[test]
fn library_memory_within_threshold() {
    // Use a reasonably large test EPUB to exercise streaming paths
    let test_epub =
        Path::new("examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub");
    if !test_epub.exists() {
        // skip on CI if example not present
        return;
    }

    let before = read_rss_kb().unwrap_or(0);

    let mut epub = futures::executor::block_on(LexEpub::open(test_epub)).unwrap();
    let _ = futures::executor::block_on(epub.extract_text_only()).unwrap();

    let after = read_rss_kb().unwrap_or(0);
    let delta_kb = after.saturating_sub(before);
    eprintln!(
        "RSS before={} KB, after={} KB, delta={} KB",
        before, after, delta_kb
    );

    // Fail if the process RSS grows by more than 2 MB during extraction
    let threshold_kb = 2 * 1024;
    assert!(
        delta_kb <= threshold_kb,
        "Memory regression: delta={} KB > {} KB",
        delta_kb,
        threshold_kb
    );
}

#[cfg(feature = "lowmem")]
#[test]
fn lowmem_library_memory_within_threshold() {
    let test_epub = Path::new("examples/epubs/test-book.epub");
    if !test_epub.exists() {
        return;
    }

    // Measure streaming RSS delta
    let before_stream = read_rss_kb().unwrap_or(0);
    let mut epub_stream = futures::executor::block_on(LexEpub::open(test_epub)).unwrap();
    let mut stream = futures::executor::block_on(epub_stream.extract_chapters_stream()).unwrap();

    futures::executor::block_on(async {
        use futures::StreamExt;
        if let Some(ch) = stream.next().await {
            let _ = ch.unwrap();
        }
    });

    let after_stream = read_rss_kb().unwrap_or(0);
    let streaming_delta_kb = after_stream.saturating_sub(before_stream);
    eprintln!(
        "LOWMEM streaming RSS before={} KB, after={} KB, delta={} KB",
        before_stream, after_stream, streaming_delta_kb
    );

    // Measure eager (non-streaming) RSS delta for comparison
    let before_eager = read_rss_kb().unwrap_or(0);
    let mut epub_eager = futures::executor::block_on(LexEpub::open(test_epub)).unwrap();
    let _ = futures::executor::block_on(epub_eager.extract_text_only()).unwrap();
    let after_eager = read_rss_kb().unwrap_or(0);
    let eager_delta_kb = after_eager.saturating_sub(before_eager);
    eprintln!(
        "LOWMEM eager RSS before={} KB, after={} KB, delta={} KB",
        before_eager, after_eager, eager_delta_kb
    );

    // Ensure streaming isn't absurdly large
    let absolute_threshold_kb = 1024;
    assert!(
        streaming_delta_kb <= absolute_threshold_kb,
        "Lowmem memory regression: streaming delta={} KB > {} KB",
        streaming_delta_kb,
        absolute_threshold_kb
    );

    // log eager delta for diagnostics
    eprintln!("LOWMEM eager delta = {} KB", eager_delta_kb);
}
