use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::path::Path;

fn read_examples_testbook_bytes() -> bytes::Bytes {
    let data = std::fs::read(Path::new("examples/epubs/test-book.epub")).unwrap();
    bytes::Bytes::from(data)
}

fn bench_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("Loading");
    let bytes = read_examples_testbook_bytes();
    let path = Path::new("examples/epubs/test-book.epub");

    group.bench_function("from_bytes", |b| {
        b.iter(|| {
            let _ = futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                .unwrap();
        })
    });

    group.bench_function("open_path", |b| {
        b.iter(|| {
            let _ = futures::executor::block_on(lexepub::epub::LexEpub::open(path)).unwrap();
        })
    });

    let buf = std::fs::read(path).unwrap();
    group.bench_function("from_reader(sync BufReader)", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(buf.clone());
            let allow = futures::io::AllowStdIo::new(cursor);
            let reader = futures::io::BufReader::new(allow);
            let _ =
                futures::executor::block_on(lexepub::epub::LexEpub::from_reader(reader)).unwrap();
        })
    });

    group.finish();
}

fn bench_metadata(c: &mut Criterion) {
    let mut group = c.benchmark_group("Metadata");
    let bytes = read_examples_testbook_bytes();

    group.bench_function("get_metadata", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| futures::executor::block_on(epub.get_metadata()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("get_toc", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| futures::executor::block_on(epub.get_toc()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("Extraction");
    let bytes = read_examples_testbook_bytes();

    group.bench_function("extract_text_only", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| futures::executor::block_on(epub.extract_text_only()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("extract_ast", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| futures::executor::block_on(epub.extract_ast()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("Analysis");
    let bytes = read_examples_testbook_bytes();

    group.bench_function("total_word_char_count", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| {
                // word_count triggers extraction and caches both counts.
                // char_count returns from cache immediately: one extraction total.
                let w = futures::executor::block_on(epub.total_word_count()).unwrap();
                let c = futures::executor::block_on(epub.total_char_count()).unwrap();
                (w, c)
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark the cached path explicitly so we can see the speedup
    group.bench_function("total_word_char_count_cached", |b| {
        b.iter_batched(
            || {
                let mut epub =
                    futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                        .unwrap();
                // Pre-warm the cache
                futures::executor::block_on(epub.total_word_count()).unwrap();
                epub
            },
            |mut epub| {
                // Both should be instant cache hits
                let w = futures::executor::block_on(epub.total_word_count()).unwrap();
                let c = futures::executor::block_on(epub.total_char_count()).unwrap();
                (w, c)
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_loading,
    bench_metadata,
    bench_extraction,
    bench_analysis
);
criterion_main!(benches);
