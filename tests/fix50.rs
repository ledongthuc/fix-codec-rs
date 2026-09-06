use fix_codec_rs::decoder::Decoder;
use fix_codec_rs::encoder::Encoder;
use fix_codec_rs::group;
use fix_codec_rs::tag;
use fix_codec_rs::version::FixVersion;

/// Convert `|` into the FIX SOH byte so fixtures read naturally in tests.
fn fix(raw: &str) -> Vec<u8> {
    raw.bytes()
        .map(|b| if b == b'|' { 0x01 } else { b })
        .collect()
}

/// Build a valid `FIXT.1.1` message: `8=FIXT.1.1`, a computed `9=`, the given
/// body fields, and a computed `10=`.
fn fixt(fields: &str) -> Vec<u8> {
    let mut body = Vec::new();
    for field in fields.split('|').filter(|s| !s.is_empty()) {
        body.extend_from_slice(field.as_bytes());
        body.push(0x01);
    }

    let mut out = Vec::new();
    out.extend_from_slice(b"8=FIXT.1.1\x01");
    out.extend_from_slice(format!("9={}\x01", body.len()).as_bytes());
    out.extend_from_slice(&body);

    let checksum = out.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
    out.extend_from_slice(format!("10={:03}\x01", checksum).as_bytes());
    out
}

#[test]
fn fixt_new_order_single_resolves_fix50_and_round_trips() {
    let raw = fixt(
        "35=D|1128=7|49=SENDER|56=TARGET|34=1|52=20240101-12:00:00|\
         11=ORD1|55=AAPL|54=1|38=100|40=2|44=150.00|1139=NONE",
    );
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.fix_version(), Some(&b"FIXT.1.1"[..]));
    assert_eq!(msg.appl_ver_id(), Some(&b"7"[..]));
    assert_eq!(msg.resolve_version(), Some(FixVersion::Fix50));
    assert_eq!(
        msg.find(tag::EXCHANGE_SPECIAL_INSTRUCTIONS).unwrap().value,
        b"NONE"
    );

    // BodyLength now spans header fields such as ApplVerID(1128).
    msg.validate_body_length().unwrap();
    msg.validate_checksum().unwrap();

    // Byte-for-byte round-trip with auto-calc disabled.
    let mut enc = Encoder::new();
    enc.disable_auto_calculate_body_length(true);
    enc.disable_auto_calculate_checksum(true);
    let mut out = Vec::new();
    enc.encode(&msg, &mut out).unwrap();
    assert_eq!(out, raw);

    // Auto-calc round-trip produces a valid 9/10.
    let mut enc = Encoder::new();
    let mut out = Vec::new();
    enc.encode(&msg, &mut out).unwrap();
    let msg2 = dec.decode(&out).unwrap();
    assert!(msg2.validate_body_length().is_ok());
    assert!(msg2.validate_checksum().is_ok());
}

#[test]
fn fixt_root_parties_nested_group_uses_fix50_groups() {
    let raw =
        fixt("35=D|1128=7|49=S|56=T|34=1|1116=1|1117=ROOT1|1118=D|1119=1|1120=1|1121=SUB1|1122=1");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), Some(FixVersion::Fix50));

    let mut root = None;
    for (spec, instances) in msg.all_groups() {
        if spec.count_tag == tag::NO_ROOT_PARTY_IDS {
            root = Some(instances);
        }
    }
    let instances = root.expect("RootParties (1116) must be discovered via FIX50_GROUPS");
    let parties: Vec<_> = instances.collect();
    assert_eq!(parties.len(), 1);
    assert_eq!(parties[0].find(tag::ROOT_PARTY_ID).unwrap().value, b"ROOT1");
    assert_eq!(
        parties[0].find(tag::ROOT_PARTY_ID_SOURCE).unwrap().value,
        b"D"
    );
    assert_eq!(parties[0].find(tag::ROOT_PARTY_ROLE).unwrap().value, b"1");

    let subs: Vec<_> = parties[0].groups(&group::ROOT_PARTY_SUB_IDS).collect();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].find(tag::ROOT_PARTY_SUB_ID).unwrap().value, b"SUB1");
    assert_eq!(
        subs[0].find(tag::ROOT_PARTY_SUB_ID_TYPE).unwrap().value,
        b"1"
    );
}

#[test]
fn fixt_without_appl_ver_id_returns_no_groups() {
    let raw = fixt("35=J|136=1|137=1.00|138=USD|139=1");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.fix_version(), Some(&b"FIXT.1.1"[..]));
    assert_eq!(msg.appl_ver_id(), None);
    assert_eq!(msg.resolve_version(), None);
    assert_eq!(msg.all_groups().count(), 0);
}

#[test]
fn legacy_begin_string_is_authoritative_over_appl_ver_id() {
    let raw = fix("8=FIX.4.4|1128=7|453=1|448=FIRM|447=D|452=1|1116=1|1117=ROOT|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), Some(FixVersion::Fix44));

    let count_tags: Vec<_> = msg.all_groups().map(|(spec, _)| spec.count_tag).collect();
    assert!(count_tags.contains(&tag::NO_PARTY_IDS));
    assert!(!count_tags.contains(&tag::NO_ROOT_PARTY_IDS));
}

#[test]
fn fixt_appl_ver_4_selects_fix42_groups() {
    let raw = fix("8=FIXT.1.1|1128=4|136=1|137=1.00|138=USD|139=1|453=1|448=FIRM|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), Some(FixVersion::Fix42));

    let count_tags: Vec<_> = msg.all_groups().map(|(spec, _)| spec.count_tag).collect();
    assert!(count_tags.contains(&tag::NO_MISC_FEES));
    assert!(!count_tags.contains(&tag::NO_PARTY_IDS));
}

#[test]
fn fixt_appl_ver_5_is_unsupported() {
    let raw = fix("8=FIXT.1.1|1128=5|136=1|137=1.00|138=USD|139=1|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), None);
    assert_eq!(msg.all_groups().count(), 0);
}

#[test]
fn fixt_absent_appl_ver_id_is_unknown() {
    let raw = fix("8=FIXT.1.1|136=1|137=1.00|138=USD|139=1|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), None);
    assert_eq!(msg.all_groups().count(), 0);
}

#[test]
fn begin_string_fix50_is_unknown() {
    let raw = fix("8=FIX.5.0|1128=7|1116=1|1117=ROOT|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), None);
    assert_eq!(msg.all_groups().count(), 0);
}

#[test]
fn unsupported_legacy_begin_strings_yield_no_groups() {
    for begin in ["8=FIX.4.0", "8=FIX.4.1", "8=FIX.4.3"] {
        let raw = fix(&format!("{begin}|136=1|137=1.00|138=USD|139=1|"));
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        assert_eq!(
            msg.resolve_version(),
            None,
            "unexpected resolve for {begin}"
        );
        assert_eq!(msg.all_groups().count(), 0, "unexpected groups for {begin}");
    }
}

#[test]
fn fixt_appl_ver_6_does_not_expose_fix50_groups() {
    let raw = fix("8=FIXT.1.1|1128=6|1116=1|1117=ROOT|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), Some(FixVersion::Fix44));
    let count_tags: Vec<_> = msg.all_groups().map(|(spec, _)| spec.count_tag).collect();
    assert!(!count_tags.contains(&tag::NO_ROOT_PARTY_IDS));
}

#[test]
fn fixt_appl_ver_7_is_superset_including_fix44_groups() {
    let raw = fix("8=FIXT.1.1|1128=7|453=1|448=FIRM|447=D|452=1|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), Some(FixVersion::Fix50));
    let count_tags: Vec<_> = msg.all_groups().map(|(spec, _)| spec.count_tag).collect();
    assert!(count_tags.contains(&tag::NO_PARTY_IDS));
}

#[test]
fn fixt_logon_without_appl_ver_id_uses_explicit_group_spec() {
    // A FIXT.1.1 Logon legitimately carries NoMsgTypes(384) but usually has no
    // ApplVerID, so all_groups() must not guess and must yield nothing.
    let raw = fixt("35=A|49=S|56=T|34=1|52=20240101-12:00:00|384=2|372=D|372=8");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    assert_eq!(msg.resolve_version(), None);
    assert_eq!(msg.all_groups().count(), 0);

    // The documented workaround is an explicit, version-dispatch-free call.
    let msg_types: Vec<_> = msg.groups(&group::MSG_TYPES).collect();
    assert_eq!(msg_types.len(), 2);
    assert_eq!(msg_types[0].find(tag::REF_MSG_TYPE).unwrap().value, b"D");
    assert_eq!(msg_types[1].find(tag::REF_MSG_TYPE).unwrap().value, b"8");
}

#[test]
fn encoder_keeps_default_version_when_tag8_absent() {
    // A constructed message with ApplVerID but no BeginString still encodes as
    // 8=FIX.4.4 (documented asymmetry); transport-independence is out of scope.
    let raw = fix("1128=7|35=D|");
    let mut dec = Decoder::new();
    let msg = dec.decode(&raw).unwrap();

    let mut enc = Encoder::new();
    let mut out = Vec::new();
    enc.encode(&msg, &mut out).unwrap();

    assert!(out.starts_with(b"8=FIX.4.4\x01"));
    assert!(out.windows(7).any(|w| w == b"1128=7\x01"));

    let msg2 = dec.decode(&out).unwrap();
    assert_eq!(msg2.resolve_version(), Some(FixVersion::Fix44));
}
