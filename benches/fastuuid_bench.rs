#[macro_use]
extern crate criterion;
extern crate rand;

use rand::Rng;

use criterion::Criterion;
use fastuuid_rs::Generator;

fn criterion_benchmark(c: &mut Criterion) {
    let g = Generator::new();
    let seed = rand::thread_rng().gen::<[u8; 24]>();

    c.bench_function("next", |b| b.iter(|| g.next()));
    c.bench_function("hex128", |b| b.iter(|| g.hex128()));
    c.bench_function("encode", |b| b.iter(|| Generator::hex128_from_bytes(&seed)));
    c.bench_function("uuidCrate-next", |b| b.iter(|| uuid::Uuid::new_v4()));
    c.bench_function("uuidCrate-nextToStr", |b| {
        b.iter(|| format!("{}", uuid::Uuid::new_v4()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
