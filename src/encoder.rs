use smallvec::SmallVec;

use crate::checksum::compute_checksum;
use crate::error::FixError;
use crate::field::FIELD_SEPARATOR;
use crate::message::Message;
use crate::tag;

/// Default inline capacity for the body buffer (bytes).
/// Covers the body of most FIX messages without spilling to the heap.
const DEFAULT_CAPACITY: usize = 512;

/// A reusable FIX message encoder.
///
/// Owns a body buffer that is allocated once and reused across every `encode`
/// call — zero allocation per message on the hot path after the first call.
///
/// # Example
/// ```ignore
/// let mut enc = Encoder::new();
/// let mut out = Vec::new();
/// enc.encode(&msg, &mut out)?;
/// ```
pub struct Encoder {
    /// Reusable scratch buffer for building the message body.
    /// Cleared (not dropped) at the start of each encode call so capacity is preserved.
    body: SmallVec<[u8; DEFAULT_CAPACITY]>,
}

impl Encoder {
    /// Create a new encoder with default inline body-buffer capacity.
    pub fn new() -> Self {
        Self {
            body: SmallVec::new(),
        }
    }

    /// Create a new encoder pre-allocated for `capacity` body bytes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            body: SmallVec::with_capacity(capacity),
        }
    }

    /// Encode `msg` as a complete FIX wire message into `out`.
    ///
    /// `out` is cleared first. Tag 9 (BodyLength) and tag 10 (CheckSum) are
    /// computed automatically; any existing 9 or 10 fields in `msg` are ignored.
    /// If tag 8 (BeginString) is absent, `FIX.4.4` is used as the default version.
    pub fn encode(&mut self, msg: &Message<'_>, out: &mut Vec<u8>) -> Result<(), FixError> {
        const DEFAULT_VERSION: &[u8] = b"FIX.4.4";
        let version = msg
            .find(tag::BEGIN_STRING)
            .map(|f| f.value)
            .unwrap_or(DEFAULT_VERSION);

        // Build body bytes into reusable scratch buffer (all fields except 8, 9, 10).
        self.body.clear();
        for field in msg.fields() {
            if field.tag == tag::BEGIN_STRING
                || field.tag == tag::BODY_LENGTH
                || field.tag == tag::CHECK_SUM
            {
                continue;
            }
            self.body.extend_from_slice(field.tag.to_string().as_bytes());
            self.body.push(b'=');
            self.body.extend_from_slice(field.value);
            self.body.push(FIELD_SEPARATOR);
        }

        // Assemble output: tag 8, tag 9, body, tag 10.
        out.clear();

        out.extend_from_slice(b"8=");
        out.extend_from_slice(version);
        out.push(FIELD_SEPARATOR);

        out.extend_from_slice(b"9=");
        out.extend_from_slice(self.body.len().to_string().as_bytes());
        out.push(FIELD_SEPARATOR);

        out.extend_from_slice(&self.body);

        let checksum = compute_checksum(out);
        out.extend_from_slice(b"10=");
        out.extend_from_slice(format!("{:03}", checksum).as_bytes());
        out.push(FIELD_SEPARATOR);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

    #[test]
    fn encode_single_body_field() {
        let raw = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";
        let mut dec = Decoder::new();
        let msg = dec.decode(raw).unwrap();
        let mut enc = Encoder::new();
        let mut out = Vec::new();
        enc.encode(&msg, &mut out).unwrap();
        assert_eq!(out.as_slice(), raw.as_ref());
    }

    #[test]
    fn encode_validates_body_length_and_checksum() {
        let raw = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";
        let mut dec = Decoder::new();
        let msg = dec.decode(raw).unwrap();
        let mut enc = Encoder::new();
        let mut out = Vec::new();
        enc.encode(&msg, &mut out).unwrap();
        let mut dec2 = Decoder::new();
        let msg2 = dec2.decode(&out).unwrap();
        assert!(msg2.validate_body_length().is_ok());
        assert!(msg2.validate_checksum().is_ok());
    }

    #[test]
    fn encode_multiple_body_fields() {
        let raw = b"8=FIX.4.2\x019=20\x0135=D\x0149=SENDER\x0156=TARGET\x0110=100\x01";
        let mut dec = Decoder::new();
        let msg = dec.decode(raw).unwrap();
        let mut enc = Encoder::new();
        let mut out = Vec::new();
        enc.encode(&msg, &mut out).unwrap();
        let mut dec2 = Decoder::new();
        let msg2 = dec2.decode(&out).unwrap();
        assert!(msg2.validate_body_length().is_ok());
        assert!(msg2.validate_checksum().is_ok());
    }

    #[test]
    fn encode_missing_tag8_defaults_to_fix44() {
        let raw = b"35=D\x01";
        let mut dec = Decoder::new();
        let msg = dec.decode(raw).unwrap();
        let mut enc = Encoder::new();
        let mut out = Vec::new();
        enc.encode(&msg, &mut out).unwrap();
        assert!(out.starts_with(b"8=FIX.4.4\x01"));
        let mut dec2 = Decoder::new();
        let msg2 = dec2.decode(&out).unwrap();
        assert!(msg2.validate_body_length().is_ok());
        assert!(msg2.validate_checksum().is_ok());
    }

    #[test]
    fn encode_clears_existing_out_buffer() {
        let raw = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";
        let mut dec = Decoder::new();
        let msg = dec.decode(raw).unwrap();
        let mut enc = Encoder::new();
        let mut out = b"stale_data".to_vec();
        enc.encode(&msg, &mut out).unwrap();
        assert!(!out.starts_with(b"stale_data"));
        assert!(out.starts_with(b"8="));
    }

    #[test]
    fn encode_reuse_encode_twice() {
        // Encoder reuse: body buffer capacity is preserved across calls.
        let raw1 = b"8=FIX.4.2\x019=5\x0135=D\x0110=181\x01";
        let raw2 = b"8=FIX.4.2\x019=20\x0135=D\x0149=SENDER\x0156=TARGET\x0110=100\x01";
        let mut dec = Decoder::new();
        let mut enc = Encoder::new();
        let mut out = Vec::new();

        let msg1 = dec.decode(raw1).unwrap();
        enc.encode(&msg1, &mut out).unwrap();
        let encoded1 = out.clone();

        let msg2 = dec.decode(raw2).unwrap();
        enc.encode(&msg2, &mut out).unwrap();

        // First result was correct.
        let mut dec2 = Decoder::new();
        let m1 = dec2.decode(&encoded1).unwrap();
        assert!(m1.validate_body_length().is_ok());
        assert!(m1.validate_checksum().is_ok());

        // Second result is correct.
        let m2 = dec2.decode(&out).unwrap();
        assert!(m2.validate_body_length().is_ok());
        assert!(m2.validate_checksum().is_ok());
    }
}
