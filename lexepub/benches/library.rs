use criterion::{criterion_group, BatchSize, Criterion};
use std::path::Path;

fn read_examples_testbook_bytes() -> bytes::Bytes {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("examples/epubs/test-book.epub");
    let data = std::fs::read(&path).unwrap();
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
        b.iter_batched(
            || {
                let cursor = std::io::Cursor::new(buf.clone());
                let allow = futures::io::AllowStdIo::new(cursor);
                futures::io::BufReader::new(allow)
            },
            |reader| {
                futures::executor::block_on(lexepub::epub::LexEpub::from_reader(reader)).unwrap()
            },
            BatchSize::SmallInput,
        )
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

    group.bench_function("total_word_count", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| futures::executor::block_on(epub.total_word_count()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("total_char_count", |b| {
        b.iter_batched(
            || {
                futures::executor::block_on(lexepub::epub::LexEpub::from_bytes(bytes.clone()))
                    .unwrap()
            },
            |mut epub| futures::executor::block_on(epub.total_char_count()).unwrap(),
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
