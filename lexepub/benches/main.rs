use criterion::criterion_main;

mod library;

criterion_main!(library::benches);
