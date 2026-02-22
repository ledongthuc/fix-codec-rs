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

// ---------------------------------------------------------------------------
// FIX 4.4 built-in group specs
// Source: https://www.onixs.biz/fix-dictionary/4.4/
// ---------------------------------------------------------------------------

/// NO_PARTY_IDS (453) — PartyID is the delimiter tag.
pub const PARTY_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_PARTY_IDS,
    delimiter_tag: tag::PARTY_ID,
    member_tags: &[tag::PARTY_ID, tag::PARTY_ID_SOURCE, tag::PARTY_ROLE, tag::PARTY_SUB_ID],
};

/// NO_SECURITY_ALT_ID (454) — SecurityAltID is the delimiter tag.
pub const SECURITY_ALT_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_SECURITY_ALT_ID,
    delimiter_tag: tag::SECURITY_ALT_ID,
    member_tags: &[tag::SECURITY_ALT_ID, tag::SECURITY_ALT_ID_SOURCE],
};

/// NO_UNDERLYING_SECURITY_ALT_ID (457) — UnderlyingSecurityAltID is the delimiter tag.
pub const UNDERLYING_SECURITY_ALT_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_UNDERLYING_SECURITY_ALT_ID,
    delimiter_tag: tag::UNDERLYING_SECURITY_ALT_ID,
    member_tags: &[tag::UNDERLYING_SECURITY_ALT_ID, tag::UNDERLYING_SECURITY_ALT_ID_SOURCE],
};

/// NO_REGIST_DTLS (473) — MailingDtls is the delimiter tag.
pub const REGIST_DTLS: GroupSpec = GroupSpec {
    count_tag: tag::NO_REGIST_DTLS,
    delimiter_tag: tag::MAILING_DTLS,
    member_tags: &[
        tag::MAILING_DTLS,
        tag::INVESTOR_COUNTRY_OF_RESIDENCE,
        tag::MAILING_INST,
        tag::REGIST_DTLS,
        tag::REGIST_EMAIL,
        tag::DISTRIB_PERCENTAGE,
        tag::REGIST_ID,
        tag::REGIST_TRANS_TYPE,
        tag::OWNER_TYPE,
        tag::NO_DISTRIB_INSTS,
        tag::DISTRIB_PAYMENT_METHOD,
        tag::CASH_DISTRIB_CURR,
        tag::CASH_DISTRIB_AGENT_NAME,
        tag::CASH_DISTRIB_AGENT_CODE,
        tag::CASH_DISTRIB_AGENT_ACCT_NUMBER,
        tag::CASH_DISTRIB_PAY_REF,
        tag::CASH_DISTRIB_AGENT_ACCT_NAME,
    ],
};

/// NO_DISTRIB_INSTS (510) — DistribPaymentMethod is the delimiter tag.
pub const DISTRIB_INSTS: GroupSpec = GroupSpec {
    count_tag: tag::NO_DISTRIB_INSTS,
    delimiter_tag: tag::DISTRIB_PAYMENT_METHOD,
    member_tags: &[
        tag::DISTRIB_PAYMENT_METHOD,
        tag::DISTRIB_PERCENTAGE,
        tag::CASH_DISTRIB_CURR,
        tag::CASH_DISTRIB_AGENT_NAME,
        tag::CASH_DISTRIB_AGENT_CODE,
        tag::CASH_DISTRIB_AGENT_ACCT_NUMBER,
        tag::CASH_DISTRIB_PAY_REF,
        tag::CASH_DISTRIB_AGENT_ACCT_NAME,
    ],
};

/// NO_CONT_AMTS (518) — ContAmtType is the delimiter tag.
pub const CONT_AMTS: GroupSpec = GroupSpec {
    count_tag: tag::NO_CONT_AMTS,
    delimiter_tag: tag::CONT_AMT_TYPE,
    member_tags: &[tag::CONT_AMT_TYPE, tag::CONT_AMT_VALUE, tag::CONT_AMT_CURR],
};

/// NO_NESTED_PARTY_IDS (539) — NestedPartyID is the delimiter tag.
pub const NESTED_PARTY_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_NESTED_PARTY_IDS,
    delimiter_tag: tag::NESTED_PARTY_ID,
    member_tags: &[
        tag::NESTED_PARTY_ID,
        tag::NESTED_PARTY_ID_SOURCE,
        tag::NESTED_PARTY_ROLE,
        tag::NESTED_PARTY_SUB_ID,
    ],
};

/// NO_SIDES (552) — Side is the delimiter tag.
pub const SIDES: GroupSpec = GroupSpec {
    count_tag: tag::NO_SIDES,
    delimiter_tag: tag::SIDE,
    member_tags: &[
        tag::SIDE,
        tag::ORDER_ID,
        tag::SECONDARY_ORDER_ID,
        tag::CL_ORD_ID,
        tag::SECONDARY_CL_ORD_ID,
        tag::LIST_ID,
        tag::ACCOUNT,
        tag::ACCT_ID_SOURCE,
        tag::ACCOUNT_TYPE,
        tag::PROCESS_CODE,
        tag::ODD_LOT,
        tag::NO_CLEARING_INSTRUCTIONS,
        tag::CLEARING_INSTRUCTION,
        tag::CLEARING_FEE_INDICATOR,
        tag::TRADE_INPUT_SOURCE,
        tag::TRADE_INPUT_DEVICE,
        tag::ORDER_INPUT_DEVICE,
        tag::CURRENCY,
        tag::COMPLIANCE_ID,
        tag::SOLICITED_FLAG,
        tag::ORDER_CAPACITY,
        tag::ORDER_RESTRICTIONS,
        tag::CUST_ORDER_CAPACITY,
        tag::ORD_TYPE,
        tag::EXEC_INST,
        tag::TRANS_BKD_TIME,
        tag::TRADING_SESSION_ID,
        tag::TRADING_SESSION_SUB_ID,
        tag::COMMISSION,
        tag::COMM_TYPE,
        tag::COMM_CURRENCY,
        tag::FUND_RENEW_WAIV,
        tag::GROSS_TRADE_AMT,
        tag::NUM_DAYS_INTEREST,
        tag::EX_DESTINATION,
        tag::ACCRUED_INTEREST_RATE,
        tag::ACCRUED_INTEREST_AMT,
        tag::INTEREST_AT_MATURITY,
        tag::END_ACCRUED_INTEREST_AMT,
        tag::START_CASH,
        tag::END_CASH,
        tag::NET_MONEY,
        tag::SETTL_CURR_AMT,
        tag::SETTL_CURRENCY,
        tag::SETTL_CURR_FX_RATE,
        tag::SETTL_CURR_FX_RATE_CALC,
        tag::POSITION_EFFECT,
        tag::TEXT,
        tag::ENCODED_TEXT_LEN,
        tag::ENCODED_TEXT,
        tag::SIDE_MULTI_LEG_REPORTING_TYPE,
        tag::NO_CONT_AMTS,
        tag::CONT_AMT_TYPE,
        tag::CONT_AMT_VALUE,
        tag::CONT_AMT_CURR,
        tag::NO_MISC_FEES,
        tag::MISC_FEE_AMT,
        tag::MISC_FEE_CURR,
        tag::MISC_FEE_TYPE,
        tag::MISC_FEE_BASIS,
        tag::EXCHANGE_RULE,
        tag::TRADE_ALLOC_INDICATOR,
        tag::PREALLOC_METHOD,
        tag::ALLOC_ID,
        tag::NO_ALLOCS,
        tag::ALLOC_ACCOUNT,
        tag::ALLOC_ACCT_ID_SOURCE,
        tag::ALLOC_SETTL_CURRENCY,
        tag::INDIVIDUAL_ALLOC_ID,
        tag::ALLOC_SHARES,
    ],
};

/// NO_SECURITY_TYPES (558) — SecurityType is the delimiter tag.
pub const SECURITY_TYPES: GroupSpec = GroupSpec {
    count_tag: tag::NO_SECURITY_TYPES,
    delimiter_tag: tag::SECURITY_TYPE,
    member_tags: &[tag::SECURITY_TYPE, tag::PRODUCT, tag::CFI_CODE],
};

/// NO_AFFECTED_ORDERS (534) — AffectedOrderID is the delimiter tag.
pub const AFFECTED_ORDERS: GroupSpec = GroupSpec {
    count_tag: tag::NO_AFFECTED_ORDERS,
    delimiter_tag: tag::AFFECTED_ORDER_ID,
    member_tags: &[tag::AFFECTED_ORDER_ID, tag::AFFECTED_SECONDARY_ORDER_ID],
};

/// NO_LEGS (555) — LegSymbol is the delimiter tag.
pub const LEGS: GroupSpec = GroupSpec {
    count_tag: tag::NO_LEGS,
    delimiter_tag: tag::LEG_SYMBOL,
    member_tags: &[
        tag::LEG_SYMBOL,
        tag::LEG_SYMBOL_SFX,
        tag::LEG_SECURITY_ID,
        tag::LEG_SECURITY_ID_SOURCE,
        tag::NO_LEG_SECURITY_ALT_ID,
        tag::LEG_SECURITY_ALT_ID,
        tag::LEG_SECURITY_ALT_ID_SOURCE,
        tag::LEG_PRODUCT,
        tag::LEG_CFI_CODE,
        tag::LEG_SECURITY_TYPE,
        tag::LEG_MATURITY_MONTH_YEAR,
        tag::LEG_MATURITY_DATE,
        tag::LEG_STRIKE_PRICE,
        tag::LEG_OPT_ATTRIBUTE,
        tag::LEG_CONTRACT_MULTIPLIER,
        tag::LEG_COUPON_RATE,
        tag::LEG_SECURITY_EXCHANGE,
        tag::LEG_ISSUER,
        tag::ENCODED_LEG_ISSUER_LEN,
        tag::ENCODED_LEG_ISSUER,
        tag::LEG_SECURITY_DESC,
        tag::ENCODED_LEG_SECURITY_DESC_LEN,
        tag::ENCODED_LEG_SECURITY_DESC,
        tag::LEG_RATIO_QTY,
        tag::LEG_SIDE,
        tag::LEG_CURRENCY,
        tag::LEG_COUNTRY_OF_ISSUE,
        tag::LEG_STATE_OR_PROVINCE_OF_ISSUE,
        tag::LEG_LOCALE_OF_ISSUE,
        tag::LEG_INSTR_REGISTRY,
        tag::LEG_DATED_DATE,
        tag::LEG_POOL,
        tag::LEG_CONTRACT_SETTL_MONTH,
        tag::LEG_INTEREST_ACCRUAL_DATE,
        tag::LEG_QTY,
        tag::LEG_SWAP_TYPE,
        tag::NO_LEG_STIPULATIONS,
        tag::LEG_STIPULATION_TYPE,
        tag::LEG_STIPULATION_VALUE,
        tag::LEG_POSITION_EFFECT,
        tag::LEG_COVERED_OR_UNCOVERED,
        tag::LEG_PRICE,
        tag::LEG_SETTL_TYPE,
        tag::LEG_SETTL_DATE,
        tag::LEG_LAST_PX,
        tag::LEG_REF_ID,
    ],
};

/// NO_UNDERLYINGS (711) — UnderlyingSymbol is the delimiter tag.
pub const UNDERLYINGS: GroupSpec = GroupSpec {
    count_tag: tag::NO_UNDERLYINGS,
    delimiter_tag: tag::UNDERLYING_SYMBOL,
    member_tags: &[
        tag::UNDERLYING_SYMBOL,
        tag::UNDERLYING_SYMBOL_SFX,
        tag::UNDERLYING_SECURITY_ID,
        tag::UNDERLYING_ID_SOURCE,
        tag::UNDERLYING_SECURITY_TYPE,
        tag::UNDERLYING_MATURITY_MONTH_YEAR,
        tag::UNDERLYING_MATURITY_DATE,
        tag::UNDERLYING_PUT_OR_CALL,
        tag::UNDERLYING_STRIKE_PRICE,
        tag::UNDERLYING_OPT_ATTRIBUTE,
        tag::UNDERLYING_CONTRACT_MULTIPLIER,
        tag::UNDERLYING_COUPON_RATE,
        tag::UNDERLYING_SECURITY_EXCHANGE,
        tag::UNDERLYING_ISSUER,
        tag::ENCODED_UNDERLYING_ISSUER_LEN,
        tag::ENCODED_UNDERLYING_ISSUER,
        tag::UNDERLYING_SECURITY_DESC,
        tag::ENCODED_UNDERLYING_SECURITY_DESC_LEN,
        tag::ENCODED_UNDERLYING_SECURITY_DESC,
        tag::UNDERLYING_COUPON_PAYMENT_DATE,
        tag::UNDERLYING_ISSUE_DATE,
        tag::UNDERLYING_REPO_COLLATERAL_SECURITY_TYPE,
        tag::UNDERLYING_REPURCHASE_TERM,
        tag::UNDERLYING_REPURCHASE_RATE,
        tag::UNDERLYING_FACTOR,
        tag::UNDERLYING_CREDIT_RATING,
        tag::UNDERLYING_INSTR_REGISTRY,
        tag::UNDERLYING_COUNTRY_OF_ISSUE,
        tag::UNDERLYING_STATE_OR_PROVINCE_OF_ISSUE,
        tag::UNDERLYING_LOCALE_OF_ISSUE,
        tag::UNDERLYING_REDEMPTION_DATE,
        tag::UNDERLYING_STRIKE_CURRENCY,
        tag::UNDERLYING_SECURITY_SUB_TYPE,
        tag::UNDERLYING_PRODUCT,
        tag::UNDERLYING_CFI_CODE,
        tag::UNDERLYING_CP_PROGRAM,
        tag::UNDERLYING_CP_REG_TYPE,
        tag::UNDERLYING_LAST_PX,
        tag::UNDERLYING_LAST_QTY,
        tag::UNDERLYING_QTY,
        tag::UNDERLYING_SETTL_PRICE,
        tag::UNDERLYING_SETTL_PRICE_TYPE,
        tag::UNDERLYING_DIRTY_PRICE,
        tag::UNDERLYING_END_PRICE,
        tag::UNDERLYING_START_VALUE,
        tag::UNDERLYING_CURRENT_VALUE,
        tag::UNDERLYING_END_VALUE,
        tag::NO_UNDERLYING_SECURITY_ALT_ID,
        tag::UNDERLYING_SECURITY_ALT_ID,
        tag::UNDERLYING_SECURITY_ALT_ID_SOURCE,
        tag::UNDERLYING_STIP_TYPE,
        tag::UNDERLYING_STIP_VALUE,
    ],
};

/// NO_POSITIONS (702) — PosType is the delimiter tag.
pub const POSITIONS: GroupSpec = GroupSpec {
    count_tag: tag::NO_POSITIONS,
    delimiter_tag: tag::POS_TYPE,
    member_tags: &[tag::POS_TYPE, tag::LONG_QTY, tag::SHORT_QTY, tag::POS_QTY_STATUS],
};

/// NO_QUOTE_QUALIFIERS (735) — QuoteQualifier is the delimiter tag.
pub const QUOTE_QUALIFIERS: GroupSpec = GroupSpec {
    count_tag: tag::NO_QUOTE_QUALIFIERS,
    delimiter_tag: tag::QUOTE_QUALIFIER,
    member_tags: &[tag::QUOTE_QUALIFIER],
};

/// NO_POS_AMT (753) — PosAmtType is the delimiter tag.
pub const POS_AMTS: GroupSpec = GroupSpec {
    count_tag: tag::NO_POS_AMT,
    delimiter_tag: tag::POS_AMT_TYPE,
    member_tags: &[tag::POS_AMT_TYPE, tag::POS_AMT],
};

/// NO_NESTED2_PARTY_IDS (756) — Nested2PartyID is the delimiter tag.
pub const NESTED2_PARTY_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_NESTED2_PARTY_IDS,
    delimiter_tag: tag::NESTED2_PARTY_ID,
    member_tags: &[
        tag::NESTED2_PARTY_ID,
        tag::NESTED2_PARTY_ID_SOURCE,
        tag::NESTED2_PARTY_ROLE,
        tag::NESTED2_PARTY_SUB_ID,
    ],
};

/// NO_TRD_REG_TIMESTAMPS (768) — TrdRegTimestamp is the delimiter tag.
pub const TRD_REG_TIMESTAMPS: GroupSpec = GroupSpec {
    count_tag: tag::NO_TRD_REG_TIMESTAMPS,
    delimiter_tag: tag::TRD_REG_TIMESTAMP,
    member_tags: &[
        tag::TRD_REG_TIMESTAMP,
        tag::TRD_REG_TIMESTAMP_TYPE,
        tag::TRD_REG_TIMESTAMP_ORIGIN,
    ],
};

/// NO_SETTL_INST (778) — SettlInstID is the delimiter tag.
pub const SETTL_INST: GroupSpec = GroupSpec {
    count_tag: tag::NO_SETTL_INST,
    delimiter_tag: tag::SETTL_INST_ID,
    member_tags: &[
        tag::SETTL_INST_ID,
        tag::SETTL_INST_TRANS_TYPE,
        tag::SETTL_INST_REF_ID,
        tag::SETTL_INST_MODE,
        tag::SETTL_INST_SOURCE,
        tag::SECURITY_ID,
        tag::SIDE,
        tag::TRANSACT_TIME,
        tag::EFFECTIVE_TIME,
    ],
};

/// NO_SETTL_PARTY_IDS (781) — SettlPartyID is the delimiter tag.
pub const SETTL_PARTY_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_SETTL_PARTY_IDS,
    delimiter_tag: tag::SETTL_PARTY_ID,
    member_tags: &[
        tag::SETTL_PARTY_ID,
        tag::SETTL_PARTY_ID_SOURCE,
        tag::SETTL_PARTY_ROLE,
        tag::SETTL_PARTY_SUB_ID,
        tag::SETTL_PARTY_SUB_ID_TYPE,
    ],
};

/// NO_PARTY_SUB_IDS (802) — PartySubID is the delimiter tag.
pub const PARTY_SUB_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_PARTY_SUB_IDS,
    delimiter_tag: tag::PARTY_SUB_ID,
    member_tags: &[tag::PARTY_SUB_ID, tag::PARTY_SUB_ID_TYPE],
};

/// NO_NESTED_PARTY_SUB_IDS (804) — NestedPartySubID is the delimiter tag.
pub const NESTED_PARTY_SUB_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_NESTED_PARTY_SUB_IDS,
    delimiter_tag: tag::NESTED_PARTY_SUB_ID,
    member_tags: &[tag::NESTED_PARTY_SUB_ID, tag::NESTED_PARTY_SUB_ID_TYPE],
};

/// NO_NESTED2_PARTY_SUB_IDS (806) — Nested2PartySubID is the delimiter tag.
pub const NESTED2_PARTY_SUB_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_NESTED2_PARTY_SUB_IDS,
    delimiter_tag: tag::NESTED2_PARTY_SUB_ID,
    member_tags: &[tag::NESTED2_PARTY_SUB_ID, tag::NESTED2_PARTY_SUB_ID_TYPE],
};

/// NO_ALT_MD_SOURCE (816) — AltMDSourceID is the delimiter tag.
pub const ALT_MD_SOURCES: GroupSpec = GroupSpec {
    count_tag: tag::NO_ALT_MD_SOURCE,
    delimiter_tag: tag::ALT_MD_SOURCE_ID,
    member_tags: &[tag::ALT_MD_SOURCE_ID],
};

/// NO_CAPACITIES (862) — OrderCapacity is the delimiter tag.
pub const CAPACITIES: GroupSpec = GroupSpec {
    count_tag: tag::NO_CAPACITIES,
    delimiter_tag: tag::ORDER_CAPACITY,
    member_tags: &[tag::ORDER_CAPACITY, tag::ORDER_CAPACITY_QTY],
};

/// NO_EVENTS (864) — EventType is the delimiter tag.
pub const EVENTS: GroupSpec = GroupSpec {
    count_tag: tag::NO_EVENTS,
    delimiter_tag: tag::EVENT_TYPE,
    member_tags: &[tag::EVENT_TYPE, tag::EVENT_DATE, tag::EVENT_PX, tag::EVENT_TEXT],
};

/// NO_INSTR_ATTRIB (870) — InstrAttribType is the delimiter tag.
pub const INSTR_ATTRIB: GroupSpec = GroupSpec {
    count_tag: tag::NO_INSTR_ATTRIB,
    delimiter_tag: tag::INSTR_ATTRIB_TYPE,
    member_tags: &[tag::INSTR_ATTRIB_TYPE, tag::INSTR_ATTRIB_VALUE],
};

/// NO_UNDERLYING_STIPS (887) — UnderlyingStipType is the delimiter tag.
pub const UNDERLYING_STIPS: GroupSpec = GroupSpec {
    count_tag: tag::NO_UNDERLYING_STIPS,
    delimiter_tag: tag::UNDERLYING_STIP_TYPE,
    member_tags: &[tag::UNDERLYING_STIP_TYPE, tag::UNDERLYING_STIP_VALUE],
};

/// NO_TRADES (897) — TradeReportID is the delimiter tag.
pub const TRADES: GroupSpec = GroupSpec {
    count_tag: tag::NO_TRADES,
    delimiter_tag: tag::TRADE_REPORT_ID,
    member_tags: &[tag::TRADE_REPORT_ID, tag::SECONDARY_TRADE_REPORT_ID],
};

/// NO_COMP_IDS (936) — RefCompID is the delimiter tag.
pub const COMP_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_COMP_IDS,
    delimiter_tag: tag::REF_COMP_ID,
    member_tags: &[
        tag::REF_COMP_ID,
        tag::REF_SUB_ID,
        tag::STATUS_VALUE,
        tag::STATUS_TEXT,
    ],
};

/// NO_COLL_INQUIRY_QUALIFIER (938) — CollInquiryQualifier is the delimiter tag.
pub const COLL_INQUIRY_QUALIFIERS: GroupSpec = GroupSpec {
    count_tag: tag::NO_COLL_INQUIRY_QUALIFIER,
    delimiter_tag: tag::COLL_INQUIRY_QUALIFIER,
    member_tags: &[tag::COLL_INQUIRY_QUALIFIER],
};

/// NO_NESTED3_PARTY_IDS (948) — Nested3PartyID is the delimiter tag.
pub const NESTED3_PARTY_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_NESTED3_PARTY_IDS,
    delimiter_tag: tag::NESTED3_PARTY_ID,
    member_tags: &[
        tag::NESTED3_PARTY_ID,
        tag::NESTED3_PARTY_ID_SOURCE,
        tag::NESTED3_PARTY_ROLE,
        tag::NESTED3_PARTY_SUB_ID,
        tag::NESTED3_PARTY_SUB_ID_TYPE,
    ],
};

/// NO_LEG_SECURITY_ALT_ID (604) — LegSecurityAltID is the delimiter tag.
pub const LEG_SECURITY_ALT_IDS: GroupSpec = GroupSpec {
    count_tag: tag::NO_LEG_SECURITY_ALT_ID,
    delimiter_tag: tag::LEG_SECURITY_ALT_ID,
    member_tags: &[tag::LEG_SECURITY_ALT_ID, tag::LEG_SECURITY_ALT_ID_SOURCE],
};

/// NO_LEG_STIPULATIONS (683) — LegStipulationType is the delimiter tag.
pub const LEG_STIPULATIONS: GroupSpec = GroupSpec {
    count_tag: tag::NO_LEG_STIPULATIONS,
    delimiter_tag: tag::LEG_STIPULATION_TYPE,
    member_tags: &[tag::LEG_STIPULATION_TYPE, tag::LEG_STIPULATION_VALUE],
};

/// NO_LEG_ALLOCS (670) — LegAllocAccount is the delimiter tag.
pub const LEG_ALLOCS: GroupSpec = GroupSpec {
    count_tag: tag::NO_LEG_ALLOCS,
    delimiter_tag: tag::LEG_ALLOC_ACCOUNT,
    member_tags: &[
        tag::LEG_ALLOC_ACCOUNT,
        tag::LEG_INDIVIDUAL_ALLOC_ID,
        tag::LEG_ALLOC_QTY,
        tag::LEG_ALLOC_ACCT_ID_SOURCE,
        tag::LEG_SETTL_CURRENCY,
    ],
};

/// NO_HOPS (627) — HopCompID is the delimiter tag.
pub const HOPS: GroupSpec = GroupSpec {
    count_tag: tag::NO_HOPS,
    delimiter_tag: tag::HOP_COMP_ID,
    member_tags: &[tag::HOP_COMP_ID, tag::HOP_SENDING_TIME, tag::HOP_REF_ID],
};

/// NO_CLEARING_INSTRUCTIONS (576) — ClearingInstruction is the delimiter tag.
pub const CLEARING_INSTRUCTIONS: GroupSpec = GroupSpec {
    count_tag: tag::NO_CLEARING_INSTRUCTIONS,
    delimiter_tag: tag::CLEARING_INSTRUCTION,
    member_tags: &[tag::CLEARING_INSTRUCTION],
};

/// All built-in FIX 4.4 group specs (superset of `FIX42_GROUPS`).
///
/// Includes all FIX 4.2 groups plus the groups introduced in FIX 4.4,
/// so this array alone covers every repeating group that can appear in
/// a FIX 4.4 message.
pub const FIX44_GROUPS: &[&GroupSpec] = &[
    // -- FIX 4.2 groups (inherited) --
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
    // -- FIX 4.4 additions --
    &PARTY_IDS,
    &SECURITY_ALT_IDS,
    &UNDERLYING_SECURITY_ALT_IDS,
    &REGIST_DTLS,
    &DISTRIB_INSTS,
    &CONT_AMTS,
    &NESTED_PARTY_IDS,
    &SIDES,
    &SECURITY_TYPES,
    &AFFECTED_ORDERS,
    &LEGS,
    &UNDERLYINGS,
    &POSITIONS,
    &QUOTE_QUALIFIERS,
    &POS_AMTS,
    &NESTED2_PARTY_IDS,
    &TRD_REG_TIMESTAMPS,
    &SETTL_INST,
    &SETTL_PARTY_IDS,
    &PARTY_SUB_IDS,
    &NESTED_PARTY_SUB_IDS,
    &NESTED2_PARTY_SUB_IDS,
    &ALT_MD_SOURCES,
    &CAPACITIES,
    &EVENTS,
    &INSTR_ATTRIB,
    &UNDERLYING_STIPS,
    &TRADES,
    &COMP_IDS,
    &COLL_INQUIRY_QUALIFIERS,
    &NESTED3_PARTY_IDS,
    &LEG_SECURITY_ALT_IDS,
    &LEG_STIPULATIONS,
    &LEG_ALLOCS,
    &HOPS,
    &CLEARING_INSTRUCTIONS,
];

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

    /// Return an iterator over the instances of a repeating group nested inside
    /// this group instance. Mirrors [`Message::groups`] exactly.
    ///
    /// Because this group's `offsets` slice is already bounded to this parent
    /// instance, the nested iterator cannot escape into sibling parent instances.
    ///
    /// Returns an empty iterator if the nested count tag is absent or zero.
    #[inline]
    pub fn groups(&self, spec: &GroupSpec) -> GroupIter<'a> {
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

    // -----------------------------------------------------------------------
    // Nested groups — Group::groups()
    // -----------------------------------------------------------------------

    #[test]
    fn nested_group_single_parent_single_child() {
        // SIDES=1, one side with NO_CONT_AMTS=1
        let raw = fix("552=1|54=1|37=ORD1|518=1|519=1|520=100.00|521=USD|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let side = msg.groups(&SIDES).next().expect("expected one side");
        assert_eq!(side.find(tag::SIDE).unwrap().value, b"1");
        assert_eq!(side.find(tag::ORDER_ID).unwrap().value, b"ORD1");

        let mut nested = side.groups(&CONT_AMTS);
        let ca = nested.next().expect("expected one CONT_AMTS instance");
        assert!(nested.next().is_none());
        assert_eq!(ca.find(tag::CONT_AMT_TYPE).unwrap().value, b"1");
        assert_eq!(ca.find(tag::CONT_AMT_VALUE).unwrap().value, b"100.00");
        assert_eq!(ca.find(tag::CONT_AMT_CURR).unwrap().value, b"USD");
    }

    #[test]
    fn nested_group_single_parent_multiple_children() {
        // SIDES=1, one side with NO_CONT_AMTS=2
        let raw = fix("552=1|54=1|518=2|519=1|520=100.00|521=USD|519=2|520=50.00|521=EUR|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let side = msg.groups(&SIDES).next().unwrap();
        let cont_amts: Vec<_> = side.groups(&CONT_AMTS).collect();
        assert_eq!(cont_amts.len(), 2);
        assert_eq!(cont_amts[0].find(tag::CONT_AMT_TYPE).unwrap().value, b"1");
        assert_eq!(cont_amts[0].find(tag::CONT_AMT_VALUE).unwrap().value, b"100.00");
        assert_eq!(cont_amts[0].find(tag::CONT_AMT_CURR).unwrap().value, b"USD");
        assert_eq!(cont_amts[1].find(tag::CONT_AMT_TYPE).unwrap().value, b"2");
        assert_eq!(cont_amts[1].find(tag::CONT_AMT_VALUE).unwrap().value, b"50.00");
        assert_eq!(cont_amts[1].find(tag::CONT_AMT_CURR).unwrap().value, b"EUR");
    }

    #[test]
    fn nested_group_multiple_parents_each_with_children() {
        // SIDES=2: side1 has 1 CONT_AMT, side2 has 2 CONT_AMTs — critical boundary test
        let raw = fix(
            "552=2|\
             54=1|518=1|519=1|520=100.00|521=USD|\
             54=2|518=2|519=1|520=5.00|521=EUR|519=2|520=3.00|521=GBP|",
        );
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let sides: Vec<_> = msg.groups(&SIDES).collect();
        assert_eq!(sides.len(), 2);

        let side1_cas: Vec<_> = sides[0].groups(&CONT_AMTS).collect();
        assert_eq!(side1_cas.len(), 1);
        assert_eq!(side1_cas[0].find(tag::CONT_AMT_VALUE).unwrap().value, b"100.00");
        assert_eq!(side1_cas[0].find(tag::CONT_AMT_CURR).unwrap().value, b"USD");

        let side2_cas: Vec<_> = sides[1].groups(&CONT_AMTS).collect();
        assert_eq!(side2_cas.len(), 2);
        assert_eq!(side2_cas[0].find(tag::CONT_AMT_VALUE).unwrap().value, b"5.00");
        assert_eq!(side2_cas[0].find(tag::CONT_AMT_CURR).unwrap().value, b"EUR");
        assert_eq!(side2_cas[1].find(tag::CONT_AMT_VALUE).unwrap().value, b"3.00");
        assert_eq!(side2_cas[1].find(tag::CONT_AMT_CURR).unwrap().value, b"GBP");
    }

    #[test]
    fn nested_group_two_different_nested_groups_in_same_parent() {
        // SIDES=1, one side with NO_CONT_AMTS=1 and NO_MISC_FEES=1
        let raw = fix("552=1|54=1|518=1|519=1|520=100.00|521=USD|136=1|137=10.00|138=EUR|139=1|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let side = msg.groups(&SIDES).next().unwrap();

        let cont_amts: Vec<_> = side.groups(&CONT_AMTS).collect();
        assert_eq!(cont_amts.len(), 1);
        assert_eq!(cont_amts[0].find(tag::CONT_AMT_VALUE).unwrap().value, b"100.00");
        assert_eq!(cont_amts[0].find(tag::CONT_AMT_CURR).unwrap().value, b"USD");

        let misc_fees: Vec<_> = side.groups(&MISC_FEES).collect();
        assert_eq!(misc_fees.len(), 1);
        assert_eq!(misc_fees[0].find(tag::MISC_FEE_AMT).unwrap().value, b"10.00");
        assert_eq!(misc_fees[0].find(tag::MISC_FEE_CURR).unwrap().value, b"EUR");
        assert_eq!(misc_fees[0].find(tag::MISC_FEE_TYPE).unwrap().value, b"1");
    }

    #[test]
    fn nested_group_count_tag_absent_yields_empty() {
        // SIDES=1, one side with no CONT_AMTS tag at all
        let raw = fix("552=1|54=1|37=ORD1|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let side = msg.groups(&SIDES).next().unwrap();
        assert!(side.groups(&CONT_AMTS).next().is_none());
    }

    #[test]
    fn nested_group_count_zero_yields_empty() {
        // SIDES=1, one side with NO_CONT_AMTS=0
        let raw = fix("552=1|54=1|37=ORD1|518=0|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let side = msg.groups(&SIDES).next().unwrap();
        assert!(side.groups(&CONT_AMTS).next().is_none());
    }

    #[test]
    fn nested_group_iter_size_hint() {
        // SIDES=1, one side with NO_CONT_AMTS=3
        let raw = fix("552=1|54=1|518=3|519=1|520=10.00|519=2|520=20.00|519=3|520=30.00|");
        let mut dec = Decoder::new();
        let msg = dec.decode(&raw).unwrap();

        let side = msg.groups(&SIDES).next().unwrap();
        let mut nested = side.groups(&CONT_AMTS);
        assert_eq!(nested.size_hint(), (3, Some(3)));
        nested.next();
        assert_eq!(nested.size_hint(), (2, Some(2)));
        nested.next();
        assert_eq!(nested.size_hint(), (1, Some(1)));
        nested.next();
        assert_eq!(nested.size_hint(), (0, Some(0)));
        assert!(nested.next().is_none());
    }
}
