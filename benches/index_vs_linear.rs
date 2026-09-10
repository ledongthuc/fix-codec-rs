//! Evidence for dropping the `BTreeMap` tag index.
//!
//! This binary measures, for a range of field counts `n`, the cost of building
//! a reusable `BTreeMap<(Tag, u32), (u32, u32)>` index (the exact
//! `rebuild_index()` cost `Decoder::decode` used to pay) versus a linear
//! `position()` scan over a pre-parsed `Vec<(Tag, u32, u32)>` (the current
//! `find`/`find_all` implementation), at three lookup positions: first, mid,
//! and last/absent.
//!
//! This replaces the flawed `bench_sorted_vs_linear` Criterion benchmark: both
//! of its arms still paid `decode()`'s index build, so it misrepresented the
//! index/linear comparison. Run:
//!
//! ```sh
//! cargo bench --bench index_vs_linear
//! ```

use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

type Tag = u32;
type Offsets = Vec<(Tag, u32, u32)>;

const SIZES: [usize; 6] = [16, 32, 64, 128, 256, 512];

/// Build a wire-order offset vector with `n` fields, tag `n` starting at 8.
fn offsets(n: usize) -> Offsets {
    (0..n)
        .map(|i| {
            let tag = 8 + i as u32;
            let start = i as u32 * 4;
            (tag, start, start + 1)
        })
        .collect()
}

/// Linear scan over the offsets, returning the position of the first match.
#[inline]
fn linear_find(offsets: &[(Tag, u32, u32)], tag: Tag) -> Option<usize> {
    offsets.iter().position(|&(t, _, _)| t == tag)
}

/// Rebuild the tag index (clear + insert) exactly as `rebuild_index()` did.
fn build_index(offsets: &[(Tag, u32, u32)], map: &mut BTreeMap<(Tag, u32), (u32, u32)>) {
    map.clear();
    for (i, &(tag, start, end)) in offsets.iter().enumerate() {
        map.insert((tag, i as u32), (start, end));
    }
}

/// Time `iters` iterations of `f`, returning nanoseconds per iteration.
fn ns_per_iter(iters: u64, mut f: impl FnMut()) -> f64 {
    // Warm up so the first timed iteration is not paying cold-cache costs.
    for _ in 0..(iters / 10).max(1) {
        f();
    }
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    start.elapsed().as_nanos() as f64 / iters as f64
}

fn main() {
    let iters = 200_000u64;

    println!("index build (clear + insert), ns/iter");
    println!("{:>8} {:>12}", "n", "build");
    for n in SIZES {
        let offs = offsets(n);
        let mut map = BTreeMap::new();
        let ns = ns_per_iter(iters, || build_index(black_box(&offs), &mut map));
        println!("{n:>8} {ns:>12.1}");
    }

    println!();
    println!("linear scan (position), ns/iter");
    println!(
        "{:>8} {:>10} {:>10} {:>14}",
        "n", "first", "mid", "last/absent"
    );
    for n in SIZES {
        let offs = offsets(n);
        let first_tag = offs[0].0;
        let mid_tag = offs[n / 2].0;
        let absent_tag = u32::MAX; // forces a full scan
        let scan_iters = iters * 4; // scan is much cheaper; more iters for stability
        let first = ns_per_iter(scan_iters, || {
            black_box(linear_find(black_box(&offs), first_tag));
        });
        let mid = ns_per_iter(scan_iters, || {
            black_box(linear_find(black_box(&offs), mid_tag));
        });
        let last = ns_per_iter(scan_iters, || {
            black_box(linear_find(black_box(&offs), absent_tag));
        });
        println!("{n:>8} {first:>10.2} {mid:>10.2} {last:>14.2}");
    }
}
