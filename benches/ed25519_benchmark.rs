use criterion::{criterion_group, criterion_main, Criterion};
use ed25519::field::Field25519Element;

fn bench_inverse(c: &mut Criterion) {
    let mut items = [0; 32];
    // force last byte to be less than 128
    // so that the MSB is 0. This is because
    // p = 2^255-19. we only allow numbers
    // in [0,2^255] (see unpack docs)
    items[31] = 0x2;
    let packed = Field25519Element::new(items);
    let mut unpacked = packed.unpack();

    c.bench_function("inverse", |b| {
        b.iter(|| {
            unpacked.inverse();
        })
    });
}

fn bench_add(c: &mut Criterion) {
    let mut a = [0; 32];
    a[31] = 0x2;
    let packed_a = Field25519Element::new(a);

    let mut b = [0; 32];
    b[31] = 0x2;
    let packed_b = Field25519Element::new(b);

    let mut unpacked_a = packed_a.unpack();
    let unpacked_b = packed_b.unpack();

    c.bench_function("add", |b| {
        b.iter(|| {
            unpacked_a.add(&unpacked_b);
        })
    });
}

fn bench_mul(c: &mut Criterion) {
    let mut a = [0; 32];
    a[31] = 0x2;
    let packed_a = Field25519Element::new(a);

    let mut b = [0; 32];
    b[31] = 0x2;
    let packed_b = Field25519Element::new(b);

    let mut unpacked_a = packed_a.unpack();
    let unpacked_b = packed_b.unpack();

    c.bench_function("mul", |b| {
        b.iter(|| {
            unpacked_a.mul(&unpacked_b);
        })
    });
}

fn bench_sub(c: &mut Criterion) {
    let mut a = [0; 32];
    a[31] = 0x2;
    let packed_a = Field25519Element::new(a);

    let mut b = [0; 32];
    b[31] = 0x2;
    let packed_b = Field25519Element::new(b);

    let mut unpacked_a = packed_a.unpack();
    let unpacked_b = packed_b.unpack();

    c.bench_function("sub", |b| {
        b.iter(|| {
            unpacked_a.sub(&unpacked_b);
        })
    });
}

fn bench_unpack(c: &mut Criterion) {
    let mut items = [0; 32];
    // force last byte to be less than 128
    // so that the MSB is 0. This is because
    // p = 2^255-19. we only allow numbers
    // in [0,2^255] (see unpack docs)
    items[31] = 0x2;
    let packed = Field25519Element::new(items);

    c.bench_function("unpack", |b| b.iter(|| packed.unpack()));
}

fn bench_pack(c: &mut Criterion) {
    let mut items = [0; 32];
    // force last byte to be less than 128
    // so that the MSB is 0. This is because
    // p = 2^255-19. we only allow numbers
    // in [0,2^255] (see unpack docs)
    items[31] = 0x2;
    let packed = Field25519Element::new(items);
    let mut unpacked = packed.unpack();

    c.bench_function("pack", |b| b.iter(|| unpacked.pack()));
}

criterion_group!(
    benches,
    bench_inverse,
    bench_add,
    bench_mul,
    bench_sub,
    bench_unpack,
    bench_pack
);
criterion_main!(benches);
