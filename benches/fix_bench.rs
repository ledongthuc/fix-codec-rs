use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::encoder::Encoder;

// ---------------------------------------------------------------------------
// Benchmark inputs
// ---------------------------------------------------------------------------

/// Minimal single-field message (tag 35 only).
const MSG_TINY: &[u8] = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";

/// Typical order message: 8 body fields (NewOrderSingle-like).
const MSG_ORDER: &[u8] = b"8=FIX.4.2\x019=73\x0135=D\x0149=SENDER\x0156=TARGET\x0134=1\x01\
      52=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0140=2\x0144=150.00\x01\
      10=042\x01";

/// Execution report: 12 body fields (ExecutionReport-like).
const MSG_EXEC: &[u8] = b"8=FIX.4.2\x019=104\x0135=8\x0149=TARGET\x0156=SENDER\x0134=2\x01\
      52=20240101-12:00:01\x0111=ORD001\x0137=EXEC001\x0117=FILL001\x0120=0\x01\
      39=2\x0155=AAPL\x0154=1\x0138=100\x0132=100\x0131=150.00\x016=150.00\x01\
      10=201\x01";

/// MarketData snapshot: 2 MD entries (bid + offer), 20+ fields total.
const MSG_MARKET_DATA: &[u8] = b"8=FIX.4.2\x019=100\x0135=W\x0149=MDSRC\x0156=CLIENT\x0134=5\x01\
      52=20240101-12:00:00\x0155=AAPL\x01268=2\x01\
      269=0\x01270=149.50\x01271=500\x01272=20240101\x01273=12:00:00\x01\
      269=1\x01270=150.00\x01271=300\x01272=20240101\x01273=12:00:00\x01\
      10=088\x01";

// ---------------------------------------------------------------------------
// Decode benchmarks
// ---------------------------------------------------------------------------

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for (name, msg) in [
        ("tiny_1field", MSG_TINY),
        ("order_8fields", MSG_ORDER),
        ("exec_report_12fields", MSG_EXEC),
        ("market_data_20fields", MSG_MARKET_DATA),
    ] {
        group.throughput(Throughput::Bytes(msg.len() as u64));
        group.bench_with_input(BenchmarkId::new("reuse", name), msg, |b, msg| {
            let mut dec = Decoder::new();
            b.iter(|| {
                let msg = dec.decode(black_box(msg)).unwrap();
                black_box(msg.len())
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Sorted-index cost/benefit: how many find() calls does it take to break even?
//
// The sorted_index is built on every decode() call — it costs time even when
// you never call find(). These benchmarks isolate that trade-off by measuring:
//   - decode with 0 find() calls  → sorted_index is pure overhead
//   - decode with 1..N find() calls → sorted_index starts paying off
//
// The "linear" variants simulate what the code would do WITHOUT a sorted index
// (iterate offsets from the start) so we can compute the break-even point.
// ---------------------------------------------------------------------------

fn bench_sorted_vs_linear(c: &mut Criterion) {
    use fix_codec_rs::tag;

    // Use the exec-report message (14 fields) as the representative case.
    // It's large enough that the sort cost and the linear-scan savings are both visible.
    let msg_bytes = MSG_EXEC;

    let mut group = c.benchmark_group("sorted_vs_linear");
    group.throughput(Throughput::Bytes(msg_bytes.len() as u64));

    // ---- Baseline: decode only, zero find() calls ----
    // This measures the pure cost of building sorted_index that is never used.
    group.bench_function("decode_only_0finds", |b| {
        let mut dec = Decoder::new();
        b.iter(|| {
            let msg = dec.decode(black_box(msg_bytes)).unwrap();
            black_box(msg.len()) // prevent dead-code elimination
        });
    });

    // ---- find() calls using the binary-search sorted index (current impl) ----
    for n_finds in [1usize, 2, 4, 6, 8] {
        // Tags to look up, in a realistic order for an ExecutionReport.
        let tags = [
            tag::SYMBOL,         // 55
            tag::SIDE,           // 54
            tag::ORDER_QTY,      // 38
            tag::PRICE,          // 44
            tag::MSG_SEQ_NUM,    // 34
            tag::SENDER_COMP_ID, // 49
            tag::CL_ORD_ID,      // 11
            tag::ORD_STATUS,     // 39
        ];
        let label = format!("binary_search_{}finds", n_finds);
        group.bench_function(&label, |b| {
            let mut dec = Decoder::new();
            b.iter(|| {
                let msg = dec.decode(black_box(msg_bytes)).unwrap();
                let mut total = 0usize;
                for &t in &tags[..n_finds] {
                    total += msg.find(t).map(|f| f.value.len()).unwrap_or(0);
                }
                black_box(total)
            });
        });
    }

    // ---- Simulated linear scan (what find() would cost WITHOUT sorted index) ----
    // We iterate over msg.fields() manually and match tags, mirroring O(n) find().
    for n_finds in [1usize, 2, 4, 6, 8] {
        let tags = [
            tag::SYMBOL,
            tag::SIDE,
            tag::ORDER_QTY,
            tag::PRICE,
            tag::MSG_SEQ_NUM,
            tag::SENDER_COMP_ID,
            tag::CL_ORD_ID,
            tag::ORD_STATUS,
        ];
        let label = format!("linear_scan_{}finds", n_finds);
        group.bench_function(&label, |b| {
            let mut dec = Decoder::new();
            b.iter(|| {
                let msg = dec.decode(black_box(msg_bytes)).unwrap();
                let mut total = 0usize;
                for &t in &tags[..n_finds] {
                    // Manual O(n) scan — mimics find() without a sorted index.
                    for f in msg.fields() {
                        if f.tag == t {
                            total += f.value.len();
                            break;
                        }
                    }
                }
                black_box(total)
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Decode + field access benchmarks
// ---------------------------------------------------------------------------

fn bench_decode_and_find(c: &mut Criterion) {
    use fix_codec_rs::tag;

    let mut group = c.benchmark_group("decode_and_find");

    // Simulate a realistic trading path: decode an order, look up key fields.
    // Field values are &[u8] borrowed from the decoder; copy lengths out so
    // the references don't escape the closure.
    group.throughput(Throughput::Bytes(MSG_ORDER.len() as u64));
    group.bench_function("order_find_symbol_side_qty_price", |b| {
        let mut dec = Decoder::new();
        b.iter(|| {
            let msg = dec.decode(black_box(MSG_ORDER)).unwrap();
            let symbol_len = msg.find(tag::SYMBOL).map(|f| f.value.len());
            let side_len = msg.find(tag::SIDE).map(|f| f.value.len());
            let qty_len = msg.find(tag::ORDER_QTY).map(|f| f.value.len());
            let price_len = msg.find(tag::PRICE).map(|f| f.value.len());
            black_box((symbol_len, side_len, qty_len, price_len))
        });
    });

    // MarketData: decode + read all MD entry prices via group iteration.
    group.throughput(Throughput::Bytes(MSG_MARKET_DATA.len() as u64));
    group.bench_function("market_data_iterate_entries", |b| {
        use fix_codec_rs::group;
        let mut dec = Decoder::new();
        b.iter(|| {
            let msg = dec.decode(black_box(MSG_MARKET_DATA)).unwrap();
            let mut count = 0usize;
            for g in msg.groups(&group::MD_ENTRIES) {
                count += g
                    .find(tag::MD_ENTRY_TYPE)
                    .map(|f| f.value.len())
                    .unwrap_or(0);
                count += g.find(tag::MD_ENTRY_PX).map(|f| f.value.len()).unwrap_or(0);
                count += g
                    .find(tag::MD_ENTRY_SIZE)
                    .map(|f| f.value.len())
                    .unwrap_or(0);
            }
            black_box(count)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Encode benchmarks
// ---------------------------------------------------------------------------

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for (name, raw) in [
        ("tiny_1field", MSG_TINY),
        ("order_8fields", MSG_ORDER),
        ("exec_report_12fields", MSG_EXEC),
        ("market_data_20fields", MSG_MARKET_DATA),
    ] {
        group.throughput(Throughput::Bytes(raw.len() as u64));
        group.bench_with_input(BenchmarkId::new("reuse", name), raw, |b, raw| {
            let mut dec = Decoder::new();
            let mut enc = Encoder::new();
            let mut out = Vec::with_capacity(512);
            // Pre-decode so we only measure encode time.
            let msg_buf = raw.to_vec();
            b.iter(|| {
                let msg = dec.decode(black_box(&msg_buf)).unwrap();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Round-trip benchmarks (decode → encode)
// ---------------------------------------------------------------------------

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for (name, raw) in [
        ("order_8fields", MSG_ORDER),
        ("exec_report_12fields", MSG_EXEC),
    ] {
        group.throughput(Throughput::Bytes(raw.len() as u64));
        group.bench_with_input(BenchmarkId::new("decode_encode", name), raw, |b, raw| {
            let mut dec = Decoder::new();
            let mut enc = Encoder::new();
            let mut out = Vec::with_capacity(512);
            b.iter(|| {
                let msg = dec.decode(black_box(raw)).unwrap();
                enc.encode(&msg, &mut out).unwrap();
                black_box(out.len())
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry point
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_decode,
    bench_decode_and_find,
    bench_encode,
    bench_roundtrip,
    bench_sorted_vs_linear,
);
criterion_main!(benches);
