use crate::tag::Tag;

pub const FIELD_SEPARATOR: u8 = 0x01;
pub const FIELD_SEPARATOR_DISPLAY: char = '|'; // Only use to printing to UI or text-based debug
// purpose
pub const FIELD_KEY_VALUE_SEPARATOR: u8 = b'=';

#[derive(Debug)]
pub struct Field<'a> {
    pub tag: Tag,
    pub value: &'a [u8],
}
