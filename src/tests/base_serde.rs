//! # Tests partially adopted from GoFAST library
//!
//! See: https://github.com/mcsakoff/goFAST/tree/main
//!
use serde_derive::{Deserialize, Serialize};

use crate::{Decimal, Decoder, Encoder, from_vec};
use crate::ser::to_vec;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Message {
    Integer(IntegerMsg),
    String(StringMsg),
    ByteVector(BytesMsg),
    Decimal(DecimalMsg),
    Sequence(SequenceMsg),
    Group(GroupMsg),
    RefData(RefDataMsg),
    StaticReference(StaticReferenceMsg),
    DynamicReference(DynamicReferenceMsg),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IntegerMsg {
    mandatory_uint32: u32,
    optional_uint32: Option<u32>,
    mandatory_uint64: u64,
    optional_uint64: Option<u64>,
    mandatory_int32: i32,
    optional_int32: Option<i32>,
    mandatory_int64: i64,
    optional_int64: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StringMsg {
    mandatory_ascii: String,
    optional_ascii: Option<String>,
    mandatory_unicode: String,
    optional_unicode: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BytesMsg {
    #[serde(with = "serde_bytes")]
    mandatory_vector: Vec<u8>,
    #[serde(with = "serde_bytes")]
    optional_vector: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DecimalMsg {
    copy_decimal: Option<f64>,
    mandatory_decimal: Decimal,
    individual_decimal: f64,
    individual_decimal_opt: Option<f64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InnerSequenceItem {
    inner_test_data: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OuterSequenceItem {
    outer_test_data: u32,
    inner_sequence: Option<Vec<InnerSequenceItem>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NextOuterSequenceItem {
    next_outer_test_data: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SequenceMsg {
    test_data: u32,
    outer_sequence: Vec<OuterSequenceItem>,
    next_outer_sequence: Vec<NextOuterSequenceItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GroupMsg {
    test_data: u32,
    outer_group: OuterGroup,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OuterGroup {
    outer_test_data: u32,
    inner_group: Option<InnerGroup>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InnerGroup {
    inner_test_data: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RefDataMsg {
    test_data: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StaticReferenceMsg {
    pre_ref_data: u32,
    #[serde(flatten)]
    ref_data: RefDataMsg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DynamicReferenceMsg {
    pre_ref_data: u32,
    #[serde(rename = "templateRef:0")]
    ref0: Box<Message>,
}

const DEFINITION: &str = include_str!("templates/base.xml");

fn do_test(raw: Vec<u8>, data: Message) {
    {
        let mut d = Decoder::new_from_xml(DEFINITION).unwrap();
        let msg: Message = from_vec(&mut d, raw.clone()).unwrap();
        assert_eq!(msg, data, "decode mismatch");
    }
    {
        let mut e = Encoder::new_from_xml(DEFINITION).unwrap();
        let res = to_vec(&mut e, &data).unwrap();
        assert_eq!(res, raw, "encode mismatch");
    }
}

#[test]
fn decode_integers() {
    do_test(
        vec![0xc0, 0x81, 0x83, 0x85, 0x25, 0x20, 0x2f, 0x47, 0xfe, 0x25, 0x20, 0x2f, 0x48, 0x80, 0x85, 0x87, 0x8, 0x23, 0x51, 0x57, 0x8d, 0x8, 0x23, 0x51, 0x57, 0x8f],
        Message::Integer(IntegerMsg {
            mandatory_uint32: 3,
            optional_uint32: Some(4),
            mandatory_uint64: 9999999998,
            optional_uint64: Some(9999999999),
            mandatory_int32: 5,
            optional_int32: Some(6),
            mandatory_int64: 2222222221,
            optional_int64: Some(2222222222),
        }),
    )
}

#[test]
fn decode_strings() {
    do_test(
        vec![0xc0, 0x82, 0x61, 0x62, 0xe3, 0x64, 0x65, 0xe6, 0x83, 0x67, 0x68, 0x69, 0x84, 0x6b, 0x6c, 0x6d],
        Message::String(StringMsg {
            mandatory_ascii: "abc".to_string(),
            optional_ascii: Some("def".to_string()),
            mandatory_unicode: "ghi".to_string(),
            optional_unicode: Some("klm".to_string()),
        }),
    )
}

#[test]
fn decode_bytes() {
    do_test(
        vec![0xc0, 0x83, 0x81, 0xc1, 0x82, 0xb3],
        Message::ByteVector(BytesMsg {
            mandatory_vector: vec![0xc1],
            optional_vector: Some(vec![0xb3]),
        }),
    )
}

#[test]
fn decode_decimals_1() {
    do_test(
        vec![0xf8, 0x84, 0xfe, 0x4, 0x83, 0xff, 0xc, 0x8a, 0xfc, 0xa0, 0xff, 0x0, 0xef],
        Message::Decimal(DecimalMsg {
            copy_decimal: Some(5.15),
            mandatory_decimal: Decimal::new(-1, 1546),
            individual_decimal: 0.0032,
            individual_decimal_opt: Some(11.1),
        }),
    )
}

#[test]
fn decode_decimals_2() {
    do_test(
        vec![0xf8, 0x84, 0xfe, 0x4, 0x83, 0xff, 0xc, 0x8a, 0xfc, 0xa0, 0x80],
        Message::Decimal(DecimalMsg {
            copy_decimal: Some(5.15),
            mandatory_decimal: Decimal::new(-1, 1546),
            individual_decimal: 0.0032,
            individual_decimal_opt: None,
        }),
    )
}

#[test]
fn decode_sequence_1() {
    do_test(
        vec![0xc0, 0x85, 0x81, 0x81, 0x82, 0x83, 0x83, 0x84, 0x81, 0xc0, 0x82],
        Message::Sequence(SequenceMsg {
            test_data: 1,
            outer_sequence: vec![
                OuterSequenceItem {
                    outer_test_data: 2,
                    inner_sequence: Some(vec![
                        InnerSequenceItem {
                            inner_test_data: 3,
                        },
                        InnerSequenceItem {
                            inner_test_data: 4,
                        },
                    ]),
                }
            ],
            next_outer_sequence: vec![
                NextOuterSequenceItem {
                    next_outer_test_data: 2,
                },
            ],
        }),
    )
}

#[test]
fn decode_sequence_2() {
    do_test(
        vec![0xc0, 0x85, 0x81, 0x81, 0x82, 0x80, 0x81, 0xc0, 0x82],
        Message::Sequence(SequenceMsg {
            test_data: 1,
            outer_sequence: vec![
                OuterSequenceItem {
                    outer_test_data: 2,
                    inner_sequence: None,
                }
            ],
            next_outer_sequence: vec![
                NextOuterSequenceItem {
                    next_outer_test_data: 2,
                },
            ],
        }),
    )
}

#[test]
fn decode_group_1() {
    do_test(
        vec![0xc0, 0x86, 0x81, 0xc0, 0x82, 0x83],
        Message::Group(GroupMsg {
            test_data: 1,
            outer_group: OuterGroup {
                outer_test_data: 2,
                inner_group: Some(InnerGroup {
                    inner_test_data: 3,
                }),
            },
        }),
    )
}

#[test]
fn decode_group_2() {
    do_test(
        vec![0xc0, 0x86, 0x81, 0x80, 0x82],
        Message::Group(GroupMsg {
            test_data: 1,
            outer_group: OuterGroup {
                outer_test_data: 2,
                inner_group: None,
            },
        }),
    )
}

#[test]
fn decode_static_reference() {
    do_test(
        vec![0xe0, 0x88, 0x86, 0x87],
        Message::StaticReference(StaticReferenceMsg {
            pre_ref_data: 6,
            ref_data: RefDataMsg {
                test_data: 7,
            },
        }),
    )
}

#[test]
fn decode_dynamic_reference() {
    do_test(
        vec![0xc0, 0x89, 0x86, 0xe0, 0x87, 0x85],
        Message::DynamicReference(DynamicReferenceMsg {
            pre_ref_data: 6,
            ref0: Box::new(Message::RefData(RefDataMsg {
                test_data: 5,
            })),
        }),
    )
}
