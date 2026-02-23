use std::cell::OnceCell;

use smallvec::SmallVec;

use crate::body_length::parse_body_length;
use crate::checksum::{compute_checksum, parse_checksum};
use crate::error::FixError;
use crate::field::Field;
use crate::group::{parse_count, GroupIter, GroupSpec, FIX42_GROUPS, FIX44_GROUPS};
use crate::tag::{self, Tag};

/// Default inline capacity for the sorted index — matches the decoder's field capacity.
const SORTED_CAPACITY: usize = 32;

/// A decoded FIX message.
///
/// Zero-copy: field values are sub-slices of the original input buffer — no
/// bytes are copied when accessing fields.
///
/// The sorted tag index for [`find`] is built lazily on the first call and
/// cached for the lifetime of the message. This means `decode()` pays no sort
/// cost when you never call `find()`, and pays it at most once when you do.
#[derive(Debug)]
pub struct Message<'a> {
    /// The raw bytes of the complete FIX message as received (e.g. the network
    /// buffer passed to `Decoder::decode`). Every field value is a sub-slice of
    /// this buffer — no bytes are copied when accessing fields.
    pub(crate) buf: &'a [u8],

    /// Index of parsed fields. Each entry is `(tag, start, end)` where:
    /// - `tag`   — the numeric FIX tag (e.g. `8`, `35`, `49`).
    /// - `start` — byte offset in `buf` where the field *value* begins
    ///             (the byte immediately after `=`).
    /// - `end`   — byte offset in `buf` where the field value ends
    ///             (the SOH byte `\x01`, exclusive).
    ///
    /// A field value is recovered as `&buf[start as usize..end as usize]`.
    /// The slice is borrowed from the `Decoder`'s internal `SmallVec`, so it
    /// lives as long as `'a`.
    pub(crate) offsets: &'a [(Tag, u32, u32)],

    /// Sorted (tag, offsets_index) pairs for O(log n) binary search in find().
    ///
    /// Built lazily on the first call to `find()` and cached for the lifetime
    /// of the message via `OnceCell`. Never allocated if `find()` is never
    /// called, and built at most once regardless of how many times `find()` is
    /// called.
    sorted: OnceCell<SmallVec<[(Tag, u16); SORTED_CAPACITY]>>,
}

impl<'a> Message<'a> {
    /// Create a new message from a buffer and an offset slice.
    /// The sorted index starts uninitialized and is built lazily on first find().
    pub(crate) fn new(buf: &'a [u8], offsets: &'a [(Tag, u32, u32)]) -> Self {
        Self {
            buf,
            offsets,
            sorted: OnceCell::new(),
        }
    }

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

    /// Return the value of tag 8 (`BEGIN_STRING`) as a byte slice, or `None`
    /// if the field is absent.
    ///
    /// Common values are `b"FIX.4.2"`, `b"FIX.4.4"`, `b"FIXT.1.1"`, etc.
    #[inline]
    pub fn fix_version(&self) -> Option<&'a [u8]> {
        self.find(tag::BEGIN_STRING).map(|f| f.value)
    }

    /// Find the first field with the given tag, or `None` if not present.
    ///
    /// The sorted index is built lazily on the first call (O(n log n)) and
    /// cached for subsequent calls (O(log n) binary search). If `find()` is
    /// never called, the sort never happens.
    #[inline]
    pub fn find(&self, tag: Tag) -> Option<Field<'a>> {
        let sorted = self.sorted.get_or_init(|| {
            let mut v: SmallVec<[(Tag, u16); SORTED_CAPACITY]> =
                SmallVec::with_capacity(self.offsets.len());
            for (i, &(t, _, _)) in self.offsets.iter().enumerate() {
                v.push((t, i as u16));
            }
            v.sort_unstable_by_key(|&(t, _)| t);
            v
        });

        let idx = sorted.partition_point(|&(t, _)| t < tag);
        let &(found_tag, offset_idx) = sorted.get(idx)?;
        if found_tag != tag {
            return None;
        }
        let (t, start, end) = self.offsets[offset_idx as usize];
        Some(Field {
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
    /// Scans the appropriate group spec array based on the FIX version detected
    /// from tag 8 (`BEGIN_STRING`): `FIX42_GROUPS` for FIX 4.2 messages, and
    /// both `FIX42_GROUPS` + `FIX44_GROUPS` for FIX 4.4 messages (which is a
    /// superset). Yields `(&'static GroupSpec, GroupIter<'a>)` for each spec
    /// whose count tag is found in the message with a non-zero count. Groups
    /// whose count tag is absent or zero are skipped.
    ///
    /// The order follows the order of the spec arrays, not the order fields
    /// appear in the message.
    ///
    /// # Example
    /// ```ignore
    /// for (spec, instances) in msg.all_groups() {
    ///     for g in instances {
    ///         // process each group instance
    ///     }
    /// }
    /// ```
    /// Validate the BodyLength field (tag 9).
    ///
    /// A FIX message body spans from the first byte after the `9=…\x01` field
    /// up to and including the SOH that terminates the last field before `10=`.
    /// This method computes that byte count from the raw buffer and compares it
    /// to the value declared in tag 9.
    ///
    /// # Errors
    /// Returns `FixError::InvalidBodyLength` when:
    /// - The message has fewer than 3 fields (no room for tags 8, 9, and 10).
    /// - Tag 9 is not at position 1 or its value cannot be parsed as an integer.
    /// - Tag 10 is not the last field.
    /// - The computed byte count does not match the declared value.
    pub fn validate_body_length(&self) -> Result<(), FixError> {
        let n = self.offsets.len();
        if n < 3 {
            return Err(FixError::InvalidBodyLength);
        }

        // Tag 9 must be the second field.
        let (tag9, _, body_length_value_end) = self.offsets[1];
        if tag9 != tag::BODY_LENGTH {
            return Err(FixError::InvalidBodyLength);
        }

        // Tag 10 must be the last field.
        let (tag10, checksum_value_start, _) = self.offsets[n - 1];
        if tag10 != tag::CHECK_SUM {
            return Err(FixError::InvalidBodyLength);
        }

        // Parse the declared body length from the raw buffer.
        let declared = parse_body_length(
            &self.buf[self.offsets[1].1 as usize..body_length_value_end as usize],
        )
        .ok_or(FixError::InvalidBodyLength)?;

        // Body bytes: from (SOH of tag-9 field + 1) to (start of "10=" tag bytes).
        // "10=" is 3 bytes, so the tag-10 field starts at checksum_value_start - 3.
        let body_start = body_length_value_end as usize + 1;
        let checksum_tag_start = checksum_value_start as usize - 3; // len("10=") == 3
        let computed = checksum_tag_start.saturating_sub(body_start);

        if computed == declared {
            Ok(())
        } else {
            Err(FixError::InvalidBodyLength)
        }
    }

    /// Validate the CheckSum field (tag 10).
    ///
    /// The FIX checksum is the sum of every byte from the start of the buffer
    /// up to (but not including) the `10=` tag bytes, taken mod 256. This
    /// method computes that value and compares it to the 3-digit decimal string
    /// stored in tag 10.
    ///
    /// # Errors
    /// Returns `FixError::InvalidCheckSum` when:
    /// - The message has fewer than 1 field.
    /// - Tag 10 is not the last field or its value cannot be parsed.
    /// - The computed checksum does not match the declared value.
    pub fn validate_checksum(&self) -> Result<(), FixError> {
        let n = self.offsets.len();
        if n == 0 {
            return Err(FixError::InvalidCheckSum);
        }

        // Tag 10 must be the last field.
        let (tag10, checksum_value_start, checksum_value_end) = self.offsets[n - 1];
        if tag10 != tag::CHECK_SUM {
            return Err(FixError::InvalidCheckSum);
        }

        // Parse the declared checksum from the raw buffer.
        let declared = parse_checksum(
            &self.buf[checksum_value_start as usize..checksum_value_end as usize],
        )
        .ok_or(FixError::InvalidCheckSum)?;

        // Checksum covers all bytes before the "10=" tag bytes.
        let checksum_tag_start = checksum_value_start as usize - 3; // len("10=") == 3
        let computed = compute_checksum(&self.buf[..checksum_tag_start]);

        if computed == declared {
            Ok(())
        } else {
            Err(FixError::InvalidCheckSum)
        }
    }

    #[inline]
    pub fn all_groups(&self) -> impl Iterator<Item = (&'static GroupSpec, GroupIter<'a>)> + '_ {
        let specs: &[&GroupSpec] = match self.fix_version() {
            Some(b"FIX.4.4") => FIX44_GROUPS,
            _ => FIX42_GROUPS,
        };

        specs.iter().copied().filter_map(|spec| {
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
