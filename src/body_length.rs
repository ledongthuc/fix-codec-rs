/// Parse the ASCII decimal body-length value stored in a FIX tag-9 field value.
///
/// Returns `None` if the bytes are not valid UTF-8 or not a valid integer.
#[inline]
pub(crate) fn parse_body_length(value: &[u8]) -> Option<usize> {
    let s = std::str::from_utf8(value).ok()?;
    s.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        assert_eq!(parse_body_length(b"0"), Some(0));
        assert_eq!(parse_body_length(b"42"), Some(42));
        assert_eq!(parse_body_length(b"1024"), Some(1024));
    }

    #[test]
    fn parse_non_numeric() {
        assert_eq!(parse_body_length(b"abc"), None);
        assert_eq!(parse_body_length(b""), None);
    }
}
