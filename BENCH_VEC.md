# SmallVec vs Vec benchmark

This document records an A/B benchmark that tests whether the library's use of
`SmallVec` in the `Decoder` and `Encoder` actually pays off, versus using a
plain `Vec` for the same storage.

## Files

| File | Purpose |
|------|---------|
| `benches/common/mod.rs` | Shared fixtures + three mirror implementations of the codec |
| `benches/smallvec_vs_vec.rs` | Criterion throughput benchmark |
| `benches/alloc_count.rs` | Counting global allocator → heap allocations per message |

## Running

```sh
cargo bench --bench alloc_count        # allocations per message (near-instant)
cargo bench --bench smallvec_vs_vec    # throughput (Criterion)
```

The numbers below were collected with a reduced Criterion profile for speed:

```sh
cargo bench --bench smallvec_vs_vec -- \
    --warm-up-time 0.2 --measurement-time 0.8 --sample-size 10
```

Re-run with default Criterion settings for final, publication-grade numbers.

## Method

### Fixtures

| Name | Description | Fields | Body |
|------|-------------|--------|------|
| `small_4fields` | Minimal `8=…9=…35=D…10=…` | 4 | ~5 B |
| `typical_14fields` | NewOrderSingle-like | 14 | ~73 B |
| `large_100fields` | Synthetic, 100 fields × 16 B values | 100 | ~2 KB |

`small` and `typical` fit inside every inline capacity. `large` exceeds both
the decoder's 32-field inline capacity and the encoder's 512-byte inline
capacity, forcing `SmallVec` to spill to the heap.

### Variants (identical algorithm, different storage)

| Label | Storage | Crate boundary |
|-------|---------|----------------|
| `lib` | `SmallVec` (the real library) | cross-crate call |
| `smallvec` | `SmallVec` (same-crate mirror) | none (isolates inlining) |
| `vec` | `Vec` (same-crate mirror) | none |

The `smallvec` and `vec` mirrors are byte-for-byte the same code; the **only**
difference is the storage type, so any measured delta is attributable to
`SmallVec` vs `Vec`. The `lib` variant is included to show what a real user of
the library gets (including the cross-crate call boundary).

### Access patterns

- `reuse` — one instance created up-front, used on every iteration
  (steady state; the library's intended usage).
- `cold` — a fresh instance per iteration (includes first-message cost and the
  first heap allocation, which `SmallVec` is supposed to avoid).

## Results

### Heap allocations per message

Measured with a counting global allocator, averaged over 100 000 iterations.

| scenario | small (4f) | typical (14f) | large (100f) |
|---|---|---|---|
| decode cold · smallvec | **0** | **0** | **2** |
| decode cold · vec | 1 | 3 | 6 |
| decode reuse · smallvec | 0 | 0 | 0 |
| decode reuse · vec | 0 | 0 | 0 |
| encode cold · smallvec | **0** | **0** | **2** |
| encode cold · vec | 1 | 5 | 9 |
| encode reuse · smallvec | 0 | 0 | 0 |
| encode reuse · vec | 0 | 0 | 0 |

Key points:

- For messages that fit inline, `SmallVec` performs **zero** allocations even on
  a cold start; `Vec` performs at least one.
- For the 100-field case `SmallVec` still allocates *fewer* times (2 vs 6 for
  decode, 2 vs 9 for encode) because its inline storage absorbs the first 32
  fields / 512 bytes before the growth reallocations begin.
- In the `reuse` (steady-state) pattern both reach **zero** allocations per
  message — `Vec::clear()` and `SmallVec::clear()` both preserve capacity.

### Struct sizes

| Type | Size |
|------|------|
| `Decoder` (`SmallVec<[(Tag,u32,u32); 32]>`) | 400 B |
| `VecDecoder` (`Vec<(Tag,u32,u32)>`) | 24 B |
| `Encoder` (`SmallVec<[u8; 512]>`) | 536 B |
| `VecEncoder` (`Vec<u8>`) | 24 B |

`SmallVec` embeds its buffer inline, which is why these structs are much larger.
This is harmless for "create once, reuse" but is a real cost if an instance is
constructed per message.

### Throughput (median)

#### Decode

| message | `lib` | `smallvec` | `vec` |
|---|---|---|---|
| small · reuse | 18.9 ns | 16.5 ns | 15.7 ns |
| small · cold | 18.5 ns | 15.6 ns | 27.8 ns |
| typical · reuse | 132 ns | 127 ns | 125 ns |
| typical · cold | 134 ns | 126 ns | 146 ns |
| large · reuse | 1224 ns | 1204 ns | 1216 ns |
| large · cold | 1267 ns | 1241 ns | 1279 ns |

`SmallVec` and `Vec` are effectively tied in steady-state decode. `SmallVec`
wins cold-start for small/typical messages because it skips the `malloc`, while
`Vec` cold-start is slower by roughly the cost of one allocation.

#### Encode

| message | `lib` | `smallvec` | `vec` |
|---|---|---|---|
| small · reuse | 24.8 ns | 22.7 ns | 18.8 ns |
| small · cold | 25.1 ns | 22.5 ns | 30.0 ns |
| typical · reuse | 147 ns | 132 ns | 78 ns |
| typical · cold | 144 ns | 133 ns | 174 ns |
| large · reuse | 1015 ns | 941 ns | 540 ns |
| large · cold | 1141 ns | 1056 ns | 838 ns |

Encode is where the two storage types diverge:

- **Cold start**: `SmallVec` is faster for small/typical (no allocation), but
  `Vec` is faster for large messages — once the body is big enough, the faster
  `Vec` append path outweighs the one-time malloc.
- **Steady-state reuse**: `Vec` is **~1.7× faster** for typical/large bodies
  (132 → 78 ns, 941 → 540 ns). This is a real storage difference, not an
  inlining artifact — the same-crate `smallvec` mirror is also slow.

### Bonus: cross-crate call overhead

`lib_*` is consistently ~7–15% slower than the same-crate `smallvec` mirror.
The library's `Decoder::decode` and `Encoder::encode` are not marked
`#[inline]`, so users pay a cross-crate call boundary. This is unrelated to
`SmallVec` and is a separate, easy optimization.

## Interpretation

- **Decoder — `SmallVec` is a clear win.** It removes *all* heap allocations
  for the common case (≤32 fields), reduces allocation count even for larger
  messages, and matches `Vec` throughput. The larger struct size is irrelevant
  because the decoder is created once and reused.
- **Encoder — `SmallVec` is a mixed bag, leaning negative for the recommended
  usage.** It avoids cold-start allocations, but the body scratch buffer is
  measurably slower than a plain `Vec` in the steady-state `reuse` pattern the
  library itself recommends. For a long-lived encoder, a `Vec<u8>` body buffer
  (or a two-pass encode that computes body length first and writes directly
  into the output, removing the scratch buffer entirely) would be faster.

## Caveats

- The `large` fixture is synthetic (tags `1..=100`, fixed-size values), used to
  force a heap spill; it is not a realistic FIX message.
- Throughput figures above come from a reduced Criterion profile; exact numbers
  vary by machine and toolchain, but the *relative* ordering (`Vec` ≈ `SmallVec`
  for decode, `Vec` faster than `SmallVec` for encode reuse) was consistent
  across runs.
