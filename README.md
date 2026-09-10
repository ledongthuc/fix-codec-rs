# fix-codec-rs

A high-performance FIX (Financial Information Exchange) protocol encoder/decoder library written in Rust.

We target to encode and decode level, no session or application level for this protocol.

Tested with FIX versions 4.2, 4.4, and 5.0

## Features

- **Zero-copy decoding** — field values are byte slices into the original buffer
- **Reusable decoder/encoder** — single instance across thousands of messages, amortizes allocation cost
- **Reusable internal buffers** — the decoder's field-offset `Vec` and the encoder's body `Vec` are cleared and reused across messages
- **SIMD-accelerated scanning** — uses `memchr` for fast `=` and SOH delimiter search
- **Allocation-free `decode`** — `find()`/`find_all()` are linear scans over a reused offset buffer (no index)
- **Lazy field iteration** — `decode_fields()` parses one field per `next()` with no storage and no allocation
- **Repeating groups** — full support for nested groups, FIX 4.2, FIX 4.4, and FIX 5.0 specifications
- **Auto checksum/body length** — automatic tag 9 and tag 10 computation during encoding (toggleable)
- **1100+ tag constants** — comprehensive coverage of FIX 4.2, FIX 4.4, and FIX 5.0 tag definitions

## Installation

```sh
cargo add fix-codec-rs
```

Or add to your `Cargo.toml` manually:

```toml
[dependencies]
fix-codec-rs = "0.3.0"
```

## Usage

### Decoding

```rust
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::tag;

fn main() {
    // Create a reusable decoder — allocate once, reuse across messages
    let mut decoder = Decoder::new();

    let raw = b"8=FIX.4.2\x019=73\x0135=D\x0149=CLIENT\x0156=BROKER\x0134=1\x0152=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0144=150.00\x0140=2\x0110=128\x01";

    // Full decode: reuses the offset buffer; allocation-free in steady state.
    {
        let msg = decoder.decode(raw).unwrap();

        // Access fields in wire order
        for field in msg.fields() {
            println!("Tag {}: {:?}", field.tag, field.value);
        }

        // Lookup by tag (linear scan over the wire-order offsets)
        if let Some(field) = msg.find(tag::SYMBOL) {
            println!("Symbol: {}", std::str::from_utf8(field.value).unwrap());
        }

        if let Some(field) = msg.find(tag::ORDER_QTY) {
            println!("Qty: {}", std::str::from_utf8(field.value).unwrap());
        }

        // Every occurrence of a repeated tag, in original wire order
        for field in msg.find_all(tag::TEXT) {
            println!("Text: {:?}", field.value);
        }
    } // msg dropped here — the decoder can be reused

    // Lazy iteration: one field per next(), no offsets, no allocation.
    // `buf` must be a complete message; this is not resumable/incremental.
    for field in decoder.decode_fields(raw) {
        match field {
            Ok(f) => println!("Tag {}: {:?}", f.tag, f.value),
            Err(e) => eprintln!("parse error: {:?}", e),
        }
    }
}
```

### Choosing an API

- **`decode_fields`** — lazy field iteration. Parses one field per `next()`,
  stores no offsets, borrows only the input buffer, and is allocation-free. Use
  it when you only need to walk fields and never look up by tag (and can
  re-parse the complete buffer from scratch on error).
- **`decode`** — full zero-copy view (`fields`/`groups`/validation) plus
  `find`/`find_all`, which are linear scans over the wire-order offsets. The
  offset `Vec` is cleared and reused, so this path is allocation-free in steady
  state. Use it when you need tag lookups, groups, or validation.

### Decoding with Validation

```rust
use fix_codec_rs::decoder::Decoder;

let mut decoder = Decoder::new();
let raw = b"8=FIX.4.2\x019=73\x0135=D\x0149=CLIENT\x0156=BROKER\x0134=1\x0152=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0144=150.00\x0140=2\x0110=128\x01";

let msg = decoder.decode(raw).unwrap();

// Validate body length (tag 9) and checksum (tag 10)
msg.validate_body_length().unwrap();
msg.validate_checksum().unwrap();
```

### Encoding

```rust
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::encoder::Encoder;

let mut decoder = Decoder::new();
let mut encoder = Encoder::new();

let raw = b"8=FIX.4.2\x019=73\x0135=D\x0149=CLIENT\x0156=BROKER\x0134=1\x0152=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0144=150.00\x0140=2\x0110=128\x01";

let msg = decoder.decode(raw).unwrap();

// Encode back to wire format — tag 9 and tag 10 are recomputed automatically
let mut out = Vec::new();
encoder.encode(&msg, &mut out).unwrap();
```

### Encoding with Auto-Calculation Disabled

```rust
use fix_codec_rs::encoder::Encoder;

let mut encoder = Encoder::new();

// Preserve original tag 9 and tag 10 values without recomputing
encoder.disable_auto_calculate_body_length(true);
encoder.disable_auto_calculate_checksum(true);
```

### Pre-sizing for Large Messages

```rust
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::encoder::Encoder;

// Pre-allocate for messages with up to 64 fields (avoids reallocation)
let mut decoder = Decoder::with_capacity(64);

// Pre-allocate 1024-byte output buffer
let mut encoder = Encoder::with_capacity(1024);
```

### Repeating Groups

```rust
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::group;
use fix_codec_rs::tag;

let mut decoder = Decoder::new();

// Market data snapshot with 2 MD entries
let raw = b"8=FIX.4.2\x019=100\x0135=W\x0149=SERVER\x0156=CLIENT\x01268=2\x01269=0\x01270=150.25\x01271=500\x01269=1\x01270=150.30\x01271=300\x0110=200\x01";

let msg = decoder.decode(raw).unwrap();

// Iterate MD entries using the built-in FIX 4.2 group spec
for entry in msg.groups(&group::MD_ENTRIES) {
    if let Some(price) = entry.find(tag::MD_ENTRY_PX) {
        println!("Price: {}", std::str::from_utf8(price.value).unwrap());
    }
    if let Some(size) = entry.find(tag::MD_ENTRY_SIZE) {
        println!("Size: {}", std::str::from_utf8(size.value).unwrap());
    }
}
```

### Nested Groups

```rust
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::group;
use fix_codec_rs::tag;

let mut decoder = Decoder::new();
let raw = b"8=FIX.4.4\x019=...\x0135=AE\x01453=2\x01448=FIRM_A\x01447=D\x01452=1\x01539=1\x01524=TRADER1\x01448=FIRM_B\x01447=D\x01452=2\x0110=000\x01";

let msg = decoder.decode(raw).unwrap();

for party in msg.groups(&group::PARTY_IDS) {
    if let Some(id) = party.find(tag::PARTY_ID) {
        println!("Party: {}", std::str::from_utf8(id.value).unwrap());
    }
    // Access nested group within each party
    for nested in party.groups(&group::NESTED_PARTY_IDS) {
        if let Some(nid) = nested.find(tag::NESTED_PARTY_ID) {
            println!("  Nested: {}", std::str::from_utf8(nid.value).unwrap());
        }
    }
}
```

### Custom Group Spec

```rust
use fix_codec_rs::group::GroupSpec;
use fix_codec_rs::tag;

// Define a custom repeating group
const MY_GROUP: GroupSpec = GroupSpec {
    count_tag: tag::NO_ALLOCS,      // Tag that holds the group count
    delimiter_tag: tag::ALLOC_ACCOUNT, // First tag of each instance
    member_tags: &[
        tag::ALLOC_ACCOUNT,
        tag::ALLOC_SHARES,
        tag::ALLOC_PRICE,
    ],
};
```

## Benchmark

Benchmarks run with Criterion.rs on Apple M-series (arm64). Run your own with `cargo bench`.

### Heap allocations per message

Measured with the counting-allocator benchmark (`cargo bench --bench alloc_count`),
averaged over 100 000 iterations.

| scenario        | small (4 fields) | typical (14 fields) | large (100 fields) |
|-----------------|------------------|---------------------|--------------------|
| decode cold     | 1                | 3                   | 6                  |
| decode reuse    | 0                | 0                   | 0                  |
| decode_fields   | 0                | 0                   | 0                  |
| encode cold     | 1                | 5                   | 9                  |
| encode reuse    | 0                | 0                   | 0                  |

A fresh `Decoder` allocates only as its offset `Vec` grows to fit `decode`
(geometric growth: 1 / 3 / 6 allocations for the 4 / 14 / 100 field fixtures).
On reuse the `Vec` is cleared (capacity preserved), so `decode` is
allocation-free in steady state. `decode_fields` (which stores nothing) and
steady-state `encode` are also allocation-free. Struct sizes: `Decoder` is 24 B
and `Encoder` is 32 B.

### Decode throughput

| Message                    | Time    | Throughput   |
|----------------------------|---------|--------------|
| Tiny (26 B)                | 13.9 ns | 1787.1 MiB/s |
| New Order Single (118 B)   | 119.1 ns| 944.8 MiB/s  |
| Execution Report (162 B)   | 169.2 ns| 913.1 MiB/s  |
| Market Data Snapshot (189 B)| 181.2 ns| 994.7 MiB/s  |
| FIX 5.0 RootParties (129 B) | 142.7 ns| 861.9 MiB/s  |

### `decode_fields` throughput (lazy iteration)

| Message                    | Time    | Throughput  |
|----------------------------|---------|-------------|
| Tiny (26 B)                | 25.2 ns | 985.2 MiB/s |
| New Order Single (118 B)   | 151.4 ns| 743.1 MiB/s |
| Execution Report (162 B)   | 210.3 ns| 734.6 MiB/s |
| Market Data Snapshot (189 B) | 224.9 ns| 801.5 MiB/s |

### `find()` strategy: why there is no tag index (measured)

A sweep measured, per message, the cost of building a `BTreeMap` tag index
versus a linear scan over the parsed field offsets, across field counts `n`
(Apple M-series arm64). The index is rebuilt per message (no cross-message
caching is possible in this design), so its build cost cannot amortize.

Index build cost (per message):

| n fields | 16 | 32 | 64 | 128 | 256 | 512 |
|---|---:|---:|---:|---:|---:|---:|
| BTreeMap | 270 ns | 281 ns | 627 ns | 1536 ns | 3414 ns | 7652 ns |

Linear scan cost (single `find`):

| n fields | first (tag 8) | mid | last / absent |
|---|---:|---:|---:|
| 16 | 0.49 ns | 2.26 ns | 4.27 ns |
| 32 | 0.47 ns | 4.45 ns | 8.67 ns |
| 64 | 0.45 ns | 8.99 ns | 19.89 ns |
| 128 | 0.48 ns | 16.07 ns | 34.91 ns |
| 256 | 0.44 ns | 34.96 ns | 67.19 ns |
| 512 | 0.45 ns | 69.79 ns | 124.37 ns |

The index build is ~1.7 ns · n · log₂(n); a linear full scan is ~0.25 ns · n.
Even in the most favorable case for the index (an absent or trailing tag, which
forces a full scan), breaking even requires ~8 · log₂(n) lookups — about 48 at
n = 64 and 72 at n = 512. The first-field case (`find(8)`, the hottest path) is
a single comparison and never breaks even.

`find()`/`find_all()` therefore use a linear scan over the parsed offsets
(`O(n)`). The `BTreeMap` index never pays off for realistic message sizes and
lookup counts, so the library does not build one.

### Encode throughput

| Message                     | Time    | Throughput  |
|-----------------------------|---------|-------------|
| Tiny (26 B)                 | 32.7 ns | 758.6 MiB/s |
| New Order Single (118 B)    | 193.3 ns| 582.1 MiB/s |
| Execution Report (162 B)    | 275.9 ns| 560.0 MiB/s |
| Market Data Snapshot (189 B)| 294.0 ns| 613.0 MiB/s |
| FIX 5.0 RootParties (129 B) | 232.0 ns| 530.3 MiB/s |

### Roundtrip (decode + encode)

| Message                   | Time    | Throughput  |
|---------------------------|---------|-------------|
| New Order Single (118 B)  | 192.8 ns| 583.7 MiB/s |
| Execution Report (162 B)  | 273.6 ns| 564.8 MiB/s |

### FIX 5.0 group iteration

| Benchmark                                   | Time    | Throughput  |
|---------------------------------------------|---------|-------------|
| RootParties → RootPartySubIDs iterate + find | 476.2 ns| 258.4 MiB/s |

Run full benchmarks:

```sh
cargo bench
# Open HTML report
open target/criterion/report/index.html
```

## Design Notes

**Zero-copy** — `Message<'a>` and `Group<'a>` hold references into the original input buffer. No string copies. Values are `&[u8]` slices; callers parse numeric/string values as needed.

**Reusable decoder** — `Decoder` holds a single field-offset `Vec`. Reuse the same instance across messages to avoid repeated offset reallocation. The decoder clears it on each `decode()` call (capacity preserved).

**Reusable internal buffers** — the decoder's field-offset `Vec` and the encoder's body scratch buffer are cleared (not dropped) between calls, so their capacity is preserved and steady-state `decode`/`encode` perform zero allocations.

**Linear `find`/`find_all`** — `find` and `find_all` scan the wire-order offset slice (`O(n)`) and return matches in wire order. There is no tag index: measured, an eagerly built `BTreeMap` index costs more than it saves at realistic message sizes and lookup counts (see the `find()` strategy section above).

**Lazy field iteration** — `Decoder::decode_fields` parses one field per `next()` directly from the buffer. It stores no offsets and no index, takes `&self`, and is allocation-free. It is not resumable: `buf` must be a complete message and a parse error fuses the iterator.

**Group specs are `'static`** — built-in `GroupSpec` values reference static tag slices. Zero overhead at runtime.

## Supported FIX Versions

| Version | Tag Coverage | Group Specs |
|---------|-------------|-------------|
| FIX 4.2 | 450+ tags   | 19 groups   |
| FIX 4.4 | 500+ tags   | 55 groups   |
| FIX 5.0 | 177 new tags (957–1139) | 65 groups   |

Tag constants are in `fix_codec_rs::tag`. Group specs are flat constants in
`fix_codec_rs::group` (e.g. `group::MD_ENTRIES`, `group::PARTY_IDS`), plus the
version arrays `FIX42_GROUPS`, `FIX44_GROUPS`, and `FIX50_GROUPS` (there are no
`group::fix42`/`group::fix44` submodules).

## FIX 5.0 (`FIXT.1.1`) notes

FIX 5.0 does **not** use `BeginString(8) = FIX.5.0`. The session/transport
`BeginString` is **`FIXT.1.1`**, and the application version is carried in
header field **`ApplVerID(1128)`** (`4` = FIX 4.2, `6` = FIX 4.4, `7` = FIX 5.0).
A `BeginString = FIX.5.0` is not the standard spelling.

Group dispatch uses the message's own fields via `Message::resolve_version()`:

| `BeginString(8)` | `ApplVerID(1128)` | Group specs |
|------------------|-------------------|-------------|
| `FIXT.1.1`       | `7`               | `FIX50_GROUPS` |
| `FIXT.1.1`       | `6`               | `FIX44_GROUPS` |
| `FIXT.1.1`       | `4`               | `FIX42_GROUPS` |
| `FIXT.1.1`       | absent            | empty (unknown — no guessing) |
| `FIXT.1.1`       | other             | empty (unsupported) |
| `FIX.4.4`        | any/absent        | `FIX44_GROUPS` |
| `FIX.4.2`        | any/absent        | `FIX42_GROUPS` |
| absent/unknown   | any/absent        | empty (unknown — no guessing) |

A `FIXT.1.1` Logon (`35=A`) legitimately carries the `NoMsgTypes(384)` repeating
group but usually has no `ApplVerID`, so `all_groups()` returns an empty set for
it. To iterate Logon `NoMsgTypes`, call the version-dispatch-free API directly:

```rust
for entry in msg.groups(&group::MSG_TYPES) {
    // ...
}
```

## Behavior changes in 0.3.0

- `Decoder::decode` is allocation-free in steady state: no tag index is built;
  the field-offset `Vec` is cleared and reused.
- `Message::find` is now a linear scan and returns the **first occurrence in
  wire order** (0.2.0 built a sorted index lazily on first `find` and returned
  an arbitrary duplicate, due to an unstable sort).
- New `Message::find_all` yields every occurrence of a tag in wire order.
- New `Decoder::decode_fields` provides lazy, allocation-free field iteration.
- The 0.2.0 `u16` field-index truncation caveat is gone (no index exists).

## Breaking changes in 0.2.0

`Message::all_groups()` no longer falls back to `FIX42_GROUPS` for unknown or
absent `BeginString` values. Unknown/absent `BeginString`, unresolved `FIXT.1.1`,
and the valid-but-unsupported legacy `BeginString`s `FIX.4.0`, `FIX.4.1`, and
`FIX.4.3` now return an empty group set. Use `Message::resolve_version()`
(`Option<FixVersion>`, where `None` means unknown/unsupported) for a hard signal,
or the raw accessors `Message::fix_version()` / `Message::appl_ver_id()`.

The encoder keeps `8=FIX.4.4` as the default when tag 8 is absent regardless of
`ApplVerID`; a constructed message with `ApplVerID` but no `8=` encodes as
`8=FIX.4.4` + `1128=…`. Set `8=FIXT.1.1` explicitly when building FIX 5.0
messages from scratch. Transport-independence (omitting `8=` entirely) is out of
scope for this codec.

## Dev Setup

### Prerequisites

- Rust toolchain (stable, edition 2024): https://rustup.rs

### Clone and build

```sh
git clone https://github.com/ledongthuc/fix-codec-rs
cd fix-codec-rs
cargo build
```

### Run tests

```sh
cargo test
```

### Run benchmarks

```sh
cargo bench
```

Criterion generates an HTML report at `target/criterion/report/index.html`.

## Contributing

Contributions are welcome. Please follow the process below:

1. Fork the repository and create a feature branch from `main`.
2. Write tests for any new behavior. All existing tests must pass.
3. Run `cargo clippy -- -D warnings` and `cargo fmt` before submitting.
4. Open a pull request with a clear description of the change and motivation.

For bug reports, open a GitHub issue with a minimal reproducing example — ideally a failing test case.

## License

MIT
