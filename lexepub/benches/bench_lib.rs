use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// Use jemalloc for precise heap statistics inside the bench binary.
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use jemalloc_ctl::{epoch, stats};
use std::path::Path;

fn read_examples_testbook_bytes() -> bytes::Bytes {
    let data = std::fs::read(Path::new("examples/epubs/test-book.epub")).unwrap();
    bytes::Bytes::from(data)
}

fn bench_from_bytes(c: &mut Criterion) {
    let bytes = read_examples_testbook_bytes();
    c.bench_function("LexEpub::from_bytes", |b| {
        b.iter(|| {
            epoch::advance().ok();
            let before = stats::allocated::read().unwrap_or(0);
            let mut epub =
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap();
            let _ = futures::executor::block_on(epub.extract_text_only());
            epoch::advance().ok();
            let after = stats::allocated::read().unwrap_or(0);
            println!("allocated delta (bytes): {}", after.saturating_sub(before));
        })
    });
}

fn bench_from_reader(c: &mut Criterion) {
    let buf = std::fs::read(Path::new("examples/epubs/test-book.epub")).unwrap();
    c.bench_function("LexEpub::from_reader(sync BufReader)", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(buf.clone());
            let allow = futures::io::AllowStdIo::new(cursor);
            let reader = futures::io::BufReader::new(allow);
            epoch::advance().ok();
            let before = stats::allocated::read().unwrap_or(0);
            let mut epub =
                futures::executor::block_on(lexepub::epub::LexEpub::from_reader(reader)).unwrap();
            let _ = futures::executor::block_on(epub.extract_text_only());
            epoch::advance().ok();
            let after = stats::allocated::read().unwrap_or(0);
            println!("allocated delta (bytes): {}", after.saturating_sub(before));
        })
    });
}

fn bench_extract_text_only(c: &mut Criterion) {
    let bytes = read_examples_testbook_bytes();
    let mut group = c.benchmark_group("extract_text_only");
    for size in [1usize].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut epub =
                    futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                        .unwrap();
                epoch::advance().ok();
                let before = stats::allocated::read().unwrap_or(0);
                let _ = futures::executor::block_on(epub.extract_text_only()).unwrap();
                epoch::advance().ok();
                let after = stats::allocated::read().unwrap_or(0);
                println!("allocated delta (bytes): {}", after.saturating_sub(before));
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_from_bytes,
    bench_from_reader,
    bench_extract_text_only
);
criterion_main!(benches);
