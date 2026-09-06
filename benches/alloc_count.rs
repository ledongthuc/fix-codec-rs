//! Allocation-count comparison: library (`SmallVec`) vs. `Vec` mirrors.
//!
//! This uses a counting global allocator to report the *actual* number of heap
//! allocations per message — the metric `SmallVec` is supposed to improve.
//! Run with: `cargo bench --bench alloc_count`

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "common/mod.rs"]
mod common;

use common::{VecDecoder, VecEncoder, fixtures};
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::encoder::Encoder;

static ALLOCS: AtomicU64 = AtomicU64::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static A: CountingAlloc = CountingAlloc;

/// Run `f` and return how many heap allocations it performed.
fn allocs(mut f: impl FnMut()) -> u64 {
    let before = ALLOCS.load(Ordering::Relaxed);
    f();
    ALLOCS.load(Ordering::Relaxed) - before
}

/// Map a closure over the three fixtures, producing `[small, typical, large]`.
fn per_fixture(mut f: impl FnMut(&[u8]) -> u64) -> [u64; 3] {
    let mut out = [0u64; 3];
    for (i, (_, buf)) in fixtures().into_iter().enumerate() {
        out[i] = f(buf);
    }
    out
}

const ITERS: u64 = 100_000;

fn decode_cold_smallvec() -> [u64; 3] {
    per_fixture(|buf| {
        allocs(|| {
            for _ in 0..ITERS {
                let mut dec = Decoder::new();
                black_box(dec.decode(buf).unwrap().len());
            }
        }) / ITERS
    })
}

fn decode_cold_vec() -> [u64; 3] {
    per_fixture(|buf| {
        allocs(|| {
            for _ in 0..ITERS {
                let mut dec = VecDecoder::new();
                black_box(dec.decode(buf).unwrap());
            }
        }) / ITERS
    })
}

fn decode_reuse_smallvec() -> [u64; 3] {
    per_fixture(|buf| {
        let mut dec = Decoder::new();
        dec.decode(buf).unwrap(); // warm-up: absorb the one-time allocation
        allocs(|| {
            for _ in 0..ITERS {
                black_box(dec.decode(buf).unwrap().len());
            }
        }) / ITERS
    })
}

fn decode_reuse_vec() -> [u64; 3] {
    per_fixture(|buf| {
        let mut dec = VecDecoder::new();
        dec.decode(buf).unwrap(); // warm-up
        allocs(|| {
            for _ in 0..ITERS {
                black_box(dec.decode(buf).unwrap());
            }
        }) / ITERS
    })
}

fn encode_cold_smallvec() -> [u64; 3] {
    per_fixture(|buf| {
        let mut dec = Decoder::new();
        let msg = dec.decode(buf).unwrap();
        let mut out = Vec::with_capacity(8192);
        allocs(|| {
            for _ in 0..ITERS {
                let mut enc = Encoder::new();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len());
            }
        }) / ITERS
    })
}

fn encode_cold_vec() -> [u64; 3] {
    per_fixture(|buf| {
        let mut dec = Decoder::new();
        let msg = dec.decode(buf).unwrap();
        let mut out = Vec::with_capacity(8192);
        allocs(|| {
            for _ in 0..ITERS {
                let mut enc = VecEncoder::new();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len());
            }
        }) / ITERS
    })
}

fn encode_reuse_smallvec() -> [u64; 3] {
    per_fixture(|buf| {
        let mut dec = Decoder::new();
        let msg = dec.decode(buf).unwrap();
        let mut out = Vec::with_capacity(8192);
        let mut enc = Encoder::new();
        enc.encode(&msg, &mut out).unwrap(); // warm-up
        allocs(|| {
            for _ in 0..ITERS {
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len());
            }
        }) / ITERS
    })
}

fn encode_reuse_vec() -> [u64; 3] {
    per_fixture(|buf| {
        let mut dec = Decoder::new();
        let msg = dec.decode(buf).unwrap();
        let mut out = Vec::with_capacity(8192);
        let mut enc = VecEncoder::new();
        enc.encode(&msg, &mut out).unwrap(); // warm-up
        allocs(|| {
            for _ in 0..ITERS {
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len());
            }
        }) / ITERS
    })
}

fn print_row(label: &str, row: [u64; 3]) {
    println!("{:<22} {:>8} {:>10} {:>10}", label, row[0], row[1], row[2]);
}

fn main() {
    // Sanity check: the Vec mirrors must match the library byte-for-byte.
    for (name, buf) in fixtures() {
        let mut dec = Decoder::new();
        let msg = dec.decode(buf).unwrap();

        let mut lib_enc = Encoder::new();
        let mut lib_out = Vec::with_capacity(8192);
        lib_enc.encode(&msg, &mut lib_out).unwrap();

        let mut vec_enc = VecEncoder::new();
        let mut vec_out = Vec::with_capacity(8192);
        vec_enc.encode(&msg, &mut vec_out).unwrap();

        assert_eq!(lib_out, vec_out, "encoder output mismatch for {name}");

        let mut vec_dec = VecDecoder::new();
        let sum = vec_dec.decode(buf).unwrap();
        let lib_sum = msg.fields().fold(0u32, |a, f| a.wrapping_add(f.tag));
        assert_eq!(sum, lib_sum, "decoder tag sum mismatch for {name}");
    }
    println!("sanity ok: Vec mirrors match the library");

    println!("\nstruct sizes (bytes)");
    println!(
        "  Decoder    (SmallVec<[(Tag,u32,u32); 32]>): {}",
        std::mem::size_of::<Decoder>()
    );
    println!(
        "  VecDecoder (Vec<(Tag,u32,u32)>):            {}",
        std::mem::size_of::<VecDecoder>()
    );
    println!(
        "  Encoder    (SmallVec<[u8; 512]>):           {}",
        std::mem::size_of::<Encoder>()
    );
    println!(
        "  VecEncoder (Vec<u8>):                       {}",
        std::mem::size_of::<VecEncoder>()
    );

    println!("\nheap allocations per message (averaged over {ITERS} iterations)");
    println!(
        "{:<22} {:>8} {:>10} {:>10}",
        "scenario", "small", "typical", "large"
    );
    println!("{:-<22} {:-<8} {:-<10} {:-<10}", "", "", "", "");

    print_row("decode cold  smallvec", decode_cold_smallvec());
    print_row("decode cold  vec", decode_cold_vec());
    print_row("decode reuse smallvec", decode_reuse_smallvec());
    print_row("decode reuse vec", decode_reuse_vec());
    print_row("encode cold  smallvec", encode_cold_smallvec());
    print_row("encode cold  vec", encode_cold_vec());
    print_row("encode reuse smallvec", encode_reuse_smallvec());
    print_row("encode reuse vec", encode_reuse_vec());
}
