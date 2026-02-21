/// Compute the FIX checksum over `bytes`: sum of every byte value, mod 256.
///
/// The result fits in a `u8` because wrapping arithmetic is used throughout.
#[inline]
pub(crate) fn compute_checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// Parse the ASCII decimal checksum value stored in a FIX tag-10 field value.
///
/// The value must be a decimal integer in 0–255. Returns `None` if the bytes
/// are not valid UTF-8, not a valid integer, or out of range.
#[inline]
pub(crate) fn parse_checksum(value: &[u8]) -> Option<u8> {
    let s = std::str::from_utf8(value).ok()?;
    let n: u32 = s.trim().parse().ok()?;
    if n > 255 {
        return None;
    }
    Some(n as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_single_byte() {
        assert_eq!(compute_checksum(b"A"), b'A'); // 65
    }

    #[test]
    fn compute_wraps_at_256() {
        // 200 + 100 = 300 → 300 % 256 = 44
        assert_eq!(compute_checksum(&[200u8, 100u8]), 44);
    }

    #[test]
    fn compute_empty_is_zero() {
        assert_eq!(compute_checksum(b""), 0);
    }

    #[test]
    fn parse_valid() {
        assert_eq!(parse_checksum(b"000"), Some(0));
        assert_eq!(parse_checksum(b"128"), Some(128));
        assert_eq!(parse_checksum(b"255"), Some(255));
    }

    #[test]
    fn parse_out_of_range() {
        assert_eq!(parse_checksum(b"256"), None);
        assert_eq!(parse_checksum(b"999"), None);
    }

    #[test]
    fn parse_non_numeric() {
        assert_eq!(parse_checksum(b"abc"), None);
        assert_eq!(parse_checksum(b""), None);
    }
}
