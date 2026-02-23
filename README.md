# fix-rs

A high-performance FIX (Financial Information Exchange) protocol encoder/decoder library written in Rust, designed for HFT and low-latency trading systems.

We target to encode and decode level, no session or application level for this protocol.

Tested with FIX version 4.2 and 4.4

## Features

- **Zero-copy decoding** — field values are byte slices into the original buffer, no allocation on the hot path
- **Reusable decoder/encoder** — single instance across thousands of messages, amortizes allocation cost
- **SmallVec inline storage** — 95%+ of messages fit in inline stack storage (32-field default), avoiding heap allocation entirely
- **SIMD-accelerated scanning** — uses `memchr` for fast `=` and SOH delimiter search
- **Lazy sorted index** — O(log n) `find()` via binary search, built only on first use
- **Repeating groups** — full support for nested groups, both FIX 4.2 and FIX 4.4 specifications
- **Auto checksum/body length** — automatic tag 9 and tag 10 computation during encoding (toggleable)
- **500+ tag constants** — comprehensive coverage of FIX 4.2 and FIX 4.4 tag definitions

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fix-rs = { git = "https://github.com/ledongthuc/fix-rs" }
```

## Usage

### Decoding

```rust
use fix_rs::decoder::Decoder;
use fix_rs::tag;

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
use fix_rs::decoder::Decoder;

let mut decoder = Decoder::new();
let raw = b"8=FIX.4.2\x019=73\x0135=D\x0149=CLIENT\x0156=BROKER\x0134=1\x0152=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0144=150.00\x0140=2\x0110=128\x01";

let msg = decoder.decode(raw).unwrap();

// Validate body length (tag 9) and checksum (tag 10)
msg.validate_body_length().unwrap();
msg.validate_checksum().unwrap();
```

### Encoding

```rust
use fix_rs::decoder::Decoder;
use fix_rs::encoder::Encoder;

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
use fix_rs::encoder::Encoder;

let mut encoder = Encoder::new();

// Preserve original tag 9 and tag 10 values without recomputing
encoder.disable_auto_calculate_body_length(true);
encoder.disable_auto_calculate_checksum(true);
```

### Pre-sizing for Large Messages

```rust
use fix_rs::decoder::Decoder;
use fix_rs::encoder::Encoder;

// Pre-allocate for messages with up to 64 fields (avoids reallocation)
let mut decoder = Decoder::with_capacity(64);

// Pre-allocate 1024-byte output buffer
let mut encoder = Encoder::with_capacity(1024);
```

### Repeating Groups

```rust
use fix_rs::decoder::Decoder;
use fix_rs::group;
use fix_rs::tag;

let mut decoder = Decoder::new();

// Market data snapshot with 2 MD entries
let raw = b"8=FIX.4.2\x019=100\x0135=W\x0149=SERVER\x0156=CLIENT\x01268=2\x01269=0\x01270=150.25\x01271=500\x01269=1\x01270=150.30\x01271=300\x0110=200\x01";

let msg = decoder.decode(raw).unwrap();

// Iterate MD entries using the built-in FIX 4.2 group spec
for entry in msg.groups(&group::fix42::MD_ENTRIES) {
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
use fix_rs::decoder::Decoder;
use fix_rs::group;
use fix_rs::tag;

let mut decoder = Decoder::new();
let raw = b"8=FIX.4.4\x019=...\x0135=AE\x01453=2\x01448=FIRM_A\x01447=D\x01452=1\x01539=1\x01524=TRADER1\x01448=FIRM_B\x01447=D\x01452=2\x0110=000\x01";

let msg = decoder.decode(raw).unwrap();

for party in msg.groups(&group::fix44::PARTY_IDS) {
    if let Some(id) = party.find(tag::PARTY_ID) {
        println!("Party: {}", std::str::from_utf8(id.value).unwrap());
    }
    // Access nested group within each party
    for nested in party.groups(&group::fix44::NESTED_PARTY_IDS) {
        if let Some(nid) = nested.find(tag::NESTED_PARTY_ID) {
            println!("  Nested: {}", std::str::from_utf8(nid.value).unwrap());
        }
    }
}
```

### Custom Group Spec

```rust
use fix_rs::group::GroupSpec;
use fix_rs::tag;

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

| Message                     | Fields | Size    | Throughput  |
|-----------------------------|--------|---------|-------------|
| Tiny (1 field)              | 1      | ~9 B    | ~600 ns/msg |
| New Order Single            | 8      | ~73 B   | ~800 ns/msg |
| Execution Report            | 12     | ~104 B  | ~1.0 µs/msg |
| Market Data Snapshot        | 20     | ~100 B  | ~1.1 µs/msg |

### `find()` strategy: binary search vs linear scan

| Strategy        | 1 lookup | 4 lookups | 8 lookups |
|-----------------|----------|-----------|-----------|
| Binary search   | lower    | lower     | lower     |
| Linear scan     | lower    | higher    | higher    |

Binary search (via lazy sorted index) is the default. Break-even point is typically around 2–3 lookups per message.

### Encode throughput

| Message                     | Throughput  |
|-----------------------------|-------------|
| New Order Single            | ~700 ns/msg |
| Execution Report            | ~900 ns/msg |

### Roundtrip (decode + encode)

| Message                     | Throughput    |
|-----------------------------|---------------|
| New Order Single            | ~1.5 µs/msg   |
| Market Data Snapshot        | ~2.0 µs/msg   |

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
| FIX 4.4 | 500+ tags   | 37 groups   |

Tag constants are in `fix_rs::tag`. Group specs are in `fix_rs::group::fix42` and `fix_rs::group::fix44`.

## Dev Setup

### Prerequisites

- Rust toolchain (stable, edition 2024): https://rustup.rs

### Clone and build

```sh
git clone https://github.com/ledongthuc/fix-rs
cd fix-rs
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
