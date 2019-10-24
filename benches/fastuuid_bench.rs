#[macro_use]
extern crate criterion;
extern crate rand;

use criterion::Criterion;
use fastuuid::Generator;
use uuid::Uuid;

fn criterion_benchmark(c: &mut Criterion) {
    let g = Generator::new();

    c.bench_function("next", |b| b.iter(|| g.next()));
    c.bench_function("hex128_as_str", |b| {
        b.iter(|| {
            let mut buffer: [u8; 36] = [0; 36];
            let _ = g.hex128_as_str(&mut buffer).unwrap();
        })
    });
    c.bench_function("hex128_as_str_unchecked", |b| {
        b.iter(|| {
            let mut buffer: [u8; 36] = [0; 36];
            unsafe {
                let _ = g.hex128_as_str_unchecked(&mut buffer);
            }
        })
    });
    c.bench_function("hex128_as_string", |b| {
        b.iter(|| g.hex128_as_string().unwrap())
    });

    c.bench_function("uuid_uuidV4", |b| {
        b.iter(|| {
            let mut buffer: [u8; 36] = [0; 36];
            let _my_uuid = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
        })
    });

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
