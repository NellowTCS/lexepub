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

    // Fail if the process RSS grows by more than 3 MB during extraction
    let threshold_kb = 3 * 1024; // 3 MB
    assert!(
        delta_kb <= threshold_kb,
        "Memory regression: delta={} KB > {} KB",
        delta_kb,
        threshold_kb
    );
}
