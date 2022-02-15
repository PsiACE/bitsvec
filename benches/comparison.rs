#![feature(portable_simd)]
use core::simd::*;

use criterion::*;

fn benchmark_bitvector_simd(c: &mut Criterion) {
    let b1 = bitsvec::BitVec::ones(100_000);
    let b2 = bitsvec::BitVec::zeros(100_000);
    c.bench_function("bitsvec(this crate)", |b| {
        b.iter(|| {
            black_box(b1.and_cloned(&b2));
        })
    });
}

fn benchmark_bitvector_simd2(c: &mut Criterion) {
    c.bench_function("bitsvec(this crate) with creation", |b| {
        b.iter(|| {
            let b1 = bitsvec::BitVec::ones(100_000);
            let b2 = bitsvec::BitVec::zeros(100_000);
            black_box(b1.and(b2));
        })
    });
}

fn benchmark_bitvector_simd3(c: &mut Criterion) {
    c.bench_function("bitsvec(this crate) resize false", |b| {
        b.iter(|| {
            let mut b1 = bitsvec::BitVec::ones(100_000);
            black_box(b1.resize(200_000, false));
        })
    });
}

fn benchmark_bitvector_simd4(c: &mut Criterion) {
    c.bench_function("bitsvec(this crate) resize true", |b| {
        b.iter(|| {
            let mut b1 = bitsvec::BitVec::ones(100_000);
            black_box(b1.resize(200_000, true));
        })
    });
}

fn benchmark_bitvector_simd_u16x8(c: &mut Criterion) {
    let b1 = bitsvec::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
    let b2 = bitsvec::BitVecSimd::<[u16x8; 4], 8>::zeros(100_000);
    c.bench_function("bitsvec_u16x8(this crate)", |b| {
        b.iter(|| {
            black_box(b1.and_cloned(&b2));
        })
    });
}

fn benchmark_bitvector_simd2_u16x8(c: &mut Criterion) {
    c.bench_function("bitsvec_u16x8(this crate) with creation", |b| {
        b.iter(|| {
            let b1 = bitsvec::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
            let b2 = bitsvec::BitVecSimd::<[u16x8; 4], 8>::zeros(100_000);
            black_box(b1.and(b2));
        })
    });
}

fn benchmark_bitvector_simd3_u16x8(c: &mut Criterion) {
    c.bench_function("bitsvec_u16x8(this crate) resize false", |b| {
        b.iter(|| {
            let mut b1 = bitsvec::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
            black_box(b1.resize(200_000, false));
        })
    });
}

fn benchmark_bitvector_simd4_u16x8(c: &mut Criterion) {
    c.bench_function("bitsvec_u16x8(this crate) resize true", |b| {
        b.iter(|| {
            let mut b1 = bitsvec::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
            black_box(b1.resize(200_000, true));
        })
    });
}

fn benchmark_bitvector_bitvec_simd(c: &mut Criterion) {
    let b1 = bitvec_simd::BitVec::ones(100_000);
    let b2 = bitvec_simd::BitVec::zeros(100_000);
    c.bench_function("bitvec_simd 0.15.0", |b| {
        b.iter(|| {
            black_box(b1.and_cloned(&b2));
        })
    });
}

fn benchmark_bitvector_bitvec_simd2(c: &mut Criterion) {
    c.bench_function("bitvec_simd 0.15.0 with creation", |b| {
        b.iter(|| {
            let b1 = bitvec_simd::BitVec::ones(100_000);
            let b2 = bitvec_simd::BitVec::zeros(100_000);
            black_box(b1.and(b2));
        })
    });
}

fn benchmark_bitvector_bitvec_simd3(c: &mut Criterion) {
    c.bench_function("bitvec_simd 0.15.0 resize false", |b| {
        b.iter(|| {
            let mut b1 = bitvec_simd::BitVec::ones(100_000);
            black_box(b1.resize(200_000, false));
        })
    });
}

fn benchmark_bitvector_bitvec_simd4(c: &mut Criterion) {
    c.bench_function("bitvec_simd 0.15.0 resize true", |b| {
        b.iter(|| {
            let mut b1 = bitvec_simd::BitVec::ones(100_000);
            black_box(b1.resize(200_000, true));
        })
    });
}

fn benchmark_bitvector_bitvec_simd_u16x8(c: &mut Criterion) {
    use wide::*;

    let b1 = bitvec_simd::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
    let b2 = bitvec_simd::BitVecSimd::<[u16x8; 4], 8>::zeros(100_000);
    c.bench_function("bitvec_simd 0.15.0 u16x8", |b| {
        b.iter(|| {
            black_box(b1.and_cloned(&b2));
        })
    });
}

fn benchmark_bitvector_bitvec_simd2_u16x8(c: &mut Criterion) {
    use wide::*;

    c.bench_function("bitvec_simd 0.15.0 u16x8 with creation", |b| {
        b.iter(|| {
            let b1 = bitvec_simd::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
            let b2 = bitvec_simd::BitVecSimd::<[u16x8; 4], 8>::zeros(100_000);
            black_box(b1.and(b2));
        })
    });
}

fn benchmark_bitvector_bitvec_simd3_u16x8(c: &mut Criterion) {
    use wide::*;

    c.bench_function("bitvec_simd 0.15.0 u16x8 resize false", |b| {
        b.iter(|| {
            let mut b1 = bitvec_simd::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
            black_box(b1.resize(200_000, false));
        })
    });
}

fn benchmark_bitvector_bitvec_simd4_u16x8(c: &mut Criterion) {
    use wide::*;

    c.bench_function("bitvec_simd 0.15.0 u16x8 resize true", |b| {
        b.iter(|| {
            let mut b1 = bitvec_simd::BitVecSimd::<[u16x8; 4], 8>::ones(100_000);
            black_box(b1.resize(200_000, true));
        })
    });
}

fn benchmark_bitvector_bitvector_simd(c: &mut Criterion) {
    let b1 = bitvector_simd::BitVector::ones(100_000);
    let b2 = bitvector_simd::BitVector::zeros(100_000);
    c.bench_function("bitvector_simd 0.2.2", |b| {
        b.iter(|| {
            black_box(b1.and_cloned(&b2));
        })
    });
}

fn benchmark_bitvector_bitvector_simd2(c: &mut Criterion) {
    c.bench_function("bitvector_simd 0.2.2 with creation", |b| {
        b.iter(|| {
            let b1 = bitvector_simd::BitVector::ones(100_000);
            let b2 = bitvector_simd::BitVector::zeros(100_000);
            black_box(b1.and(b2));
        })
    });
}

fn benchmark_bitvector_bitvec(c: &mut Criterion) {
    let b1 = bit_vec::BitVec::from_elem(100_000, true);
    let b2 = bit_vec::BitVec::from_elem(100_000, false);
    c.bench_function("bit-vec 0.6", |b| {
        b.iter(|| {
            black_box(b1.clone().and(&b2));
        })
    });
}

fn benchmark_bitvector_bitvec2(c: &mut Criterion) {
    c.bench_function("bit-vec 0.6 with creation", |b| {
        b.iter(|| {
            let mut b1 = bit_vec::BitVec::from_elem(100_000, true);
            let b2 = bit_vec::BitVec::from_elem(100_000, false);
            black_box(b1.and(&b2));
        })
    });
}

fn benchmark_bitvector_bitvec_n(c: &mut Criterion) {
    let b1 = bitvec::bitvec![usize, bitvec::order::Msb0; 1; 100_000];
    let b2 = bitvec::bitvec![usize, bitvec::order::Msb0; 0; 100_000];
    c.bench_function("bitvec 1.0", |b| {
        b.iter(|| {
            black_box(b1.clone() & b2.clone());
        })
    });
}

fn benchmark_bitvector_bitvec_n2(c: &mut Criterion) {
    c.bench_function("bitvec 1.0 with creation", |b| {
        b.iter(|| {
            let b1 = bitvec::bitvec![usize, bitvec::order::Msb0; 1; 100_000];
            let b2 = bitvec::bitvec![usize, bitvec::order::Msb0; 0; 100_000];
            black_box(b1 & b2);
        })
    });
}

fn benchmark_bitvector_bitvec_n3(c: &mut Criterion) {
    c.bench_function("bitvec 1.0 resize false", |b| {
        b.iter(|| {
            let mut b1 = bitvec::bitvec![usize, bitvec::order::Msb0; 1; 100_000];
            black_box(b1.resize(200_000, false));
        })
    });
}

fn benchmark_bitvector_bitvec_n4(c: &mut Criterion) {
    c.bench_function("bitvec 1.0 resize true", |b| {
        b.iter(|| {
            let mut b1 = bitvec::bitvec![usize, bitvec::order::Msb0; 1; 100_000];
            black_box(b1.resize(200_000, true));
        })
    });
}

criterion_group!(
    normal_benches,
    benchmark_bitvector_simd,
    benchmark_bitvector_simd_u16x8,
    benchmark_bitvector_bitvec_simd,
    benchmark_bitvector_bitvec_simd_u16x8,
    benchmark_bitvector_bitvector_simd,
    benchmark_bitvector_bitvec,
    benchmark_bitvector_bitvec_n
);
criterion_group!(
    with_creation_benches,
    benchmark_bitvector_simd2,
    benchmark_bitvector_simd2_u16x8,
    benchmark_bitvector_bitvec_simd2,
    benchmark_bitvector_bitvec_simd2_u16x8,
    benchmark_bitvector_bitvector_simd2,
    benchmark_bitvector_bitvec2,
    benchmark_bitvector_bitvec_n2
);
criterion_group!(
    resize_false_benches,
    benchmark_bitvector_simd3,
    benchmark_bitvector_simd3_u16x8,
    benchmark_bitvector_bitvec_simd3,
    benchmark_bitvector_bitvec_simd3_u16x8,
    benchmark_bitvector_bitvec_n3
);
criterion_group!(
    resize_true_benches,
    benchmark_bitvector_simd4,
    benchmark_bitvector_simd4_u16x8,
    benchmark_bitvector_bitvec_simd4,
    benchmark_bitvector_bitvec_simd4_u16x8,
    benchmark_bitvector_bitvec_n4
);
criterion_main!(
    normal_benches,
    with_creation_benches,
    resize_false_benches,
    resize_true_benches
);
