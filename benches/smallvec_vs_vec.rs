//! Throughput comparison of the storage type behind `Decoder`/`Encoder`.
//!
//! Three variants, differing only where noted:
//! - `lib`       — the real library (`SmallVec`), called across a crate boundary
//! - `smallvec`  — a same-crate mirror using `SmallVec` (isolates inlining)
//! - `vec`       — a same-crate mirror using plain `Vec`
//!
//! Each is measured in two access patterns:
//! - `reuse`: one instance created up-front, used every iteration (steady state).
//! - `cold`:  a fresh instance per iteration (first-message cost included).
//!
//! Run: `cargo bench --bench smallvec_vs_vec`

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::encoder::Encoder;

#[path = "common/mod.rs"]
mod common;

use common::{SmallVecDecoder, SmallVecEncoder, VecDecoder, VecEncoder, fixtures};

/// Sum of all parsed tags via the library decoder (forces offsets to be read).
fn decode_sum_lib(dec: &mut Decoder, buf: &[u8]) -> u32 {
    let msg = dec.decode(buf).unwrap();
    msg.fields().fold(0u32, |a, f| a.wrapping_add(f.tag))
}

fn bench_decode(c: &mut Criterion) {
    let mut g = c.benchmark_group("decode_smallvec_vs_vec");

    for (name, buf) in fixtures() {
        g.throughput(Throughput::Bytes(buf.len() as u64));

        // ---- reuse (steady state) ----
        g.bench_with_input(BenchmarkId::new("lib_reuse", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            b.iter(|| black_box(decode_sum_lib(&mut dec, black_box(buf))));
        });
        g.bench_with_input(BenchmarkId::new("smallvec_reuse", name), buf, |b, buf| {
            let mut dec = SmallVecDecoder::new();
            b.iter(|| black_box(dec.decode(black_box(buf)).unwrap()));
        });
        g.bench_with_input(BenchmarkId::new("vec_reuse", name), buf, |b, buf| {
            let mut dec = VecDecoder::new();
            b.iter(|| black_box(dec.decode(black_box(buf)).unwrap()));
        });

        // ---- cold (fresh instance per iteration) ----
        g.bench_with_input(BenchmarkId::new("lib_cold", name), buf, |b, buf| {
            b.iter(|| {
                let mut dec = Decoder::new();
                black_box(decode_sum_lib(&mut dec, black_box(buf)))
            });
        });
        g.bench_with_input(BenchmarkId::new("smallvec_cold", name), buf, |b, buf| {
            b.iter(|| {
                let mut dec = SmallVecDecoder::new();
                black_box(dec.decode(black_box(buf)).unwrap())
            });
        });
        g.bench_with_input(BenchmarkId::new("vec_cold", name), buf, |b, buf| {
            b.iter(|| {
                let mut dec = VecDecoder::new();
                black_box(dec.decode(black_box(buf)).unwrap())
            });
        });
    }

    g.finish();
}

fn bench_encode(c: &mut Criterion) {
    let mut g = c.benchmark_group("encode_smallvec_vs_vec");

    for (name, buf) in fixtures() {
        g.throughput(Throughput::Bytes(buf.len() as u64));

        // ---- reuse (steady state) ----
        g.bench_with_input(BenchmarkId::new("lib_reuse", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            let msg = dec.decode(buf).unwrap();
            let mut enc = Encoder::new();
            let mut out = Vec::with_capacity(8192);
            b.iter(|| {
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
        g.bench_with_input(BenchmarkId::new("smallvec_reuse", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            let msg = dec.decode(buf).unwrap();
            let mut enc = SmallVecEncoder::new();
            let mut out = Vec::with_capacity(8192);
            b.iter(|| {
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
        g.bench_with_input(BenchmarkId::new("vec_reuse", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            let msg = dec.decode(buf).unwrap();
            let mut enc = VecEncoder::new();
            let mut out = Vec::with_capacity(8192);
            b.iter(|| {
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });

        // ---- cold (fresh instance per iteration) ----
        g.bench_with_input(BenchmarkId::new("lib_cold", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            let msg = dec.decode(buf).unwrap();
            let mut out = Vec::with_capacity(8192);
            b.iter(|| {
                let mut enc = Encoder::new();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
        g.bench_with_input(BenchmarkId::new("smallvec_cold", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            let msg = dec.decode(buf).unwrap();
            let mut out = Vec::with_capacity(8192);
            b.iter(|| {
                let mut enc = SmallVecEncoder::new();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
        g.bench_with_input(BenchmarkId::new("vec_cold", name), buf, |b, buf| {
            let mut dec = Decoder::new();
            let msg = dec.decode(buf).unwrap();
            let mut out = Vec::with_capacity(8192);
            b.iter(|| {
                let mut enc = VecEncoder::new();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
    }

    g.finish();
}

criterion_group!(benches, bench_decode, bench_encode);
criterion_main!(benches);
