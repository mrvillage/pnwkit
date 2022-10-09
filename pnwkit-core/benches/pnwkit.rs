use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let a = black_box(1);
    c.bench_function("temp", |b| b.iter(|| a));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
