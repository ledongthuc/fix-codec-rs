/// The application-level FIX version, resolved from a message's own fields.
///
/// FIX 5.0 and later split the session (transport) layer from the application
/// layer: the `BeginString(8)` is `FIXT.1.1` and the application version is
/// carried in `ApplVerID(1128)`. Legacy `BeginString` values (`FIX.4.2`,
/// `FIX.4.4`) identify both layers directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixVersion {
    Fix42,
    Fix44,
    Fix50,
    // SP1/SP2 are out of scope for the Dec 2006 spec; leave room to add later.
}

/// Resolve the application version from the message's own fields only.
///
/// `begin_string` is the raw tag 8 value and `appl_ver_id` is the raw tag 1128
/// value. This is deliberately codec-level: it never consults session state
/// (e.g. Logon `NoMsgTypes` or `DefaultApplVerID`).
///
/// Returns `None` when the version is unknown or unsupported — callers must not
/// guess.
pub(crate) fn resolve(
    begin_string: Option<&[u8]>,
    appl_ver_id: Option<&[u8]>,
) -> Option<FixVersion> {
    match begin_string {
        Some(b"FIX.4.2") => Some(FixVersion::Fix42),
        Some(b"FIX.4.4") => Some(FixVersion::Fix44),
        Some(b"FIXT.1.1") => match appl_ver_id {
            // ApplVerID is an enumerated String; compare exact bytes, no trimming.
            // Valid values in this spec are "0"..="7".
            Some(b"4") => Some(FixVersion::Fix42),
            Some(b"6") => Some(FixVersion::Fix44),
            Some(b"7") => Some(FixVersion::Fix50),
            _ => None, // absent, or unsupported (0/1/2/3/5/8/9/...)
        },
        _ => None, // unknown or absent BeginString
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_begin_strings_are_authoritative() {
        assert_eq!(resolve(Some(b"FIX.4.2"), None), Some(FixVersion::Fix42));
        assert_eq!(resolve(Some(b"FIX.4.4"), None), Some(FixVersion::Fix44));

        // ApplVerID is ignored for legacy BeginString values.
        assert_eq!(
            resolve(Some(b"FIX.4.4"), Some(b"7")),
            Some(FixVersion::Fix44)
        );
        assert_eq!(
            resolve(Some(b"FIX.4.2"), Some(b"7")),
            Some(FixVersion::Fix42)
        );
    }

    #[test]
    fn fixt_uses_appl_ver_id() {
        assert_eq!(
            resolve(Some(b"FIXT.1.1"), Some(b"4")),
            Some(FixVersion::Fix42)
        );
        assert_eq!(
            resolve(Some(b"FIXT.1.1"), Some(b"6")),
            Some(FixVersion::Fix44)
        );
        assert_eq!(
            resolve(Some(b"FIXT.1.1"), Some(b"7")),
            Some(FixVersion::Fix50)
        );
    }

    #[test]
    fn fixt_unknown_or_absent_appl_ver_id_resolves_none() {
        assert_eq!(resolve(Some(b"FIXT.1.1"), None), None);
        assert_eq!(resolve(Some(b"FIXT.1.1"), Some(b"5")), None);
        assert_eq!(resolve(Some(b"FIXT.1.1"), Some(b"8")), None);
        assert_eq!(resolve(Some(b"FIXT.1.1"), Some(b"FIX.4.2")), None);
    }

    #[test]
    fn unknown_or_absent_begin_string_resolves_none() {
        assert_eq!(resolve(None, Some(b"7")), None);
        assert_eq!(resolve(Some(b"FIX.5.0"), Some(b"7")), None);
        assert_eq!(resolve(Some(b"FIX.4.0"), None), None);
        assert_eq!(resolve(Some(b"FIX.4.1"), None), None);
        assert_eq!(resolve(Some(b"FIX.4.3"), None), None);
    }
}
