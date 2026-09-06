# fix-codec-rs

A high-performance FIX (Financial Information Exchange) protocol encoder/decoder library written in Rust.

We target to encode and decode level, no session or application level for this protocol.

Tested with FIX versions 4.2, 4.4, and 5.0

## Features

- **Zero-copy decoding** — field values are byte slices into the original buffer, no allocation on the hot path
- **Reusable decoder/encoder** — single instance across thousands of messages, amortizes allocation cost
- **SmallVec inline storage** — 95%+ of messages fit in inline stack storage (32-field default), avoiding heap allocation entirely
- **SIMD-accelerated scanning** — uses `memchr` for fast `=` and SOH delimiter search
- **Lazy sorted index** — O(log n) `find()` via binary search, built only on first use
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
fix-codec-rs = "0.2.0"
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

    let msg = decoder.decode(raw).unwrap();

    // Access fields by index (O(1))
    for field in msg.fields() {
        println!("Tag {}: {:?}", field.tag, field.value);
    }

    // Lookup by tag (O(log n) binary search, index built lazily on first call)
    if let Some(field) = msg.find(tag::SYMBOL) {
        println!("Symbol: {}", std::str::from_utf8(field.value).unwrap());
    }

    if let Some(field) = msg.find(tag::ORDER_QTY) {
        println!("Qty: {}", std::str::from_utf8(field.value).unwrap());
    }
}
```

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

### Decode throughput

| Message                    | Time    | Throughput  |
|----------------------------|---------|-------------|
| Tiny (26 B)                | 14.7 ns | 1.65 GiB/s  |
| New Order Single (118 B)   | 122.5 ns| 919.0 MiB/s |
| Execution Report (162 B)   | 172.2 ns| 897.4 MiB/s |
| Market Data Snapshot (189 B) | 186.1 ns| 968.3 MiB/s |
| FIX 5.0 RootParties (129 B) | 146.0 ns| 842.7 MiB/s |

### `find()` strategy: binary search vs linear scan

Measured on the Execution Report message:

| Strategy      | 1 lookup | 4 lookups | 8 lookups |
|---------------|----------|-----------|-----------|
| Binary search | 281.7 ns | 290.3 ns  | 305.8 ns  |
| Linear scan   | 180.0 ns | 200.2 ns  | 210.4 ns  |

Binary search (via lazy sorted index) is the default. On small messages the
sorted-index build cost means linear scan is faster for a handful of lookups;
binary search pays off on larger messages and higher lookup counts.

### Encode throughput

| Message                     | Time    | Throughput  |
|-----------------------------|---------|-------------|
| Tiny (26 B)                 | 77.0 ns | 322.1 MiB/s |
| New Order Single (118 B)    | 334.4 ns| 336.5 MiB/s |
| Execution Report (162 B)    | 465.4 ns| 332.0 MiB/s |
| Market Data Snapshot (189 B)| 491.8 ns| 366.5 MiB/s |
| FIX 5.0 RootParties (129 B) | 380.7 ns| 323.1 MiB/s |

### Roundtrip (decode + encode)

| Message                   | Time    | Throughput  |
|---------------------------|---------|-------------|
| New Order Single (118 B)  | 340.3 ns| 330.7 MiB/s |
| Execution Report (162 B)  | 468.0 ns| 330.2 MiB/s |

### FIX 5.0 group iteration

| Benchmark                                   | Time    | Throughput  |
|---------------------------------------------|---------|-------------|
| RootParties → RootPartySubIDs iterate + find | 522.0 ns| 235.7 MiB/s |

Run full benchmarks:

```sh
cargo bench
# Open HTML report
open target/criterion/report/index.html
```

## Design Notes

**Zero-copy** — `Message<'a>` and `Group<'a>` hold references into the original input buffer. No string copies. Values are `&[u8]` slices; callers parse numeric/string values as needed.

**Reusable decoder** — `Decoder` holds a `SmallVec` internally. Reuse the same instance across messages to avoid repeated allocation. The decoder clears internal state on each `decode()` call.

**SmallVec inline storage** — field offset storage fits 32 entries inline on the stack. Messages with more than 32 fields spill to the heap automatically.

**Lazy sorted index** — `Message::find()` builds a sorted tag index on first call using `OnceCell`. Subsequent `find()` calls on the same message use binary search. If you only iterate with `fields()`, no sort ever happens.

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
