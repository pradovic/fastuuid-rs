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
}




criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);