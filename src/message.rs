use crate::field::Field;
use crate::tag::Tag;

/// A decoded FIX message.
///
/// Borrows offsets directly from the `Decoder`'s internal SmallVec and the
/// raw input `buf` — zero allocation, zero copy, no unsafe code.
/// The lifetime `'a` ties this `Message` to both the `Decoder` (offset slice)
/// and the input byte buffer.
#[derive(Debug)]
pub struct Message<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) offsets: &'a [(Tag, u32, u32)],
}

impl<'a> Message<'a> {
    /// Number of fields in the message.
    #[inline]
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns true if the message contains no fields.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Returns the field at `index`, reconstructing it zero-copy from the
    /// stored byte offsets. Panics if `index >= self.len()`.
    #[inline]
    pub fn field(&self, index: usize) -> Field<'a> {
        let (tag, start, end) = self.offsets[index];
        Field {
            tag,
            value: &self.buf[start as usize..end as usize],
        }
    }

    /// Returns an iterator over all fields, reconstructing each `Field<'a>`
    /// zero-copy on demand.
    #[inline]
    pub fn fields(&self) -> impl Iterator<Item = Field<'a>> + '_ {
        self.offsets.iter().map(move |&(tag, start, end)| Field {
            tag,
            value: &self.buf[start as usize..end as usize],
        })
    }

    /// Find the first field with the given tag, or `None` if not present.
    #[inline]
    pub fn find(&self, tag: Tag) -> Option<Field<'a>> {
        self.offsets
            .iter()
            .find(|&&(t, _, _)| t == tag)
            .map(|&(t, start, end)| Field {
                tag: t,
                value: &self.buf[start as usize..end as usize],
            })
    }
}
