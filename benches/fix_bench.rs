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

/// FIXT.1.1 (FIX 5.0) message with a nested `RootParties` → `RootPartySubIDs`
/// group. Body length and checksum are pre-computed and valid.
const MSG_FIX50: &[u8] = b"8=FIXT.1.1\x019=105\x0135=AB\x0149=S\x0156=T\x0134=1\x01\
      52=20240101-12:00:00\x011128=7\x011116=1\x011117=ROOT1\x011118=D\x011119=1\x01\
      1120=1\x011121=SUB1\x011122=1\x0110=109\x01";

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
        ("fix50_root_parties", MSG_FIX50),
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
// Decode + field access benchmarks
// ---------------------------------------------------------------------------

fn bench_decode_and_find(c: &mut Criterion) {
    use fix_codec_rs::tag;

    let mut group = c.benchmark_group("decode_and_find");

    // Decode an order message, then look up key fields by tag.
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
// Lazy field iteration (decode_fields)
// ---------------------------------------------------------------------------

fn bench_decode_fields(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_fields");

    for (name, msg) in [
        ("tiny_1field", MSG_TINY),
        ("order_8fields", MSG_ORDER),
        ("exec_report_12fields", MSG_EXEC),
        ("market_data_20fields", MSG_MARKET_DATA),
    ] {
        group.throughput(Throughput::Bytes(msg.len() as u64));
        group.bench_with_input(BenchmarkId::new("iterate", name), msg, |b, msg| {
            let dec = Decoder::new();
            b.iter(|| {
                let mut total = 0usize;
                for field in dec.decode_fields(black_box(msg)) {
                    total += field.unwrap().value.len();
                }
                black_box(total)
            });
        });
    }

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
        ("fix50_root_parties", MSG_FIX50),
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
// FIX 5.0 group iteration benchmark
// ---------------------------------------------------------------------------

fn bench_fix50_groups(c: &mut Criterion) {
    use fix_codec_rs::group;
    use fix_codec_rs::tag;

    let mut group = c.benchmark_group("fix50_groups");
    group.throughput(Throughput::Bytes(MSG_FIX50.len() as u64));
    group.bench_function("root_parties_nested_iterate_find", |b| {
        let mut dec = Decoder::new();
        b.iter(|| {
            let msg = dec.decode(black_box(MSG_FIX50)).unwrap();
            let mut total = msg
                .find(tag::APPL_VER_ID)
                .map(|f| f.value.len())
                .unwrap_or(0);
            for (spec, instances) in msg.all_groups() {
                if spec.count_tag != tag::NO_ROOT_PARTY_IDS {
                    continue;
                }
                for party in instances {
                    total += party
                        .find(tag::ROOT_PARTY_ID)
                        .map(|f| f.value.len())
                        .unwrap_or(0);
                    for sub in party.groups(&group::ROOT_PARTY_SUB_IDS) {
                        total += sub
                            .find(tag::ROOT_PARTY_SUB_ID)
                            .map(|f| f.value.len())
                            .unwrap_or(0);
                    }
                }
            }
            black_box(total)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry point
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_decode,
    bench_decode_fields,
    bench_decode_and_find,
    bench_encode,
    bench_roundtrip,
    bench_fix50_groups,
);
criterion_main!(benches);
