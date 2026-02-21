use memchr::memchr;
use smallvec::SmallVec;

use crate::error::FixError;
use crate::field::{FIELD_KEY_VALUE_SEPARATOR, FIELD_SEPARATOR};
use crate::message::Message;
use crate::tag::{parse_tag, Tag};

/// Default inline capacity: covers ~95% of FIX messages without heap spill.
const DEFAULT_CAPACITY: usize = 32;

/// A reusable FIX message decoder.
///
/// Owns a `SmallVec` buffer that is allocated once (at startup or first use)
/// and reused across every `decode` call — zero allocation per message on the
/// hot path.
///
/// Stores `(tag, value_start, value_end)` byte offsets rather than slices,
/// eliminating all unsafe lifetime transmutes while preserving zero-allocation
/// and zero-copy semantics.
///
/// # Example
/// ```ignore
/// let mut decoder = Decoder::new();
///
/// loop {
///     let msg = decoder.decode(buf)?;  // zero allocation after first call
///     process(msg);
///     // msg dropped here — decoder buffer ready for next call
/// }
/// ```
pub struct Decoder {
    /// Stores (tag, value_start_offset, value_end_offset) per field.
    /// clear() at the start of each decode call preserves allocated capacity —
    /// no free/malloc on the hot path.
    offsets: SmallVec<[(Tag, u32, u32); DEFAULT_CAPACITY]>,
}

impl Decoder {
    /// Create a new decoder with a default inline capacity of 32 fields.
    pub fn new() -> Self {
        Self {
            offsets: SmallVec::new(),
        }
    }

    /// Create a new decoder pre-allocated for `capacity` fields.
    /// Use this when messages consistently exceed 32 fields (e.g. MarketData).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            offsets: SmallVec::with_capacity(capacity),
        }
    }

    /// Decode a raw FIX byte buffer into a `Message`.
    ///
    /// Clears and reuses the internal offset buffer — zero allocation per call
    /// after the first. The returned `Message<'a>` borrows both from `self`
    /// (the offset slice) and from `buf` (the raw bytes). Drop `Message`
    /// before calling `decode` again.
    ///
    /// # Errors
    /// - `FixError::IncompleteMessage` — the buffer contains a partial field
    ///   (no `=` or no SOH delimiter found); buffer more bytes before retrying.
    /// - `FixError::InvalidTag` — a tag contained non-digit bytes or overflowed `u32`.
    pub fn decode<'a>(&'a mut self, buf: &'a [u8]) -> Result<Message<'a>, FixError> {
        // clear() keeps existing capacity — no allocator call on hot path
        self.offsets.clear();

        let mut pos = 0;
        while pos < buf.len() {
            // SIMD scan for '=' — delimits tag from value
            let eq_pos = memchr(FIELD_KEY_VALUE_SEPARATOR, &buf[pos..])
                .ok_or(FixError::IncompleteMessage)?
                + pos;

            let tag = parse_tag(&buf[pos..eq_pos])?;

            // SIMD scan for SOH (0x01) — delimits end of value
            let soh_pos = memchr(FIELD_SEPARATOR, &buf[eq_pos + 1..])
                .ok_or(FixError::IncompleteMessage)?
                + eq_pos + 1;

            // Store byte offsets — plain integers, no lifetimes, no unsafe needed.
            self.offsets.push((tag, (eq_pos + 1) as u32, soh_pos as u32));

            pos = soh_pos + 1;
        }

        // Both borrows are genuinely 'a: offsets from &'a mut self, buf from
        // &'a [u8]. No transmutes, no unsafe.
        Ok(Message {
            buf,
            offsets: self.offsets.as_slice(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FixError;

    // -------------------------------------------------------------------------
    // Group 1 — Happy path
    // -------------------------------------------------------------------------

    #[test]
    fn happy_empty_buffer() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"").unwrap();
        assert_eq!(msg.len(), 0);
        assert!(msg.is_empty());
    }

    #[test]
    fn happy_single_field() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x01").unwrap();
        assert_eq!(msg.len(), 1);
        let f = msg.field(0);
        assert_eq!(f.tag, 8);
        assert_eq!(f.value, b"FIX.4.2");
    }

    #[test]
    fn happy_multiple_fields() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x0149=SENDER\x01").unwrap();
        assert_eq!(msg.len(), 3);
        let f0 = msg.field(0);
        assert_eq!(f0.tag, 8);
        assert_eq!(f0.value, b"FIX.4.2");
        let f1 = msg.field(1);
        assert_eq!(f1.tag, 35);
        assert_eq!(f1.value, b"D");
        let f2 = msg.field(2);
        assert_eq!(f2.tag, 49);
        assert_eq!(f2.value, b"SENDER");
    }

    #[test]
    fn happy_empty_value() {
        // A field whose value is an empty byte slice: "35=\x01"
        let mut dec = Decoder::new();
        let msg = dec.decode(b"35=\x01").unwrap();
        assert_eq!(msg.len(), 1);
        let f = msg.field(0);
        assert_eq!(f.tag, 35);
        assert_eq!(f.value, b"");
    }

    #[test]
    fn happy_value_containing_equals() {
        // '=' inside a value must not confuse the next field's tag scan
        // because we only scan for '=' starting from `pos` (start of tag).
        let mut dec = Decoder::new();
        let msg = dec.decode(b"58=price=100\x0135=D\x01").unwrap();
        assert_eq!(msg.len(), 2);
        assert_eq!(msg.field(0).tag, 58);
        assert_eq!(msg.field(0).value, b"price=100");
        assert_eq!(msg.field(1).tag, 35);
        assert_eq!(msg.field(1).value, b"D");
    }

    #[test]
    fn happy_binary_value() {
        // Values may contain arbitrary bytes (e.g. RawData tag 96)
        let mut dec = Decoder::new();
        let msg = dec.decode(b"95=3\x0196=\x02\x03\x04\x01").unwrap();
        assert_eq!(msg.len(), 2);
        assert_eq!(msg.field(1).tag, 96);
        assert_eq!(msg.field(1).value, &[0x02u8, 0x03, 0x04]);
    }

    #[test]
    fn happy_exactly_32_fields() {
        // 32 fields = inline SmallVec capacity boundary, no heap spill
        let mut dec = Decoder::new();
        let mut buf = Vec::new();
        for i in 1u32..=32 {
            buf.extend_from_slice(format!("{}=v\x01", i).as_bytes());
        }
        let msg = dec.decode(&buf).unwrap();
        assert_eq!(msg.len(), 32);
        for i in 0..32 {
            assert_eq!(msg.field(i).tag, (i + 1) as u32);
            assert_eq!(msg.field(i).value, b"v");
        }
    }

    #[test]
    fn happy_33_fields_spills_to_heap() {
        // 33 fields forces SmallVec past inline capacity — must still be correct
        let mut dec = Decoder::new();
        let mut buf = Vec::new();
        for i in 1u32..=33 {
            buf.extend_from_slice(format!("{}=v\x01", i).as_bytes());
        }
        let msg = dec.decode(&buf).unwrap();
        assert_eq!(msg.len(), 33);
        assert_eq!(msg.field(32).tag, 33);
    }

    // -------------------------------------------------------------------------
    // Group 2 — Decoder reuse
    // -------------------------------------------------------------------------

    #[test]
    fn reuse_decode_twice() {
        let mut dec = Decoder::new();
        {
            let msg = dec.decode(b"8=FIX.4.2\x01").unwrap();
            assert_eq!(msg.field(0).tag, 8);
        } // msg dropped, borrow released
        let msg2 = dec.decode(b"35=D\x01").unwrap();
        assert_eq!(msg2.field(0).tag, 35);
        assert_eq!(msg2.field(0).value, b"D");
    }

    #[test]
    fn reuse_large_then_small() {
        // After a 33-field msg causes heap spill, a 1-field msg still works
        let mut dec = Decoder::new();
        let mut big_buf = Vec::new();
        for i in 1u32..=33 {
            big_buf.extend_from_slice(format!("{}=v\x01", i).as_bytes());
        }
        {
            let msg = dec.decode(&big_buf).unwrap();
            assert_eq!(msg.len(), 33);
        }
        let msg2 = dec.decode(b"8=FIX.4.2\x01").unwrap();
        assert_eq!(msg2.len(), 1);
        assert_eq!(msg2.field(0).tag, 8);
    }

    #[test]
    fn reuse_many_iterations_stable() {
        let mut dec = Decoder::new();
        let buf = b"8=FIX.4.2\x0135=D\x0149=SENDER\x01";
        for _ in 0..1_000 {
            let msg = dec.decode(buf).unwrap();
            assert_eq!(msg.len(), 3);
            assert_eq!(msg.field(0).tag, 8);
        }
    }

    // -------------------------------------------------------------------------
    // Group 3 — IncompleteMessage (partial TCP frame)
    // -------------------------------------------------------------------------

    #[test]
    fn incomplete_tag_only_no_equals() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"8").unwrap_err(),
            FixError::IncompleteMessage
        ));
    }

    #[test]
    fn incomplete_tag_equals_value_no_soh() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"8=FIX.4.2").unwrap_err(),
            FixError::IncompleteMessage
        ));
    }

    #[test]
    fn incomplete_first_field_ok_second_tag_no_equals() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"8=FIX.4.2\x0135").unwrap_err(),
            FixError::IncompleteMessage
        ));
    }

    #[test]
    fn incomplete_second_field_value_no_soh() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"8=FIX.4.2\x0135=D").unwrap_err(),
            FixError::IncompleteMessage
        ));
    }

    #[test]
    fn incomplete_only_soh_byte() {
        // b"\x01" — SOH at pos=0, no '=' found before it → IncompleteMessage
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"\x01").unwrap_err(),
            FixError::IncompleteMessage
        ));
    }

    // -------------------------------------------------------------------------
    // Group 4 — InvalidTag errors
    // -------------------------------------------------------------------------

    #[test]
    fn invalid_tag_empty_tag_leading_equals() {
        // buf starts with '=' → tag slice is empty → InvalidTag
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"=val\x01").unwrap_err(),
            FixError::InvalidTag
        ));
    }

    #[test]
    fn invalid_tag_non_digit_byte() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"8X=val\x01").unwrap_err(),
            FixError::InvalidTag
        ));
    }

    #[test]
    fn invalid_tag_overflow_ten_nines() {
        // 9999999999 > u32::MAX
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"9999999999=val\x01").unwrap_err(),
            FixError::InvalidTag
        ));
    }

    #[test]
    fn invalid_tag_one_past_u32_max() {
        // 4294967296 = u32::MAX + 1
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"4294967296=val\x01").unwrap_err(),
            FixError::InvalidTag
        ));
    }

    #[test]
    fn invalid_tag_leading_space() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b" 8=val\x01").unwrap_err(),
            FixError::InvalidTag
        ));
    }

    #[test]
    fn invalid_tag_trailing_space() {
        let mut dec = Decoder::new();
        assert!(matches!(
            dec.decode(b"8 =val\x01").unwrap_err(),
            FixError::InvalidTag
        ));
    }

    // -------------------------------------------------------------------------
    // Group 5 — Edge cases in value scanning
    // -------------------------------------------------------------------------

    #[test]
    fn edge_single_byte_value() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=X\x01").unwrap();
        assert_eq!(msg.field(0).value, b"X");
    }

    #[test]
    fn edge_value_starts_with_soh() {
        // "8=\x01val\x01" — memchr(SOH) finds the first \x01 immediately after '=',
        // so value = b"" and pos advances to 'v'. "val" then has no '=' → IncompleteMessage.
        let mut dec = Decoder::new();
        let err = dec.decode(b"8=\x01val\x01").unwrap_err();
        assert!(matches!(err, FixError::IncompleteMessage));
    }

    #[test]
    fn edge_value_then_bare_soh() {
        // "8=A\x01B\x01" — first field ok (tag=8, value="A"),
        // then "B\x01" has no '=' → IncompleteMessage.
        let mut dec = Decoder::new();
        let err = dec.decode(b"8=A\x01B\x01").unwrap_err();
        assert!(matches!(err, FixError::IncompleteMessage));
    }

    #[test]
    fn edge_back_to_back_soh() {
        // "8=\x01\x01" — first field: value=b"", pos advances to second \x01.
        // Second byte \x01 has no '=' → IncompleteMessage.
        let mut dec = Decoder::new();
        let err = dec.decode(b"8=\x01\x01").unwrap_err();
        assert!(matches!(err, FixError::IncompleteMessage));
    }

    #[test]
    fn edge_tag_zero() {
        // Tag 0 is not a valid FIX tag but the decoder is not responsible for
        // semantic validation — it should parse it as tag=0.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"0=val\x01").unwrap();
        assert_eq!(msg.field(0).tag, 0);
        assert_eq!(msg.field(0).value, b"val");
    }

    #[test]
    fn edge_tag_u32_max() {
        // 4294967295 == u32::MAX — within range, should succeed.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"4294967295=val\x01").unwrap();
        assert_eq!(msg.field(0).tag, u32::MAX);
        assert_eq!(msg.field(0).value, b"val");
    }

    // -------------------------------------------------------------------------
    // Group 6 — pos advancement correctness
    // -------------------------------------------------------------------------

    #[test]
    fn pos_long_value_next_field_correct() {
        // First value is 1000 bytes; verify second field parses correctly.
        let mut dec = Decoder::new();
        let long_val = vec![b'A'; 1000];
        let mut buf = Vec::new();
        buf.extend_from_slice(b"96=");
        buf.extend_from_slice(&long_val);
        buf.push(0x01);
        buf.extend_from_slice(b"35=D\x01");
        let msg = dec.decode(&buf).unwrap();
        assert_eq!(msg.len(), 2);
        assert_eq!(msg.field(0).tag, 96);
        assert_eq!(msg.field(0).value.len(), 1000);
        assert_eq!(msg.field(1).tag, 35);
        assert_eq!(msg.field(1).value, b"D");
    }

    #[test]
    fn pos_equals_in_first_value_does_not_confuse_second_tag_scan() {
        // The '=' inside the first value must not be picked up as the
        // delimiter for the second field's tag.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"58=a=b=c\x0135=D\x01").unwrap();
        assert_eq!(msg.len(), 2);
        assert_eq!(msg.field(0).tag, 58);
        assert_eq!(msg.field(0).value, b"a=b=c");
        assert_eq!(msg.field(1).tag, 35);
        assert_eq!(msg.field(1).value, b"D");
    }

    #[test]
    fn pos_message_ending_exactly_at_soh() {
        // Last byte is SOH; pos = soh_pos + 1 == buf.len() → loop exits normally.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x01").unwrap();
        assert_eq!(msg.len(), 2);
        assert_eq!(msg.field(1).value, b"D");
    }

    // -------------------------------------------------------------------------
    // Group 7 — with_capacity constructor
    // -------------------------------------------------------------------------

    #[test]
    fn with_capacity_exact_fit() {
        let mut dec = Decoder::with_capacity(4);
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x0149=A\x0156=B\x01").unwrap();
        assert_eq!(msg.len(), 4);
        assert_eq!(msg.field(3).tag, 56);
        assert_eq!(msg.field(3).value, b"B");
    }

    #[test]
    fn with_capacity_one_spills_to_heap() {
        // Pre-allocate 1, decode 33 fields — SmallVec must spill correctly.
        let mut dec = Decoder::with_capacity(1);
        let mut buf = Vec::new();
        for i in 1u32..=33 {
            buf.extend_from_slice(format!("{}=v\x01", i).as_bytes());
        }
        let msg = dec.decode(&buf).unwrap();
        assert_eq!(msg.len(), 33);
        assert_eq!(msg.field(32).tag, 33);
    }
}
