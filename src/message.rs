use crate::field::Field;
use crate::group::{parse_count, GroupIter, GroupSpec, FIX42_GROUPS};
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

    /// Return an iterator over the instances of the repeating group described
    /// by `spec`.
    ///
    /// The iterator is zero-copy: each `Group` borrows directly into this
    /// message's offset slice and raw buffer. If the count tag is absent or
    /// its value is `0`, the iterator yields nothing.
    ///
    /// # Example
    /// ```ignore
    /// for entry in msg.groups(&group::MD_ENTRIES) {
    ///     let ty  = entry.find(tag::MD_ENTRY_TYPE);
    ///     let px  = entry.find(tag::MD_ENTRY_PX);
    /// }
    /// ```
    #[inline]
    pub fn groups(&self, spec: &GroupSpec) -> GroupIter<'a> {
        // Find the NO_* count tag position.
        let pos = self
            .offsets
            .iter()
            .position(|&(t, _, _)| t == spec.count_tag);

        let (count, remaining) = match pos {
            None => (0, &[][..]),
            Some(i) => {
                let (_, start, end) = self.offsets[i];
                let count = parse_count(&self.buf[start as usize..end as usize]);
                let after = &self.offsets[i + 1..];
                (count, after)
            }
        };

        GroupIter {
            buf: self.buf,
            remaining,
            delimiter_tag: spec.delimiter_tag,
            count,
            emitted: 0,
        }
    }

    /// Return an iterator over every repeating group present in this message.
    ///
    /// Scans `FIX42_GROUPS` and yields `(&'static GroupSpec, GroupIter<'a>)`
    /// for each spec whose count tag is found in the message with a non-zero
    /// count. Groups whose count tag is absent or zero are skipped.
    ///
    /// The order follows the order fields appear in the message, not the order
    /// of `FIX42_GROUPS`.
    ///
    /// # Example
    /// ```ignore
    /// for (spec, instances) in msg.all_groups() {
    ///     for g in instances {
    ///         // process each group instance
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn all_groups(&self) -> impl Iterator<Item = (&'static GroupSpec, GroupIter<'a>)> + '_ {
        FIX42_GROUPS.iter().copied().filter_map(|spec| {
            // Check if the count tag is present with a non-zero count.
            let found = self.offsets.iter().find(|&&(t, _, _)| t == spec.count_tag);
            let &(_, start, end) = found?;
            let count = parse_count(&self.buf[start as usize..end as usize]);
            if count == 0 {
                return None;
            }
            Some((spec, self.groups(spec)))
        })
    }
}
