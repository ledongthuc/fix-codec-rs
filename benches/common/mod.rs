//! Shared fixtures and "plain `Vec`" mirror implementations.
//!
//! The library's `Decoder`/`Encoder` hard-code `SmallVec` internally. To test
//! whether `SmallVec` actually helps, these mirrors replicate the exact same
//! algorithm but swap the storage for a plain `Vec`. Everything else
//! (memchr scans, `parse_tag`, field iteration) is identical, so any measured
//! difference is attributable to the storage type.
#![allow(dead_code)] // shared by two bench binaries; not all items are used by each

use memchr::memchr;

use smallvec::SmallVec;

use fix_codec_rs::error::FixError;
use fix_codec_rs::message::Message;
use fix_codec_rs::tag::{self, Tag, parse_tag};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Minimal 4-field message (fits well within every inline capacity).
pub const SMALL: &[u8] = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";

/// Typical NewOrderSingle-like message: 14 fields total.
pub const TYPICAL: &[u8] = b"8=FIX.4.2\x019=73\x0135=D\x0149=SENDER\x0156=TARGET\x0134=1\x01\
      52=20240101-12:00:00\x0111=ORD001\x0155=AAPL\x0154=1\x0138=100\x0140=2\x0144=150.00\x01\
      10=042\x01";

/// 100 fields × ~16-byte values → ~2000-byte body.
///
/// Exceeds *both* the decoder's 32-field inline capacity and the encoder's
/// 512-byte inline capacity, so it forces `SmallVec` to spill to the heap.
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

// ---------------------------------------------------------------------------
// `SmallVec`-based mirror of the decoder (same crate, for a fair A/B test)
// ---------------------------------------------------------------------------

/// Same algorithm as the library decoder, but compiled in this crate with
/// `SmallVec` storage. The only difference vs. [`VecDecoder`] is the storage
/// type, so any measured delta isolates `SmallVec` vs `Vec`.
pub struct SmallVecDecoder {
    offsets: SmallVec<[(Tag, u32, u32); 32]>,
}

impl SmallVecDecoder {
    pub fn new() -> Self {
        Self {
            offsets: SmallVec::new(),
        }
    }

    #[inline(never)]
    pub fn decode(&mut self, buf: &[u8]) -> Result<u32, FixError> {
        self.offsets.clear();

        let mut pos = 0;
        let mut sum: u32 = 0;
        while pos < buf.len() {
            let eq = memchr(b'=', &buf[pos..]).ok_or(FixError::IncompleteMessage)? + pos;
            let tag = parse_tag(&buf[pos..eq])?;
            let soh = memchr(b'\x01', &buf[eq + 1..]).ok_or(FixError::IncompleteMessage)? + eq + 1;

            self.offsets.push((tag, (eq + 1) as u32, soh as u32));
            sum = sum.wrapping_add(tag);
            pos = soh + 1;
        }
        Ok(sum)
    }
}

// ---------------------------------------------------------------------------
// `Vec`-based mirror of the decoder
// ---------------------------------------------------------------------------

/// Identical to `fix_codec_rs::decoder::Decoder` except offset storage is a
/// plain `Vec` instead of `SmallVec<[(Tag, u32, u32); 32]>`.
pub struct VecDecoder {
    offsets: Vec<(Tag, u32, u32)>,
}

impl VecDecoder {
    pub fn new() -> Self {
        Self {
            offsets: Vec::new(),
        }
    }

    /// Decode `buf`, returning the sum of parsed tags (a value that forces all
    /// stored offsets to be materialized, so the optimizer cannot elide them).
    ///
    /// `#[inline(never)]` keeps this a real function call so it matches the
    /// library's non-inlined `Decoder::decode` across the crate boundary.
    #[inline(never)]
    pub fn decode(&mut self, buf: &[u8]) -> Result<u32, FixError> {
        self.offsets.clear();

        let mut pos = 0;
        let mut sum: u32 = 0;
        while pos < buf.len() {
            let eq = memchr(b'=', &buf[pos..]).ok_or(FixError::IncompleteMessage)? + pos;
            let tag = parse_tag(&buf[pos..eq])?;
            let soh = memchr(b'\x01', &buf[eq + 1..]).ok_or(FixError::IncompleteMessage)? + eq + 1;

            self.offsets.push((tag, (eq + 1) as u32, soh as u32));
            sum = sum.wrapping_add(tag);
            pos = soh + 1;
        }
        Ok(sum)
    }
}

// ---------------------------------------------------------------------------
// `SmallVec`-based mirror of the encoder (same crate, for a fair A/B test)
// ---------------------------------------------------------------------------

/// Same algorithm as the library encoder, but compiled in this crate with
/// `SmallVec` storage. The only difference vs. [`VecEncoder`] is the storage
/// type.
pub struct SmallVecEncoder {
    body: SmallVec<[u8; 512]>,
}

impl SmallVecEncoder {
    pub fn new() -> Self {
        Self {
            body: SmallVec::new(),
        }
    }

    #[inline(never)]
    pub fn encode(&mut self, msg: &Message<'_>, out: &mut Vec<u8>) -> Result<(), FixError> {
        const DEFAULT_VERSION: &[u8] = b"FIX.4.4";
        let version = msg
            .find(tag::BEGIN_STRING)
            .map(|f| f.value)
            .unwrap_or(DEFAULT_VERSION);

        self.body.clear();
        for f in msg.fields() {
            if f.tag == tag::BEGIN_STRING || f.tag == tag::BODY_LENGTH || f.tag == tag::CHECK_SUM {
                continue;
            }
            let (digits, pos) = u32_to_ascii(f.tag);
            self.body.extend_from_slice(&digits[pos..]);
            self.body.push(b'=');
            self.body.extend_from_slice(f.value);
            self.body.push(b'\x01');
        }

        out.clear();
        out.extend_from_slice(b"8=");
        out.extend_from_slice(version);
        out.push(b'\x01');
        out.extend_from_slice(b"9=");
        let (digits, pos) = u32_to_ascii(self.body.len() as u32);
        out.extend_from_slice(&digits[pos..]);
        out.push(b'\x01');
        out.extend_from_slice(&self.body);

        let checksum = compute_checksum(out);
        out.extend_from_slice(b"10=");
        out.extend_from_slice(&checksum_to_ascii(checksum));
        out.push(b'\x01');

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// `Vec`-based mirror of the encoder
// ---------------------------------------------------------------------------

/// Identical to `fix_codec_rs::encoder::Encoder` except the body scratch buffer
/// is a plain `Vec` instead of `SmallVec<[u8; 512]>`.
pub struct VecEncoder {
    body: Vec<u8>,
}

impl VecEncoder {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    /// `#[inline(never)]` keeps this a real function call so it matches the
    /// library's non-inlined `Encoder::encode` across the crate boundary.
    #[inline(never)]
    pub fn encode(&mut self, msg: &Message<'_>, out: &mut Vec<u8>) -> Result<(), FixError> {
        const DEFAULT_VERSION: &[u8] = b"FIX.4.4";
        let version = msg
            .find(tag::BEGIN_STRING)
            .map(|f| f.value)
            .unwrap_or(DEFAULT_VERSION);

        self.body.clear();
        for f in msg.fields() {
            if f.tag == tag::BEGIN_STRING || f.tag == tag::BODY_LENGTH || f.tag == tag::CHECK_SUM {
                continue;
            }
            let (digits, pos) = u32_to_ascii(f.tag);
            self.body.extend_from_slice(&digits[pos..]);
            self.body.push(b'=');
            self.body.extend_from_slice(f.value);
            self.body.push(b'\x01');
        }

        out.clear();
        out.extend_from_slice(b"8=");
        out.extend_from_slice(version);
        out.push(b'\x01');
        out.extend_from_slice(b"9=");
        let (digits, pos) = u32_to_ascii(self.body.len() as u32);
        out.extend_from_slice(&digits[pos..]);
        out.push(b'\x01');
        out.extend_from_slice(&self.body);

        let checksum = compute_checksum(out);
        out.extend_from_slice(b"10=");
        out.extend_from_slice(&checksum_to_ascii(checksum));
        out.push(b'\x01');

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Small helpers copied from the library (they are `pub(crate)` there)
// ---------------------------------------------------------------------------

#[inline]
fn u32_to_ascii(n: u32) -> ([u8; 10], usize) {
    let mut buf = [0u8; 10];
    let mut pos = 10usize;
    let mut v = n;
    loop {
        pos -= 1;
        buf[pos] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    (buf, pos)
}

#[inline]
fn checksum_to_ascii(n: u8) -> [u8; 3] {
    [b'0' + n / 100, b'0' + (n / 10) % 10, b'0' + n % 10]
}

#[inline]
fn compute_checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}
