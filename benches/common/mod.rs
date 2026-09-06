//! Shared fixtures for the benchmark binaries.

#![allow(dead_code)] // shared by two bench binaries; not all items are used by each

/// Minimal 4-field message.
pub const SMALL: &[u8] = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";

/// Typical NewOrderSingle-like message: 14 fields total.
pub const TYPICAL: &[u8] = b"8=FIX.4.2\x019=73\x0135=D\x0149=SENDER\x0156=TARGET\x0134=1\x01\
      52=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0140=2\x0144=150.00\x01\
      10=042\x01";

/// 100 fields × ~16-byte values → ~2000-byte body.
pub fn large() -> &'static [u8] {
    static LARGE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    LARGE.get_or_init(|| {
        let mut body = Vec::with_capacity(4096);
        for i in 1u32..=100 {
            body.extend_from_slice(i.to_string().as_bytes());
            body.push(b'=');
            body.extend_from_slice(b"abcdefghijklmnop");
            body.push(b'\x01');
        }

        let mut buf = Vec::with_capacity(body.len() + 32);
        buf.extend_from_slice(b"8=FIX.4.2\x019=");
        buf.extend_from_slice(body.len().to_string().as_bytes());
        buf.push(b'\x01');
        buf.extend_from_slice(&body);
        buf.extend_from_slice(b"10=000\x01");
        buf
    })
}

/// All fixtures as `(name, bytes)`.
pub fn fixtures() -> [(&'static str, &'static [u8]); 3] {
    [
        ("small_4fields", SMALL),
        ("typical_14fields", TYPICAL),
        ("large_100fields", large()),
    ]
}
