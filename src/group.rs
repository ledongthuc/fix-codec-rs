use crate::field::Field;
use crate::tag::{self, Tag};

/// Describes one repeating group in the FIX specification.
///
/// - `count_tag`: the `NO_*` tag that precedes the group and carries the instance count.
/// - `delimiter_tag`: the first tag of every instance; its reappearance signals a new instance.
/// - `member_tags`: all tags that may appear inside an instance (includes the delimiter tag).
pub struct GroupSpec {
    pub count_tag: Tag,
    pub delimiter_tag: Tag,
    pub member_tags: &'static [Tag],
}

// ---------------------------------------------------------------------------
// FIX 4.2 built-in group specs
// Source: https://www.onixs.biz/fix-dictionary/4.2/
// ---------------------------------------------------------------------------

/// NO_ALLOCS (78) — AllocAccount is the delimiter tag.
pub const ALLOCS: GroupSpec = GroupSpec {
    count_tag: tag::NO_ALLOCS,
    delimiter_tag: tag::ALLOC_ACCOUNT,
    member_tags: &[tag::ALLOC_ACCOUNT, tag::ALLOC_SHARES, tag::PROCESS_CODE],
};

/// NO_ORDERS (73) — ClOrdID is the delimiter tag.
pub const ORDERS: GroupSpec = GroupSpec {
    count_tag: tag::NO_ORDERS,
    delimiter_tag: tag::CL_ORD_ID,
    member_tags: &[
        tag::CL_ORD_ID,
        tag::LIST_SEQ_NO,
        tag::WAVE_NO,
        tag::ACCOUNT,
        tag::SETTLMNT_TYP,
        tag::FUT_SETT_DATE,
        tag::HANDL_INST,
        tag::EXEC_INST,
        tag::MIN_QTY,
        tag::MAX_FLOOR,
        tag::EX_DESTINATION,
        tag::OPEN_CLOSE,
        tag::COVERED_OR_UNCOVERED,
        tag::CUSTOMER_OR_FIRM,
        tag::MAX_SHOW,
        tag::PRICE,
        tag::STOP_PX,
        tag::PEG_DIFFERENCE,
        tag::DISCRETION_INST,
        tag::DISCRETION_OFFSET,
        tag::CURRENCY,
        tag::COMPLIANCE_ID,
        tag::SOLICITED_FLAG,
        tag::IOI_ID,
        tag::TIME_IN_FORCE,
        tag::EXPIRE_TIME,
        tag::COMMISSION,
        tag::RULE80A,
        tag::FOREX_REQ,
        tag::SETTL_CURRENCY,
        tag::ORDER_QTY,
        tag::CASH_ORDER_QTY,
        tag::ORD_TYPE,
        tag::SIDE,
        tag::LOCATE_REQD,
        tag::TRANSACT_TIME,
        tag::SYMBOL,
        tag::SYMBOL_SFX,
        tag::SECURITY_ID,
        tag::ID_SOURCE,
        tag::SECURITY_TYPE,
        tag::MATURITY_MONTH_YEAR,
        tag::MATURITY_DAY,
        tag::PUT_OR_CALL,
        tag::STRIKE_PRICE,
        tag::OPT_ATTRIBUTE,
        tag::CONTRACT_MULTIPLIER,
        tag::COUPON_RATE,
        tag::SECURITY_EXCHANGE,
        tag::ISSUER,
        tag::SECURITY_DESC,
        tag::TEXT,
    ],
};

/// NO_RPTS (82) — RptSeq is the delimiter tag.
pub const RPTS: GroupSpec = GroupSpec {
    count_tag: tag::NO_RPTS,
    delimiter_tag: tag::RPT_SEQ,
    member_tags: &[tag::RPT_SEQ],
};

/// NO_DLVY_INST (85) — DlvyInst is the delimiter tag.
pub const DLVY_INST: GroupSpec = GroupSpec {
    count_tag: tag::NO_DLVY_INST,
    delimiter_tag: tag::DLVY_INST,
    member_tags: &[tag::DLVY_INST],
};

/// NO_EXECS (124) — ExecID is the delimiter tag.
pub const EXECS: GroupSpec = GroupSpec {
    count_tag: tag::NO_EXECS,
    delimiter_tag: tag::EXEC_ID,
    member_tags: &[tag::EXEC_ID, tag::LAST_SHARES, tag::LAST_PX, tag::LAST_CAPACITY],
};

/// NO_MISC_FEES (136) — MiscFeeAmt is the delimiter tag.
pub const MISC_FEES: GroupSpec = GroupSpec {
    count_tag: tag::NO_MISC_FEES,
    delimiter_tag: tag::MISC_FEE_AMT,
    member_tags: &[tag::MISC_FEE_AMT, tag::MISC_FEE_CURR, tag::MISC_FEE_TYPE],
};

/// NO_RELATED_SYM (146) — RelatdSym is the delimiter tag.
pub const RELATED_SYM: GroupSpec = GroupSpec {
    count_tag: tag::NO_RELATED_SYM,
    delimiter_tag: tag::RELATD_SYM,
    member_tags: &[
        tag::RELATD_SYM,
        tag::SYMBOL_SFX,
        tag::SECURITY_ID,
        tag::ID_SOURCE,
        tag::SECURITY_TYPE,
        tag::MATURITY_MONTH_YEAR,
        tag::MATURITY_DAY,
        tag::PUT_OR_CALL,
        tag::STRIKE_PRICE,
        tag::OPT_ATTRIBUTE,
        tag::CONTRACT_MULTIPLIER,
        tag::COUPON_RATE,
        tag::SECURITY_EXCHANGE,
        tag::ISSUER,
        tag::SECURITY_DESC,
    ],
};

/// NO_IOI_QUALIFIERS (199) — IOIQualifier is the delimiter tag.
pub const IOI_QUALIFIERS: GroupSpec = GroupSpec {
    count_tag: tag::NO_IOI_QUALIFIERS,
    delimiter_tag: tag::IOI_QUALIFIER,
    member_tags: &[tag::IOI_QUALIFIER],
};

/// NO_ROUTING_IDS (215) — RoutingType is the delimiter tag.
pub const ROUTING_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_ROUTING_IDS,
    delimiter_tag: tag::ROUTING_TYPE,
    member_tags: &[tag::ROUTING_TYPE, tag::ROUTING_ID],
};

/// NO_MD_ENTRY_TYPES (267) — MDEntryType is the delimiter tag.
pub const MD_ENTRY_TYPES: GroupSpec = GroupSpec {
    count_tag: tag::NO_MD_ENTRY_TYPES,
    delimiter_tag: tag::MD_ENTRY_TYPE,
    member_tags: &[tag::MD_ENTRY_TYPE],
};

/// NO_MD_ENTRIES (268) — MDEntryType is the delimiter tag.
pub const MD_ENTRIES: GroupSpec = GroupSpec {
    count_tag: tag::NO_MD_ENTRIES,
    delimiter_tag: tag::MD_ENTRY_TYPE,
    member_tags: &[
        tag::MD_ENTRY_TYPE,
        tag::MD_ENTRY_PX,
        tag::MD_ENTRY_SIZE,
        tag::MD_ENTRY_DATE,
        tag::MD_ENTRY_TIME,
        tag::TICK_DIRECTION,
        tag::MD_MKT,
        tag::QUOTE_CONDITION,
        tag::TRADE_CONDITION,
        tag::MD_ENTRY_ID,
        tag::MD_UPDATE_ACTION,
        tag::MD_ENTRY_REF_ID,
        tag::MD_ENTRY_ORIGINATOR,
        tag::LOCATION_ID,
        tag::DESK_ID,
        tag::OPEN_CLOSE_SETTLE_FLAG,
        tag::SELLER_DAYS,
        tag::MD_ENTRY_BUYER,
        tag::MD_ENTRY_SELLER,
        tag::MD_ENTRY_POSITION_NO,
        tag::FINANCIAL_STATUS,
        tag::CORPORATE_ACTION,
    ],
};

/// NO_QUOTE_ENTRIES (295) — QuoteEntryID is the delimiter tag.
pub const QUOTE_ENTRIES: GroupSpec = GroupSpec {
    count_tag: tag::NO_QUOTE_ENTRIES,
    delimiter_tag: tag::QUOTE_ENTRY_ID,
    member_tags: &[
        tag::QUOTE_ENTRY_ID,
        tag::SYMBOL,
        tag::SYMBOL_SFX,
        tag::SECURITY_ID,
        tag::ID_SOURCE,
        tag::SECURITY_TYPE,
        tag::MATURITY_MONTH_YEAR,
        tag::MATURITY_DAY,
        tag::PUT_OR_CALL,
        tag::STRIKE_PRICE,
        tag::OPT_ATTRIBUTE,
        tag::CONTRACT_MULTIPLIER,
        tag::COUPON_RATE,
        tag::SECURITY_EXCHANGE,
        tag::ISSUER,
        tag::SECURITY_DESC,
        tag::BID_PX,
        tag::OFFER_PX,
        tag::BID_SIZE,
        tag::OFFER_SIZE,
        tag::VALID_UNTIL_TIME,
        tag::BID_SPOT_RATE,
        tag::OFFER_SPOT_RATE,
        tag::BID_FORWARD_POINTS,
        tag::OFFER_FORWARD_POINTS,
        tag::TRANSACT_TIME,
        tag::TRADING_SESSION_ID,
        tag::QUOTE_ENTRY_REJECT_REASON,
    ],
};

/// NO_QUOTE_SETS (296) — QuoteSetID is the delimiter tag.
pub const QUOTE_SETS: GroupSpec = GroupSpec {
    count_tag: tag::NO_QUOTE_SETS,
    delimiter_tag: tag::QUOTE_SET_ID,
    member_tags: &[
        tag::QUOTE_SET_ID,
        tag::UNDERLYING_SYMBOL,
        tag::UNDERLYING_SYMBOL_SFX,
        tag::UNDERLYING_SECURITY_ID,
        tag::UNDERLYING_ID_SOURCE,
        tag::UNDERLYING_SECURITY_TYPE,
        tag::UNDERLYING_MATURITY_MONTH_YEAR,
        tag::UNDERLYING_MATURITY_DAY,
        tag::UNDERLYING_PUT_OR_CALL,
        tag::UNDERLYING_STRIKE_PRICE,
        tag::UNDERLYING_OPT_ATTRIBUTE,
        tag::UNDERLYING_CURRENCY,
        tag::QUOTE_SET_VALID_UNTIL_TIME,
        tag::TOT_QUOTE_ENTRIES,
        tag::NO_QUOTE_ENTRIES,
    ],
};

/// NO_CONTRA_BROKERS (382) — ContraBroker is the delimiter tag.
pub const CONTRA_BROKERS: GroupSpec = GroupSpec {
    count_tag: tag::NO_CONTRA_BROKERS,
    delimiter_tag: tag::CONTRA_BROKER,
    member_tags: &[
        tag::CONTRA_BROKER,
        tag::CONTRA_TRADER,
        tag::CONTRA_TRADE_QTY,
        tag::CONTRA_TRADE_TIME,
    ],
};

/// NO_MSG_TYPES (384) — RefMsgType is the delimiter tag.
pub const MSG_TYPES: GroupSpec = GroupSpec {
    count_tag: tag::NO_MSG_TYPES,
    delimiter_tag: tag::REF_MSG_TYPE,
    member_tags: &[tag::REF_MSG_TYPE, tag::MSG_DIRECTION],
};

/// NO_TRADING_SESSIONS (386) — TradingSessionID is the delimiter tag.
pub const TRADING_SESSIONS: GroupSpec = GroupSpec {
    count_tag: tag::NO_TRADING_SESSIONS,
    delimiter_tag: tag::TRADING_SESSION_ID,
    member_tags: &[tag::TRADING_SESSION_ID],
};

/// NO_BID_DESCRIPTORS (398) — BidDescriptorType is the delimiter tag.
pub const BID_DESCRIPTORS: GroupSpec = GroupSpec {
    count_tag: tag::NO_BID_DESCRIPTORS,
    delimiter_tag: tag::BID_DESCRIPTOR_TYPE,
    member_tags: &[
        tag::BID_DESCRIPTOR_TYPE,
        tag::BID_DESCRIPTOR,
        tag::SIDE_VALUE_IND,
        tag::LIQUIDITY_VALUE,
        tag::LIQUIDITY_NUM_SECURITIES,
        tag::LIQUIDITY_PCT_LOW,
        tag::LIQUIDITY_PCT_HIGH,
        tag::EFP_TRACKING_ERROR,
        tag::FAIR_VALUE,
        tag::OUTSIDE_INDEX_PCT,
        tag::VALUE_OF_FUTURES,
    ],
};

/// NO_BID_COMPONENTS (420) — ClearingFirm is the delimiter tag.
pub const BID_COMPONENTS: GroupSpec = GroupSpec {
    count_tag: tag::NO_BID_COMPONENTS,
    delimiter_tag: tag::CLEARING_FIRM,
    member_tags: &[
        tag::CLEARING_FIRM,
        tag::CLEARING_ACCOUNT,
        tag::LIQUIDITY_IND_TYPE,
        tag::WT_AVERAGE_LIQUIDITY,
        tag::EXCHANGE_FOR_PHYSICAL,
        tag::OUT_MAIN_CNTRY_U_INDEX,
        tag::CROSS_PERCENT,
        tag::PROG_RPT_REQS,
        tag::PROG_PERIOD_INTERVAL,
        tag::INC_TAX_IND,
        tag::NUM_BIDDERS,
        tag::TRADE_TYPE,
        tag::BASIS_PX_TYPE,
        tag::COUNTRY,
        tag::SIDE,
        tag::PRICE,
        tag::PRICE_TYPE,
        tag::FAIR_VALUE,
    ],
};

/// NO_STRIKES (428) — Symbol is the delimiter tag.
pub const STRIKES: GroupSpec = GroupSpec {
    count_tag: tag::NO_STRIKES,
    delimiter_tag: tag::SYMBOL,
    member_tags: &[
        tag::SYMBOL,
        tag::SYMBOL_SFX,
        tag::SECURITY_ID,
        tag::ID_SOURCE,
        tag::SECURITY_TYPE,
        tag::MATURITY_MONTH_YEAR,
        tag::MATURITY_DAY,
        tag::PUT_OR_CALL,
        tag::STRIKE_PRICE,
        tag::OPT_ATTRIBUTE,
        tag::CONTRACT_MULTIPLIER,
        tag::COUPON_RATE,
        tag::SECURITY_EXCHANGE,
        tag::ISSUER,
        tag::SECURITY_DESC,
    ],
};

/// All built-in FIX 4.2 group specs.
pub const FIX42_GROUPS: &[&GroupSpec] = &[
    &ALLOCS,
    &ORDERS,
    &RPTS,
    &DLVY_INST,
    &EXECS,
    &MISC_FEES,
    &RELATED_SYM,
    &IOI_QUALIFIERS,
    &ROUTING_IDS,
    &MD_ENTRY_TYPES,
    &MD_ENTRIES,
    &QUOTE_ENTRIES,
    &QUOTE_SETS,
    &CONTRA_BROKERS,
    &MSG_TYPES,
    &TRADING_SESSIONS,
    &BID_DESCRIPTORS,
    &BID_COMPONENTS,
    &STRIKES,
];

// ---------------------------------------------------------------------------
// Group and GroupIter
// ---------------------------------------------------------------------------

/// A zero-copy view over one repeating-group instance.
///
/// Borrows a sub-slice of the parent `Message`'s offset array and the raw
/// input buffer. Field access is identical to `Message`: zero allocation,
/// zero copy.
#[derive(Debug, Clone, Copy)]
pub struct Group<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) offsets: &'a [(Tag, u32, u32)],
}

impl<'a> Group<'a> {
    /// Number of fields in this group instance.
    #[inline]
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns true if this group instance contains no fields.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Returns the field at `index`. Panics if `index >= self.len()`.
    #[inline]
    pub fn field(&self, index: usize) -> Field<'a> {
        let (tag, start, end) = self.offsets[index];
        Field {
            tag,
            value: &self.buf[start as usize..end as usize],
        }
    }

    /// Iterates over all fields in this group instance.
    #[inline]
    pub fn fields(&self) -> impl Iterator<Item = Field<'a>> + '_ {
        self.offsets.iter().map(move |&(tag, start, end)| Field {
            tag,
            value: &self.buf[start as usize..end as usize],
        })
    }

    /// Returns the first field with the given tag, or `None`.
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

/// Iterator over the instances of one repeating group.
///
/// Produced by [`Message::groups`]. Each call to `next` returns the next
/// `Group` instance as a zero-copy view into the parent message.
pub struct GroupIter<'a> {
    pub(crate) buf: &'a [u8],
    /// Remaining flat offsets starting just after the NO_* count tag.
    pub(crate) remaining: &'a [(Tag, u32, u32)],
    pub(crate) delimiter_tag: Tag,
    pub(crate) count: usize,
    pub(crate) emitted: usize,
}

impl<'a> Iterator for GroupIter<'a> {
    type Item = Group<'a>;

    fn next(&mut self) -> Option<Group<'a>> {
        if self.emitted >= self.count || self.remaining.is_empty() {
            return None;
        }

        let start = self.remaining;

        // Find the end of this instance: the next occurrence of the delimiter tag
        // after the first field, or the end of remaining.
        let end_offset = start
            .iter()
            .enumerate()
            .skip(1) // skip the delimiter tag that begins this instance
            .find(|&(_, &(t, _, _))| t == self.delimiter_tag)
            .map(|(i, _)| i)
            .unwrap_or(start.len());

        let instance_offsets = &start[..end_offset];
        self.remaining = &start[end_offset..];
        self.emitted += 1;

        Some(Group {
            buf: self.buf,
            offsets: instance_offsets,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.count.saturating_sub(self.emitted);
        (left, Some(left))
    }
}

// ---------------------------------------------------------------------------
// Helpers used by message.rs
// ---------------------------------------------------------------------------

/// Parse a decimal ASCII count value from raw bytes. Returns 0 on failure.
pub(crate) fn parse_count(bytes: &[u8]) -> usize {
    let mut n: usize = 0;
    for &b in bytes {
        if b < b'0' || b > b'9' {
            return 0;
        }
        n = n.wrapping_mul(10).wrapping_add((b - b'0') as usize);
    }
    n
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;
    use crate::tag;

    // Helper: build a raw FIX byte string from "tag=value|..." notation using '|' as SOH.
    fn fix(s: &str) -> Vec<u8> {
        s.bytes().map(|b| if b == b'|' { 0x01 } else { b }).collect()
    }

    // -----------------------------------------------------------------------
    // parse_count
    // -----------------------------------------------------------------------

    #[test]
    fn parse_count_normal() {
        assert_eq!(parse_count(b"3"), 3);
        assert_eq!(parse_count(b"10"), 10);
        assert_eq!(parse_count(b"0"), 0);
    }

    #[test]
    fn parse_count_invalid_returns_zero() {
        assert_eq!(parse_count(b""), 0);
        assert_eq!(parse_count(b"abc"), 0);
        assert_eq!(parse_count(b"1a"), 0);
    }

    // -----------------------------------------------------------------------
    // GroupIter — single group, one instance
    // -----------------------------------------------------------------------

    #[test]
    fn single_group_single_instance() {
        // NO_MISC_FEES=1 | MiscFeeAmt=10.5 | MiscFeeCurr=USD | MiscFeeType=1
        let raw = fix("136=1|137=10.5|138=USD|139=1|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let mut iter = msg.groups(&MISC_FEES);
        let g = iter.next().expect("expected one instance");
        assert_eq!(g.len(), 3);
        assert_eq!(g.field(0).tag, tag::MISC_FEE_AMT);
        assert_eq!(g.field(0).value, b"10.5");
        assert_eq!(g.field(1).tag, tag::MISC_FEE_CURR);
        assert_eq!(g.field(1).value, b"USD");
        assert_eq!(g.field(2).tag, tag::MISC_FEE_TYPE);
        assert_eq!(g.field(2).value, b"1");
        assert!(iter.next().is_none());
    }

    // -----------------------------------------------------------------------
    // GroupIter — single group, multiple instances
    // -----------------------------------------------------------------------

    #[test]
    fn single_group_two_instances() {
        // NO_MISC_FEES=2 | instance1 | instance2
        let raw = fix("136=2|137=10.5|138=USD|139=1|137=5.0|138=EUR|139=2|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let instances: Vec<_> = msg.groups(&MISC_FEES).collect();
        assert_eq!(instances.len(), 2);

        let g1 = &instances[0];
        assert_eq!(g1.find(tag::MISC_FEE_AMT).unwrap().value, b"10.5");
        assert_eq!(g1.find(tag::MISC_FEE_CURR).unwrap().value, b"USD");

        let g2 = &instances[1];
        assert_eq!(g2.find(tag::MISC_FEE_AMT).unwrap().value, b"5.0");
        assert_eq!(g2.find(tag::MISC_FEE_CURR).unwrap().value, b"EUR");
    }

    // -----------------------------------------------------------------------
    // GroupIter — count=0 produces empty iterator
    // -----------------------------------------------------------------------

    #[test]
    fn group_count_zero_empty_iter() {
        let raw = fix("136=0|35=D|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let mut iter = msg.groups(&MISC_FEES);
        assert!(iter.next().is_none());
    }

    // -----------------------------------------------------------------------
    // GroupIter — count tag absent produces empty iterator
    // -----------------------------------------------------------------------

    #[test]
    fn group_count_tag_absent_empty_iter() {
        let raw = fix("35=D|49=SENDER|56=TARGET|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let mut iter = msg.groups(&MISC_FEES);
        assert!(iter.next().is_none());
    }

    // -----------------------------------------------------------------------
    // GroupIter — fields() and find() on a Group
    // -----------------------------------------------------------------------

    #[test]
    fn group_fields_iter() {
        let raw = fix("136=1|137=9.99|138=GBP|139=3|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let g = msg.groups(&MISC_FEES).next().unwrap();
        let tags: Vec<Tag> = g.fields().map(|f| f.tag).collect();
        assert_eq!(tags, vec![tag::MISC_FEE_AMT, tag::MISC_FEE_CURR, tag::MISC_FEE_TYPE]);
    }

    #[test]
    fn group_find_present_and_absent() {
        let raw = fix("136=1|137=9.99|138=GBP|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let g = msg.groups(&MISC_FEES).next().unwrap();
        assert!(g.find(tag::MISC_FEE_AMT).is_some());
        assert!(g.find(tag::MISC_FEE_TYPE).is_none()); // not present in this instance
    }

    // -----------------------------------------------------------------------
    // GroupIter — size_hint
    // -----------------------------------------------------------------------

    #[test]
    fn group_iter_size_hint() {
        let raw = fix("136=3|137=1.0|137=2.0|137=3.0|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let mut iter = msg.groups(&MISC_FEES);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        iter.next();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    // -----------------------------------------------------------------------
    // NO_MD_ENTRIES — most common market data use case
    // -----------------------------------------------------------------------

    #[test]
    fn md_entries_two_instances() {
        // MDReqID + NO_MD_ENTRIES=2 + 2 entries each with MDEntryType/MDEntryPx/MDEntrySize
        let raw = fix(
            "262=REQ1|268=2|269=0|270=100.50|271=500|269=1|270=100.75|271=300|",
        );
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let entries: Vec<_> = msg.groups(&MD_ENTRIES).collect();
        assert_eq!(entries.len(), 2);

        let bid = &entries[0];
        assert_eq!(bid.find(tag::MD_ENTRY_TYPE).unwrap().value, b"0");
        assert_eq!(bid.find(tag::MD_ENTRY_PX).unwrap().value, b"100.50");
        assert_eq!(bid.find(tag::MD_ENTRY_SIZE).unwrap().value, b"500");

        let offer = &entries[1];
        assert_eq!(offer.find(tag::MD_ENTRY_TYPE).unwrap().value, b"1");
        assert_eq!(offer.find(tag::MD_ENTRY_PX).unwrap().value, b"100.75");
        assert_eq!(offer.find(tag::MD_ENTRY_SIZE).unwrap().value, b"300");
    }

    // -----------------------------------------------------------------------
    // Multiple different groups in the same message
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_group_types_in_message() {
        // A message with both NO_MISC_FEES and NO_ROUTING_IDS
        let raw = fix("35=D|136=1|137=1.50|138=USD|139=4|215=2|216=1|217=ROUTE_A|216=2|217=ROUTE_B|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        // Check MISC_FEES group
        let fees: Vec<_> = msg.groups(&MISC_FEES).collect();
        assert_eq!(fees.len(), 1);
        assert_eq!(fees[0].find(tag::MISC_FEE_AMT).unwrap().value, b"1.50");

        // Check ROUTING_IDS group
        let routes: Vec<_> = msg.groups(&ROUTING_IDS).collect();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].find(tag::ROUTING_TYPE).unwrap().value, b"1");
        assert_eq!(routes[0].find(tag::ROUTING_ID).unwrap().value, b"ROUTE_A");
        assert_eq!(routes[1].find(tag::ROUTING_TYPE).unwrap().value, b"2");
        assert_eq!(routes[1].find(tag::ROUTING_ID).unwrap().value, b"ROUTE_B");
    }

    // -----------------------------------------------------------------------
    // Group is_empty / len
    // -----------------------------------------------------------------------

    #[test]
    fn group_is_empty_false_for_non_empty() {
        let raw = fix("136=1|137=1.0|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();
        let g = msg.groups(&MISC_FEES).next().unwrap();
        assert!(!g.is_empty());
        assert_eq!(g.len(), 1);
    }
}
