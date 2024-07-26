//! # Tests based on CQG's template
//!
//! See: https://help.cqg.com/apihelp/#!Documents/quotesdirectfixfast.htm
//!
use serde_derive::{Deserialize, Serialize};

use fastlib::{Decimal, Decoder, Encoder};

/// Message templates must be implements as `enum`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Message {
    MDSecurityDefinition(SecurityDefinition),
    MDHeartbeat(Heartbeat),
    MDLogon(Logon),
    MDLogout(Logout),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgHeader {
    #[serde(rename = "ApplVerID")]
    appl_ver_id: char,
    #[serde(rename = "SenderCompID")]
    sender_comp_id: String,
    #[serde(rename = "MsgSeqNum")]
    msg_seq_num: u32,
    #[serde(rename = "SendingTime")]
    sending_time: u64,
}

/// Static template references can be flattened with `#[serde(flatten)]` or inlined as is.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Heartbeat {
    message_type: char,

    // static templateRef as flattened struct
    #[serde(flatten)]
    msg_header: MsgHeader,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Logon {
    message_type: char,

    // static templateRef as inlined fields
    #[serde(rename = "ApplVerID")]
    appl_ver_id: char,
    #[serde(rename = "SenderCompID")]
    sender_comp_id: String,
    #[serde(rename = "MsgSeqNum")]
    msg_seq_num: u32,
    #[serde(rename = "SendingTime")]
    sending_time: u64,

    encrypt_method: u32,
    heartbeat_int: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Logout {
    message_type: char,
    #[serde(flatten)]
    msg_header: MsgHeader,
    text: Option<String>,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SecurityDefinition {
    message_type: char,
    #[serde(flatten)]
    msg_header: MsgHeader,
    tot_num_reports: u32,
    events: Option<Vec<Event>>,
    security_group: Option<String>,
    symbol: Option<String>,
    security_name: String,
    security_desc: String,
    #[serde(rename = "SecurityID")]
    security_id: u32,
    #[serde(rename = "SecurityIDSource")]
    security_id_source: u32,
    #[serde(rename = "CFICode")]
    cfi_code: String,
    security_exchange: Option<String>,
    #[serde(rename = "CQGSecurityName")]
    cqg_security_name: Option<String>,
    strike_price: Option<Decimal>,
    strike_currency: Option<String>,
    currency: Option<String>,
    settl_currency: Option<String>,
    #[serde(rename = "MDFeedTypes")]
    md_feed_types: Option<Vec<FeedType>>,
    instr_attrib: Option<Vec<InstrAttrib>>,
    maturity_month_year: Option<u64>,
    min_price_increment: Option<f64>,
    min_price_increment_amount: Option<f64>,
    display_factor: Option<Decimal>,
    #[serde(rename = "ApplID")]
    appl_id: String,
    most_active_flag: Option<String>,
    connections: Vec<Connection>,
    trading_sessions: Vec<TradingSession>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Event {
    event_type: u32,
    event_date: u64,
    event_time: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FeedType {
    #[serde(rename = "MDFeedType")]
    feed_type: String,
    market_depth: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InstrAttrib {
    instr_attrib_type: u64,
    instr_attrib_value: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Connection {
    connection_type: u32,
    #[serde(rename = "ConnectionIPAddress")]
    connection_ip_address: String,
    connection_port_number: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TradingSession {
    trade_date: u64,
    trad_ses_start_time: u64,
    trad_ses_open_time: u64,
    trad_ses_close_time: u64,
    trad_ses_end_time: u64,
}

const DEFINITION: &str = include_str!("templates.xml");

fn do_tests_seq(raw: Vec<Vec<u8>>, data: Vec<Message>) {
    let mut e = Encoder::new_from_xml(DEFINITION).unwrap();
    let mut d = Decoder::new_from_xml(DEFINITION).unwrap();

    for (i, (raw, data)) in raw.into_iter().zip(data).enumerate() {
        let res = fastlib::to_vec(&mut e, &data).unwrap();
        assert_eq!(res, raw, "encode failed #{}", i + 1);

        let msg: Message = fastlib::from_vec(&mut d, raw).unwrap();
        assert_eq!(msg, data, "decode failed #{}", i + 1);
    }
}

#[test]
fn test_heartbeats() {
    do_tests_seq(
        vec![
            vec![0xc0, 0x84, 0x81, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2c, 0x58, 0x80],
            vec![0x80, 0x82, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2d, 0x26, 0x90],
            vec![0x80, 0x83, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2d, 0x74, 0xa0],
        ],
        vec![
            Message::MDHeartbeat(Heartbeat {
                message_type: '0',
                msg_header: MsgHeader {
                    appl_ver_id: '8',
                    sender_comp_id: "CQG".to_string(),
                    msg_seq_num: 1,
                    sending_time: 20240606000000000,
                },
            }),
            Message::MDHeartbeat(Heartbeat {
                message_type: '0',
                msg_header: MsgHeader {
                    appl_ver_id: '8',
                    sender_comp_id: "CQG".to_string(),
                    msg_seq_num: 2,
                    sending_time: 20240606000010000,
                },
            }),
            Message::MDHeartbeat(Heartbeat {
                message_type: '0',
                msg_header: MsgHeader {
                    appl_ver_id: '8',
                    sender_comp_id: "CQG".to_string(),
                    msg_seq_num: 3,
                    sending_time: 20240606000020000,
                },
            }),
        ],
    )
}

#[test]
fn test_logon() {
    do_tests_seq(
        vec![
            vec![0xc0, 0x85, 0x81, 0x23, 0x7a, 0x17, 0x15, 0x7a, 0x4d, 0x51, 0x9d, 0x8a],
        ],
        vec![
            Message::MDLogon(Logon {
                message_type: 'A',
                appl_ver_id: '8',
                sender_comp_id: "CQG".to_string(),
                msg_seq_num: 1,
                sending_time: 20240606212352157,
                encrypt_method: 0,
                heartbeat_int: 10,
            }),
        ],
    )
}

#[test]
fn test_logout() {
    do_tests_seq(
        vec![
            vec![0xc0, 0x86, 0x83, 0x23, 0x7a, 0x1a, 0x19, 0x36, 0x3b, 0x5f, 0xc8, 0x52, 0x65, 0x71, 0x75, 0x65,
                 0x73, 0x74, 0x20, 0x74, 0x69, 0x6d, 0x65, 0x6f, 0x75, 0xf4],
        ],
        vec![
            Message::MDLogout(Logout {
                message_type: '5',
                msg_header: MsgHeader {
                    appl_ver_id: '8',
                    sender_comp_id: "CQG".to_string(),
                    msg_seq_num: 3,
                    sending_time: 20240710222409672,
                },
                text: Some("Request timeout".to_string()),
            }),
        ],
    )
}

#[test]
fn test_definitions() {
    do_tests_seq(
        vec![
            vec![0x7f, 0x57, 0xc0, 0x82, 0x07, 0xc4, 0x23, 0x7a, 0x17, 0x15, 0x7a, 0x4d, 0x59, 0x83, 0x07, 0xc6,
                 0x82, 0x80, 0x09, 0x53, 0x35, 0xe9, 0x00, 0x68, 0x73, 0x5e, 0x80, 0x4d, 0x42, 0x54, 0x53, 0x31,
                 0xb3, 0x4d, 0x42, 0x54, 0x53, 0x31, 0x33, 0x43, 0x31, 0x30, 0xb0, 0x4d, 0x69, 0x63, 0x72, 0x6f,
                 0x20, 0x42, 0x69, 0x74, 0x63, 0x6f, 0x69, 0x6e, 0x20, 0x52, 0x65, 0x76, 0x65, 0x72, 0x73, 0x65,
                 0x20, 0x43, 0x61, 0x6c, 0x20, 0x53, 0x70, 0x72, 0x65, 0x61, 0xe4, 0x4d, 0x42, 0x54, 0x53, 0x31,
                 0x33, 0x58, 0x32, 0xb4, 0x1c, 0x79, 0x58, 0xfe, 0x46, 0x58, 0x58, 0x58, 0x58, 0xd8, 0x47, 0x4c,
                 0x42, 0xd8, 0x46, 0x2e, 0x55, 0x53, 0x2e, 0x4d, 0x42, 0x54, 0x57, 0x31, 0x33, 0x58, 0x32, 0xb4,
                 0x81, 0x80, 0x55, 0x53, 0xc4, 0x83, 0x80, 0x80, 0xc0, 0x43, 0x51, 0x47, 0xc9, 0x81, 0x82, 0xc0,
                 0x07, 0xeb, 0x31, 0x30, 0xb0, 0x0c, 0x2d, 0xac, 0x81, 0x81, 0xff, 0x81, 0x81, 0x81, 0xb4, 0x84,
                 0x81, 0x32, 0x33, 0x39, 0x2e, 0x32, 0x34, 0x36, 0x2e, 0x35, 0x2e, 0xb4, 0x55, 0xfc, 0x82, 0x32,
                 0x33, 0x39, 0x2e, 0x32, 0x34, 0x36, 0x2e, 0x36, 0x2e, 0xb4, 0x5d, 0xe4, 0x83, 0x31, 0x30, 0x2e,
                 0x31, 0x2e, 0x30, 0x2e, 0x31, 0x32, 0xb0, 0x4e, 0x90, 0x83, 0x31, 0x30, 0x2e, 0x31, 0x2e, 0x30,
                 0x2e, 0x31, 0x32, 0xb0, 0x4e, 0x91, 0x86, 0x09, 0x53, 0x31, 0x93, 0x23, 0x7a, 0x14, 0x7a, 0x6e,
                 0x50, 0x46, 0x80, 0x23, 0x7a, 0x14, 0x7a, 0x6a, 0x49, 0x5f, 0xe0, 0x23, 0x7a, 0x14, 0x7e, 0x46,
                 0x59, 0x2d, 0x80, 0x23, 0x7a, 0x14, 0x7e, 0x46, 0x59, 0x2d, 0x80, 0x00, 0xc8, 0x02, 0x0c, 0x1c,
                 0x23, 0x20, 0x80, 0x02, 0x0c, 0x1c, 0x23, 0x20, 0x80, 0x02, 0x0c, 0x1c, 0x23, 0x20, 0x80, 0x02,
                 0x0c, 0x1c, 0x23, 0x20, 0x80, 0x81, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80,
                 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x81, 0x03, 0x5c, 0x6b, 0x14, 0x80,
                 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x81,
                 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03,
                 0x5c, 0x6b, 0x14, 0x80, 0x81, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x03,
                 0x5c, 0x6b, 0x14, 0x80, 0x03, 0x5c, 0x6b, 0x14, 0x80, 0x80, 0x80, 0x80],
        ],
        vec![
            Message::MDSecurityDefinition(SecurityDefinition {
                message_type: 'd',
                msg_header: MsgHeader {
                    appl_ver_id: '8',
                    sender_comp_id: "CQG".to_string(),
                    msg_seq_num: 964,
                    sending_time: 20240606212353155,
                },
                tot_num_reports: 966,
                events: Some(vec![
                    Event {
                        event_type: 7,
                        event_date: 20241129,
                        event_time: 220000000,
                    }
                ]),
                security_group: Some("MBTS13".to_string()),
                symbol: Some("MBTS13C100".to_string()),
                security_name: "Micro Bitcoin Reverse Cal Spread".to_string(),
                security_desc: "MBTS13X24".to_string(),
                security_id: 60714110,
                security_id_source: 100,
                cfi_code: "FXXXXX".to_string(),
                security_exchange: Some("GLBX".to_string()),
                cqg_security_name: Some("F.US.MBTW13X24".to_string()),
                strike_price: Some(Decimal::default()),
                strike_currency: None,
                currency: Some("USD".to_string()),
                settl_currency: None,
                md_feed_types: Some(vec![
                    FeedType {
                        feed_type: "CQGC".to_string(),
                        market_depth: 0,
                    },
                    FeedType {
                        feed_type: "CQGI".to_string(),
                        market_depth: 1,
                    },
                ]),
                instr_attrib: Some(vec![
                    InstrAttrib {
                        instr_attrib_type: 1003,
                        instr_attrib_value: Some("100".to_string()),
                    }
                ]),
                maturity_month_year: Some(202411),
                min_price_increment: Some(1.0),
                min_price_increment_amount: Some(0.1),
                display_factor: Some(Decimal::new(0, 1)),
                appl_id: "4".to_string(),
                most_active_flag: None,
                connections: vec![
                    Connection {
                        connection_type: 1,
                        connection_ip_address: "239.246.5.4".to_string(),
                        connection_port_number: 11004,
                    },
                    Connection {
                        connection_type: 2,
                        connection_ip_address: "239.246.6.4".to_string(),
                        connection_port_number: 12004,
                    },
                    Connection {
                        connection_type: 3,
                        connection_ip_address: "10.1.0.120".to_string(),
                        connection_port_number: 10000,
                    },
                    Connection {
                        connection_type: 3,
                        connection_ip_address: "10.1.0.120".to_string(),
                        connection_port_number: 10001,
                    },
                ],
                trading_sessions: vec![
                    TradingSession {
                        trade_date: 20240531,
                        trad_ses_start_time: 20240530220000000,
                        trad_ses_open_time: 20240530211500000,
                        trad_ses_close_time: 20240531210000000,
                        trad_ses_end_time: 20240531210000000,
                    },
                    TradingSession {
                        trade_date: 20240603,
                        trad_ses_start_time: 20240602220000000,
                        trad_ses_open_time: 20240602211500000,
                        trad_ses_close_time: 20240603210000000,
                        trad_ses_end_time: 20240603210000000,
                    },
                    TradingSession {
                        trade_date: 20240604,
                        trad_ses_start_time: 20240603220000000,
                        trad_ses_open_time: 20240603211500000,
                        trad_ses_close_time: 20240604210000000,
                        trad_ses_end_time: 20240604210000000,
                    },
                    TradingSession {
                        trade_date: 20240605,
                        trad_ses_start_time: 20240604220000000,
                        trad_ses_open_time: 20240604211500000,
                        trad_ses_close_time: 20240605210000000,
                        trad_ses_end_time: 20240605210000000,
                    },
                    TradingSession {
                        trade_date: 20240606,
                        trad_ses_start_time: 20240605220000000,
                        trad_ses_open_time: 20240605211500000,
                        trad_ses_close_time: 20240606210000000,
                        trad_ses_end_time: 20240606210000000,
                    },
                    TradingSession {
                        trade_date: 20240607,
                        trad_ses_start_time: 20240606220000000,
                        trad_ses_open_time: 20240606211500000,
                        trad_ses_close_time: 20240607210000000,
                        trad_ses_end_time: 20240607210000000,
                    },
                ],
            })
        ],
    )
}
