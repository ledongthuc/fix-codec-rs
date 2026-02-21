#[derive(Debug)]
pub enum FixError {
    /// A tag field contained non-digit bytes or was otherwise malformed.
    InvalidTag,
    /// A value field contained bytes that are not valid UTF-8.
    InvalidUtf8,
    /// A numeric value field contained non-digit bytes.
    InvalidValue,
    /// The buffer contains a partial FIX field; more bytes are needed (TCP framing).
    IncompleteMessage,
    /// An error occurred during message encoding.
    EncodeError,
    /// An error occurred during message decoding.
    DecodeError,
}
