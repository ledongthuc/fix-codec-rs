use memchr::memchr;

use crate::error::FixError;
use crate::field::{FIELD_KEY_VALUE_SEPARATOR, FIELD_SEPARATOR, Field};
use crate::message::Message;
use crate::tag::{Tag, parse_tag};

/// A reusable FIX message decoder.
///
/// Owns one reusable buffer, allocated once (at startup or first use) and
/// reused across every `decode` call:
///
/// - `offsets` — a `Vec<(Tag, value_start, value_end)>`, cleared per decode and
///   reused without reallocating (capacity preserved).
///
/// `find`/`find_all` are linear scans over `offsets`; there is no tag index.
///
/// There are two entry points:
///
/// - [`Decoder::decode`] — parses the full message, returning a [`Message`]
///   that supports `find`/`find_all`, field iteration, groups, and validation.
/// - [`Decoder::decode_fields`] — a lazy iterator that parses one field per
///   `next()` from a complete buffer, storing no offsets. It is
///   allocation-free and is *not* resumable/incremental.
///
/// # Example
/// ```ignore
/// let mut decoder = Decoder::new();
///
/// loop {
///     let msg = decoder.decode(buf)?;
///     process(msg);
///     // msg dropped here — decoder buffers ready for next call
/// }
/// ```
pub struct Decoder {
    /// Stores (tag, value_start_offset, value_end_offset) per field.
    /// clear() at the start of each decode call preserves allocated capacity —
    /// no free/malloc for offsets on the hot path.
    offsets: Vec<(Tag, u32, u32)>,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    /// Create a new decoder with an empty field-offset buffer.
    pub fn new() -> Self {
        Self {
            offsets: Vec::new(),
        }
    }

    /// Create a new decoder pre-allocated for `capacity` fields.
    /// Use this to avoid offset reallocations when messages consistently contain
    /// many fields (e.g. MarketData).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            offsets: Vec::with_capacity(capacity),
        }
    }

    /// Decode a raw FIX byte buffer into a `Message`.
    ///
    /// Parses all fields into the reusable offset buffer. Allocation-free in
    /// steady state (the offset `Vec` is cleared and reused). The returned
    /// [`Message`] borrows from `self` (offset slice) and from `buf` (raw
    /// bytes). Drop `Message` before calling `decode` again.
    ///
    /// # Errors
    /// - `FixError::IncompleteMessage` — the buffer contains a partial field
    ///   (no `=` or no SOH delimiter found); buffer more bytes before retrying.
    /// - `FixError::InvalidTag` — a tag contained non-digit bytes or overflowed `u32`.
    pub fn decode<'a>(&'a mut self, buf: &'a [u8]) -> Result<Message<'a>, FixError> {
        self.parse_offsets(buf)?;
        Ok(Message::new(buf, self.offsets.as_slice()))
    }

    /// Return a lazy iterator over the fields of a complete FIX message.
    ///
    /// Each [`FieldsIter::next`] parses one field from `buf`; no offsets and no
    /// index are stored, and only `buf` is borrowed. This is *not*
    /// resumable/incremental — `buf` must be a complete message. On a parse
    /// error the offending item is yielded as `Err(..)` and iteration stops:
    /// subsequent `next()` returns `None`.
    ///
    /// Empty buffers yield an empty iterator (no error).
    #[inline]
    pub fn decode_fields<'a>(&self, buf: &'a [u8]) -> FieldsIter<'a> {
        FieldsIter { buf, pos: 0 }
    }

    /// Parse every field in `buf` into `self.offsets`.
    fn parse_offsets(&mut self, buf: &[u8]) -> Result<(), FixError> {
        // clear() keeps existing capacity — no allocator call for offsets on
        // the hot path.
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
                + eq_pos
                + 1;

            // Store byte offsets — plain integers, no lifetimes, no unsafe needed.
            self.offsets
                .push((tag, (eq_pos + 1) as u32, soh_pos as u32));

            pos = soh_pos + 1;
        }

        Ok(())
    }
}

/// A lazy iterator over the fields of a complete FIX message.
///
/// Produced by [`Decoder::decode_fields`]. Each `next()` parses one field on
/// demand from the borrowed buffer. It stores no offsets and no index, and is
/// not resumable — `buf` must be a complete message. After the first parse
/// error the iterator stops permanently (subsequent `next()` returns `None`).
pub struct FieldsIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for FieldsIter<'a> {
    type Item = Result<Field<'a>, FixError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buf.len() {
            return None;
        }

        // Scan for '=' — delimits tag from value.
        let eq_pos = match memchr(FIELD_KEY_VALUE_SEPARATOR, &self.buf[self.pos..]) {
            Some(off) => off + self.pos,
            None => {
                self.pos = self.buf.len();
                return Some(Err(FixError::IncompleteMessage));
            }
        };

        // Parse the tag; on error, stop the iterator and surface the error once.
        let tag = match parse_tag(&self.buf[self.pos..eq_pos]) {
            Ok(tag) => tag,
            Err(err) => {
                self.pos = self.buf.len();
                return Some(Err(err));
            }
        };

        // Scan for SOH (0x01) — delimits end of value.
        let soh_pos = match memchr(FIELD_SEPARATOR, &self.buf[eq_pos + 1..]) {
            Some(off) => off + eq_pos + 1,
            None => {
                self.pos = self.buf.len();
                return Some(Err(FixError::IncompleteMessage));
            }
        };

        let field = Field {
            tag,
            value: &self.buf[eq_pos + 1..soh_pos],
        };
        self.pos = soh_pos + 1;
        Some(Ok(field))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FixError;
    use crate::group;

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
        // 32 fields decode correctly.
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
    fn happy_33_fields_grows_buffer() {
        // 33 fields grows the internal Vec — must still be correct
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
        // After a 33-field msg, a 1-field msg still works
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
        let msg = dec
            .decode(b"8=FIX.4.2\x0135=D\x0149=A\x0156=B\x01")
            .unwrap();
        assert_eq!(msg.len(), 4);
        assert_eq!(msg.field(3).tag, 56);
        assert_eq!(msg.field(3).value, b"B");
    }

    #[test]
    fn with_capacity_one_grows() {
        // Pre-allocate 1, decode 33 fields — Vec must grow correctly.
        let mut dec = Decoder::with_capacity(1);
        let mut buf = Vec::new();
        for i in 1u32..=33 {
            buf.extend_from_slice(format!("{}=v\x01", i).as_bytes());
        }
        let msg = dec.decode(&buf).unwrap();
        assert_eq!(msg.len(), 33);
        assert_eq!(msg.field(32).tag, 33);
    }

    // -------------------------------------------------------------------------
    // Group 8 — Repeating groups (successful decode + group navigation)
    // -------------------------------------------------------------------------

    #[test]
    fn group_single_misc_fee() {
        // Allocation message with one MiscFee instance: NO_MISC_FEES=1 followed by
        // MiscFeeAmt / MiscFeeCurr / MiscFeeType.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=50\x0135=J\x01136=1\x01137=10.50\x01138=USD\x01139=4\x01")
            .unwrap();
        assert_eq!(msg.len(), 7);

        let fees: Vec<_> = msg.groups(&group::MISC_FEES).collect();
        assert_eq!(fees.len(), 1);
        assert_eq!(
            fees[0].find(crate::tag::MISC_FEE_AMT).unwrap().value,
            b"10.50"
        );
        assert_eq!(
            fees[0].find(crate::tag::MISC_FEE_CURR).unwrap().value,
            b"USD"
        );
        assert_eq!(fees[0].find(crate::tag::MISC_FEE_TYPE).unwrap().value, b"4");
    }

    #[test]
    fn group_multiple_misc_fees() {
        // Two MiscFee instances — delimiter tag (137) reappearance splits them.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(
                b"35=J\x01136=2\x01137=5.00\x01138=USD\x01139=1\x01137=2.50\x01138=EUR\x01139=2\x01",
            )
            .unwrap();
        assert_eq!(msg.len(), 8);

        let fees: Vec<_> = msg.groups(&group::MISC_FEES).collect();
        assert_eq!(fees.len(), 2);

        assert_eq!(
            fees[0].find(crate::tag::MISC_FEE_AMT).unwrap().value,
            b"5.00"
        );
        assert_eq!(
            fees[0].find(crate::tag::MISC_FEE_CURR).unwrap().value,
            b"USD"
        );
        assert_eq!(fees[0].find(crate::tag::MISC_FEE_TYPE).unwrap().value, b"1");

        assert_eq!(
            fees[1].find(crate::tag::MISC_FEE_AMT).unwrap().value,
            b"2.50"
        );
        assert_eq!(
            fees[1].find(crate::tag::MISC_FEE_CURR).unwrap().value,
            b"EUR"
        );
        assert_eq!(fees[1].find(crate::tag::MISC_FEE_TYPE).unwrap().value, b"2");
    }

    #[test]
    fn group_md_entries_bid_and_offer() {
        // MarketDataSnapshotFullRefresh with two MDEntry instances (bid + offer).
        // NO_MD_ENTRIES=268, delimiter=MDEntryType=269.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(
                b"35=W\x0149=SENDER\x0156=TARGET\x01268=2\x01\
                269=0\x01270=99.50\x01271=1000\x01\
                269=1\x01270=99.75\x01271=500\x01",
            )
            .unwrap();
        assert_eq!(msg.len(), 10);

        let entries: Vec<_> = msg.groups(&group::MD_ENTRIES).collect();
        assert_eq!(entries.len(), 2);

        // Bid (MDEntryType=0)
        assert_eq!(
            entries[0].find(crate::tag::MD_ENTRY_TYPE).unwrap().value,
            b"0"
        );
        assert_eq!(
            entries[0].find(crate::tag::MD_ENTRY_PX).unwrap().value,
            b"99.50"
        );
        assert_eq!(
            entries[0].find(crate::tag::MD_ENTRY_SIZE).unwrap().value,
            b"1000"
        );

        // Offer (MDEntryType=1)
        assert_eq!(
            entries[1].find(crate::tag::MD_ENTRY_TYPE).unwrap().value,
            b"1"
        );
        assert_eq!(
            entries[1].find(crate::tag::MD_ENTRY_PX).unwrap().value,
            b"99.75"
        );
        assert_eq!(
            entries[1].find(crate::tag::MD_ENTRY_SIZE).unwrap().value,
            b"500"
        );
    }

    #[test]
    fn group_routing_ids_two_routes() {
        // Header with NO_ROUTING_IDS=2; RoutingType=216 is the delimiter.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"35=D\x01215=2\x01216=1\x01217=ROUTE_A\x01216=2\x01217=ROUTE_B\x01")
            .unwrap();
        assert_eq!(msg.len(), 6);

        let routes: Vec<_> = msg.groups(&group::ROUTING_IDS).collect();
        assert_eq!(routes.len(), 2);
        assert_eq!(
            routes[0].find(crate::tag::ROUTING_TYPE).unwrap().value,
            b"1"
        );
        assert_eq!(
            routes[0].find(crate::tag::ROUTING_ID).unwrap().value,
            b"ROUTE_A"
        );
        assert_eq!(
            routes[1].find(crate::tag::ROUTING_TYPE).unwrap().value,
            b"2"
        );
        assert_eq!(
            routes[1].find(crate::tag::ROUTING_ID).unwrap().value,
            b"ROUTE_B"
        );
    }

    #[test]
    fn group_count_zero_yields_no_instances() {
        // NO_MISC_FEES=0 — iterator must yield nothing even though count tag present.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"35=J\x01136=0\x0158=no fees\x01").unwrap();
        assert_eq!(msg.len(), 3);
        assert_eq!(msg.groups(&group::MISC_FEES).count(), 0);
    }

    #[test]
    fn group_count_tag_absent_yields_no_instances() {
        // Message has no NO_MISC_FEES tag at all.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x0149=SENDER\x01").unwrap();
        assert_eq!(msg.groups(&group::MISC_FEES).count(), 0);
    }

    #[test]
    fn group_fields_after_group_still_accessible() {
        // Fields that follow a group in the flat message must remain accessible
        // via Message::field() / Message::find() as usual.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"35=J\x01136=1\x01137=3.00\x01138=USD\x01139=1\x0110=200\x01")
            .unwrap();
        assert_eq!(msg.len(), 6);

        // Group navigation works.
        let fee = msg.groups(&group::MISC_FEES).next().unwrap();
        assert_eq!(fee.find(crate::tag::MISC_FEE_AMT).unwrap().value, b"3.00");

        // CheckSum field after the group is accessible normally.
        assert_eq!(msg.find(crate::tag::CHECK_SUM).unwrap().value, b"200");
    }

    // -------------------------------------------------------------------------
    // Group 9 — all_groups()
    // -------------------------------------------------------------------------

    #[test]
    fn all_groups_empty_message_yields_nothing() {
        // No fields at all — no groups present.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"").unwrap();
        assert_eq!(msg.all_groups().count(), 0);
    }

    #[test]
    fn all_groups_no_group_tags_yields_nothing() {
        // Plain message with no NO_* tags.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x0149=SENDER\x01").unwrap();
        assert_eq!(msg.all_groups().count(), 0);
    }

    #[test]
    fn all_groups_single_group_present() {
        // Message contains only NO_MISC_FEES — all_groups must yield exactly one entry.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x0135=J\x01136=1\x01137=7.00\x01138=USD\x01139=2\x01")
            .unwrap();

        let mut iter = msg.all_groups();
        let (spec, mut instances) = iter.next().expect("expected one group");
        assert_eq!(spec.count_tag, crate::tag::NO_MISC_FEES);

        let g = instances.next().unwrap();
        assert_eq!(g.find(crate::tag::MISC_FEE_AMT).unwrap().value, b"7.00");
        assert_eq!(g.find(crate::tag::MISC_FEE_CURR).unwrap().value, b"USD");
        assert!(iter.next().is_none());
    }

    #[test]
    fn all_groups_two_different_groups_present() {
        // Message contains both NO_MISC_FEES and NO_ROUTING_IDS.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(
                b"8=FIX.4.2\x0135=D\x01215=2\x01216=1\x01217=ROUTE_A\x01216=2\x01217=ROUTE_B\x01\
                  136=1\x01137=1.00\x01138=USD\x01139=3\x01",
            )
            .unwrap();

        let found: Vec<_> = msg.all_groups().map(|(spec, _)| spec.count_tag).collect();
        // Both group count tags must appear, in FIX42_GROUPS order.
        assert!(found.contains(&crate::tag::NO_MISC_FEES));
        assert!(found.contains(&crate::tag::NO_ROUTING_IDS));
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn all_groups_count_zero_skipped() {
        // NO_MISC_FEES=0 must not appear in all_groups output.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=J\x01136=0\x01").unwrap();
        assert_eq!(msg.all_groups().count(), 0);
    }

    #[test]
    fn all_groups_instances_are_correct() {
        // Verify that instances returned through all_groups() have the right field values.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x0135=W\x01268=2\x01269=0\x01270=50.00\x01269=1\x01270=50.25\x01")
            .unwrap();

        let mut all = msg.all_groups();
        let (spec, instances) = all.next().expect("expected MD_ENTRIES group");
        assert_eq!(spec.count_tag, crate::tag::NO_MD_ENTRIES);

        let entries: Vec<_> = instances.collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].find(crate::tag::MD_ENTRY_TYPE).unwrap().value,
            b"0"
        );
        assert_eq!(
            entries[0].find(crate::tag::MD_ENTRY_PX).unwrap().value,
            b"50.00"
        );
        assert_eq!(
            entries[1].find(crate::tag::MD_ENTRY_TYPE).unwrap().value,
            b"1"
        );
        assert_eq!(
            entries[1].find(crate::tag::MD_ENTRY_PX).unwrap().value,
            b"50.25"
        );

        assert!(all.next().is_none());
    }

    // -------------------------------------------------------------------------
    // Group 10 — validate_body_length() and validate_checksum()
    // -------------------------------------------------------------------------
    //
    // All expected checksums and body lengths are pre-computed and verified with:
    //   sum(bytes before "10=") % 256  and  len(bytes between "9=…\x01" and "10=")

    #[test]
    fn validate_body_length_correct() {
        // "8=FIX.4.2\x019=5\x0135=D\x0110=181\x01"
        // Body = "35=D\x01" = 5 bytes. Declared 9=5. Should pass.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01")
            .unwrap();
        assert!(msg.validate_body_length().is_ok());
    }

    #[test]
    fn validate_body_length_wrong_value() {
        // Declared 9=99 but actual body is 5 bytes. Should fail.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=99\x0135=D\x0110=000\x01")
            .unwrap();
        assert!(matches!(
            msg.validate_body_length().unwrap_err(),
            FixError::InvalidBodyLength
        ));
    }

    #[test]
    fn validate_body_length_multi_field_body() {
        // "8=FIX.4.2\x019=25\x0135=D\x0149=SENDER\x0156=TARGET\x0110=195\x01"
        // Body = "35=D\x0149=SENDER\x0156=TARGET\x01" = 25 bytes. Declared 9=25.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=25\x0135=D\x0149=SENDER\x0156=TARGET\x0110=195\x01")
            .unwrap();
        assert!(msg.validate_body_length().is_ok());
    }

    #[test]
    fn validate_body_length_tag9_missing() {
        // Message with fewer than 3 fields — no room for tag 8, 9, and 10.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x01").unwrap();
        assert!(matches!(
            msg.validate_body_length().unwrap_err(),
            FixError::InvalidBodyLength
        ));
    }

    #[test]
    fn validate_body_length_tag9_not_second_field() {
        // Tag 9 is not in position 1 — invalid message structure.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x0135=D\x019=5\x0110=000\x01")
            .unwrap();
        assert!(matches!(
            msg.validate_body_length().unwrap_err(),
            FixError::InvalidBodyLength
        ));
    }

    #[test]
    fn validate_body_length_tag10_not_last_field() {
        // Tag 10 is not the last field — invalid message structure.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=5\x0110=000\x0135=D\x01")
            .unwrap();
        assert!(matches!(
            msg.validate_body_length().unwrap_err(),
            FixError::InvalidBodyLength
        ));
    }

    #[test]
    fn validate_checksum_correct() {
        // "8=FIX.4.2\x019=5\x0135=D\x0110=181\x01"
        // sum("8=FIX.4.2\x019=5\x0135=D\x01") % 256 = 181
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01")
            .unwrap();
        assert!(msg.validate_checksum().is_ok());
    }

    #[test]
    fn validate_checksum_wrong_value() {
        // Correct message bytes but checksum declared as 000 instead of 181.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=5\x0135=D\x0110=000\x01")
            .unwrap();
        assert!(matches!(
            msg.validate_checksum().unwrap_err(),
            FixError::InvalidCheckSum
        ));
    }

    #[test]
    fn validate_checksum_multi_field_body() {
        // "8=FIX.4.2\x019=25\x0135=D\x0149=SENDER\x0156=TARGET\x0110=195\x01"
        // sum of bytes before "10=" = 195
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=25\x0135=D\x0149=SENDER\x0156=TARGET\x0110=195\x01")
            .unwrap();
        assert!(msg.validate_checksum().is_ok());
    }

    #[test]
    fn validate_checksum_tag10_missing() {
        // No tag 10 as last field — should fail.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0135=D\x01").unwrap();
        assert!(matches!(
            msg.validate_checksum().unwrap_err(),
            FixError::InvalidCheckSum
        ));
    }

    #[test]
    fn validate_checksum_tag10_not_last_field() {
        // Tag 10 is not the last field — invalid structure.
        let mut dec = Decoder::new();
        let msg = dec.decode(b"8=FIX.4.2\x0110=181\x0135=D\x01").unwrap();
        assert!(matches!(
            msg.validate_checksum().unwrap_err(),
            FixError::InvalidCheckSum
        ));
    }

    #[test]
    fn validate_both_correct_together() {
        // Both validations pass on the same well-formed message.
        let mut dec = Decoder::new();
        let msg = dec
            .decode(b"8=FIX.4.2\x019=25\x0135=D\x0149=SENDER\x0156=TARGET\x0110=195\x01")
            .unwrap();
        assert!(msg.validate_body_length().is_ok());
        assert!(msg.validate_checksum().is_ok());
    }

    // -------------------------------------------------------------------------
    // Group 11 — decode_fields (lazy field iteration)
    // -------------------------------------------------------------------------

    #[test]
    fn decode_fields_empty_buffer() {
        let dec = Decoder::new();
        let mut iter = dec.decode_fields(b"");
        assert!(iter.next().is_none());
    }

    #[test]
    fn decode_fields_multiple_fields() {
        let dec = Decoder::new();
        let fields: Vec<_> = dec
            .decode_fields(b"8=FIX.4.2\x0135=D\x0149=SENDER\x01")
            .map(|f| f.unwrap())
            .collect();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].tag, 8);
        assert_eq!(fields[0].value, b"FIX.4.2");
        assert_eq!(fields[1].tag, 35);
        assert_eq!(fields[1].value, b"D");
        assert_eq!(fields[2].tag, 49);
        assert_eq!(fields[2].value, b"SENDER");
    }

    #[test]
    fn decode_fields_empty_value() {
        let dec = Decoder::new();
        let fields: Vec<_> = dec.decode_fields(b"35=\x01").map(|f| f.unwrap()).collect();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].tag, 35);
        assert_eq!(fields[0].value, b"");
    }

    #[test]
    fn decode_fields_value_containing_equals() {
        let dec = Decoder::new();
        let fields: Vec<_> = dec
            .decode_fields(b"58=price=100\x0135=D\x01")
            .map(|f| f.unwrap())
            .collect();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].tag, 58);
        assert_eq!(fields[0].value, b"price=100");
        assert_eq!(fields[1].tag, 35);
        assert_eq!(fields[1].value, b"D");
    }

    #[test]
    fn decode_fields_binary_value() {
        let dec = Decoder::new();
        let fields: Vec<_> = dec
            .decode_fields(b"95=3\x0196=\x02\x03\x04\x01")
            .map(|f| f.unwrap())
            .collect();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].tag, 96);
        assert_eq!(fields[1].value, &[0x02u8, 0x03, 0x04]);
    }

    #[test]
    fn decode_fields_incomplete_tag_no_equals() {
        let dec = Decoder::new();
        let mut iter = dec.decode_fields(b"8");
        assert!(matches!(
            iter.next(),
            Some(Err(FixError::IncompleteMessage))
        ));
        assert!(iter.next().is_none());
    }

    #[test]
    fn decode_fields_invalid_tag() {
        let dec = Decoder::new();
        let mut iter = dec.decode_fields(b"8X=val\x01");
        assert!(matches!(iter.next(), Some(Err(FixError::InvalidTag))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn decode_fields_incomplete_mid_stream() {
        let dec = Decoder::new();
        let mut iter = dec.decode_fields(b"8=FIX.4.2\x0135");
        let first = iter.next().unwrap().unwrap();
        assert_eq!(first.tag, 8);
        assert_eq!(first.value, b"FIX.4.2");
        assert!(matches!(
            iter.next(),
            Some(Err(FixError::IncompleteMessage))
        ));
        assert!(iter.next().is_none());
    }

    #[test]
    fn decode_fields_33_fields() {
        let dec = Decoder::new();
        let mut buf = Vec::new();
        for i in 1u32..=33 {
            buf.extend_from_slice(format!("{}=v\x01", i).as_bytes());
        }
        let fields: Vec<_> = dec.decode_fields(&buf).map(|f| f.unwrap()).collect();
        assert_eq!(fields.len(), 33);
        assert_eq!(fields[32].tag, 33);
    }

    #[test]
    fn decode_matches_decode_fields() {
        let mut dec = Decoder::new();
        let buf = b"8=FIX.4.2\x0135=D\x0149=SENDER\x0156=TARGET\x0111=ORD1\x0155=AAPL\x01";
        let msg = dec.decode(buf).unwrap();
        let from_decode: Vec<(u32, Vec<u8>)> =
            msg.fields().map(|f| (f.tag, f.value.to_vec())).collect();
        let from_iter: Vec<(u32, Vec<u8>)> = dec
            .decode_fields(buf)
            .map(|f| {
                let f = f.unwrap();
                (f.tag, f.value.to_vec())
            })
            .collect();
        assert_eq!(from_decode, from_iter);
    }

    // -------------------------------------------------------------------------
    // Group 12 — find()/find_all() behavior
    // -------------------------------------------------------------------------

    #[test]
    fn decode_find_returns_first_duplicate() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"372=D\x01372=8\x01").unwrap();
        assert_eq!(msg.find(372).unwrap().value, b"D");
    }

    #[test]
    fn decode_find_all_duplicates_in_order() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"372=D\x01372=8\x01372=9\x01").unwrap();
        let values: Vec<_> = msg.find_all(372).map(|f| f.value).collect();
        assert_eq!(values, vec![&b"D"[..], &b"8"[..], &b"9"[..]]);
    }

    #[test]
    fn decode_find_absent_tag() {
        let mut dec = Decoder::new();
        let msg = dec.decode(b"35=D\x01").unwrap();
        assert!(msg.find(8).is_none());
        assert_eq!(msg.find_all(8).count(), 0);
    }

    #[test]
    fn decode_reuse_no_stale_state() {
        let mut dec = Decoder::new();
        {
            let msg = dec.decode(b"8=FIX.4.2\x0135=D\x01").unwrap();
            assert!(msg.find(8).is_some());
            assert!(msg.find(49).is_none());
        }
        {
            let msg = dec.decode(b"49=SENDER\x0156=TARGET\x01").unwrap();
            assert!(msg.find(49).is_some());
            assert!(msg.find(8).is_none());
            assert!(msg.find(35).is_none());
        }
    }

    #[test]
    fn decode_find_reflects_current_message() {
        let mut dec = Decoder::new();
        {
            let msg = dec.decode(b"8=FIX.4.2\x0135=D\x0149=A\x01").unwrap();
            assert_eq!(msg.find(49).unwrap().value, b"A");
        }
        {
            let msg = dec.decode(b"49=B\x0156=TARGET\x01").unwrap();
            assert_eq!(msg.find(49).unwrap().value, b"B");
            let tags: Vec<_> = msg.fields().map(|f| f.tag).collect();
            assert_eq!(tags, vec![49, 56]);
            assert_eq!(msg.find_all(56).count(), 1);
        }
    }
}
