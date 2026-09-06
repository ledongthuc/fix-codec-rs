//! Allocation-count benchmark for the library's `Vec`-backed `Decoder`/`Encoder`.
//!
//! This uses a counting global allocator to report the *actual* number of heap
//! allocations per message.
//! Run with: `cargo bench --bench alloc_count`

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "common/mod.rs"]
mod common;

use common::fixtures;
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

fn decode_cold() -> [u64; 3] {
    per_fixture(|buf| {
        allocs(|| {
            for _ in 0..ITERS {
                let mut dec = Decoder::new();
                black_box(dec.decode(buf).unwrap().len());
            }
        }) / ITERS
    })
}

fn decode_reuse() -> [u64; 3] {
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

fn encode_cold() -> [u64; 3] {
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

fn encode_reuse() -> [u64; 3] {
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

fn print_row(label: &str, row: [u64; 3]) {
    println!("{:<16} {:>8} {:>10} {:>10}", label, row[0], row[1], row[2]);
}

fn main() {
    // Sanity check: the library must round-trip every fixture.
    for (name, buf) in fixtures() {
        let mut dec = Decoder::new();
        let msg = dec.decode(buf).unwrap();

        let mut enc = Encoder::new();
        let mut out = Vec::with_capacity(8192);
        enc.encode(&msg, &mut out).unwrap();

        let msg2 = dec.decode(&out).unwrap();
        msg2.validate_body_length().unwrap();
        msg2.validate_checksum().unwrap();
        println!("sanity ok: round-trip validated for {name}");
    }

    println!("\nstruct sizes (bytes)");
    println!(
        "  Decoder (Vec<(Tag,u32,u32)>): {}",
        std::mem::size_of::<Decoder>()
    );
    println!(
        "  Encoder (Vec<u8>):            {}",
        std::mem::size_of::<Encoder>()
    );

    println!("\nheap allocations per message (averaged over {ITERS} iterations)");
    println!(
        "{:<16} {:>8} {:>10} {:>10}",
        "scenario", "small", "typical", "large"
    );
    println!("{:-<16} {:-<8} {:-<10} {:-<10}", "", "", "", "");

    print_row("decode cold", decode_cold());
    print_row("decode reuse", decode_reuse());
    print_row("encode cold", encode_cold());
    print_row("encode reuse", encode_reuse());
}
