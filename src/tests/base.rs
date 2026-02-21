//! # Tests partially adopted from GoFAST library
//!
//! See: https://github.com/mcsakoff/goFAST/tree/main
//!
use std::io::Cursor;

use hashbrown::HashMap;

use crate::decoder::decoder::Decoder;
use crate::encoder::encoder::Encoder;
use crate::model::value::ValueData;
use crate::model::{ModelFactory, ModelVisitor};
use crate::{Decimal, Error};

use super::*;

#[test]
fn parse_xml_template() {
    let d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    test_templates(
        &d,
        &vec![
            TestTemplate {
                id: 1,
                name: "Integer",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "MandatoryUint32",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UInt32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 2,
                        name: "OptionalUint32",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::UInt32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 3,
                        name: "MandatoryUint64",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UInt64,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 4,
                        name: "OptionalUint64",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::UInt64,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 5,
                        name: "MandatoryInt32",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Int32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 6,
                        name: "OptionalInt32",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::Int32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 7,
                        name: "MandatoryInt64",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Int64,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 8,
                        name: "OptionalInt64",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::Int64,
                        instructions: vec![],
                        has_pmap: false,
                    },
                ],
            },
            TestTemplate {
                id: 2,
                name: "String",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "MandatoryAscii",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::ASCIIString,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 2,
                        name: "OptionalAscii",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::ASCIIString,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 3,
                        name: "MandatoryUnicode",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UnicodeString,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 4,
                        name: "OptionalUnicode",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::UnicodeString,
                        instructions: vec![],
                        has_pmap: false,
                    },
                ],
            },
            TestTemplate {
                id: 3,
                name: "ByteVector",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "MandatoryVector",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Bytes,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 2,
                        name: "OptionalVector",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::Bytes,
                        instructions: vec![],
                        has_pmap: false,
                    },
                ],
            },
            TestTemplate {
                id: 4,
                name: "Decimal",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "CopyDecimal",
                        presence: Presence::Optional,
                        operator: Operator::Copy,
                        value: ValueType::Decimal,
                        instructions: vec![
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Optional,
                                operator: Operator::None,
                                value: ValueType::Exponent,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::Mantissa,
                                instructions: vec![],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: false, // for Decimal has_pmap shows if subcomponents need pmap
                    },
                    TestField {
                        id: 2,
                        name: "MandatoryDecimal",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Decimal,
                        instructions: vec![
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::Exponent,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::Mantissa,
                                instructions: vec![],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: false,
                    },
                    TestField {
                        id: 3,
                        name: "IndividualDecimal",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Decimal,
                        instructions: vec![
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Mandatory,
                                operator: Operator::Default,
                                value: ValueType::Exponent,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Mandatory,
                                operator: Operator::Delta,
                                value: ValueType::Mantissa,
                                instructions: vec![],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: true,
                    },
                    TestField {
                        id: 4,
                        name: "IndividualDecimalOpt",
                        presence: Presence::Optional,
                        operator: Operator::None,
                        value: ValueType::Decimal,
                        instructions: vec![
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Optional,
                                operator: Operator::Default,
                                value: ValueType::Exponent,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 0,
                                name: "",
                                presence: Presence::Mandatory,
                                operator: Operator::Delta,
                                value: ValueType::Mantissa,
                                instructions: vec![],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: true,
                    },
                ],
            },
            TestTemplate {
                id: 5,
                name: "Sequence",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "TestData",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UInt32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 0,
                        name: "OuterSequence",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Sequence,
                        instructions: vec![
                            TestField {
                                id: 2,
                                name: "NoOuterSequence",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::Length,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 3,
                                name: "OuterTestData",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::UInt32,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 0,
                                name: "InnerSequence",
                                presence: Presence::Optional,
                                operator: Operator::None,
                                value: ValueType::Sequence,
                                instructions: vec![
                                    TestField {
                                        id: 4,
                                        name: "NoInnerSequence",
                                        presence: Presence::Optional,
                                        operator: Operator::None,
                                        value: ValueType::Length,
                                        instructions: vec![],
                                        has_pmap: false,
                                    },
                                    TestField {
                                        id: 5,
                                        name: "InnerTestData",
                                        presence: Presence::Mandatory,
                                        operator: Operator::None,
                                        value: ValueType::UInt32,
                                        instructions: vec![],
                                        has_pmap: false,
                                    },
                                ],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: false,
                    },
                    TestField {
                        id: 0,
                        name: "NextOuterSequence",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Sequence,
                        instructions: vec![
                            TestField {
                                id: 6,
                                name: "NoNextOuterSequence",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::Length,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 7,
                                name: "NextOuterTestData",
                                presence: Presence::Mandatory,
                                operator: Operator::Copy,
                                value: ValueType::UInt32,
                                instructions: vec![],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: true,
                    },
                ],
            },
            TestTemplate {
                id: 6,
                name: "Group",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "TestData",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UInt32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 0,
                        name: "OuterGroup",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::Group,
                        instructions: vec![
                            TestField {
                                id: 2,
                                name: "OuterTestData",
                                presence: Presence::Mandatory,
                                operator: Operator::None,
                                value: ValueType::UInt32,
                                instructions: vec![],
                                has_pmap: false,
                            },
                            TestField {
                                id: 0,
                                name: "InnerGroup",
                                presence: Presence::Optional,
                                operator: Operator::None,
                                value: ValueType::Group,
                                instructions: vec![TestField {
                                    id: 3,
                                    name: "InnerTestData",
                                    presence: Presence::Mandatory,
                                    operator: Operator::None,
                                    value: ValueType::UInt32,
                                    instructions: vec![],
                                    has_pmap: false,
                                }],
                                has_pmap: false,
                            },
                        ],
                        has_pmap: true,
                    },
                ],
            },
            TestTemplate {
                id: 7,
                name: "RefData",
                dictionary: Dictionary::Global,
                instructions: vec![TestField {
                    id: 1,
                    name: "TestData",
                    presence: Presence::Mandatory,
                    operator: Operator::Copy,
                    value: ValueType::UInt32,
                    instructions: vec![],
                    has_pmap: false,
                }],
            },
            TestTemplate {
                id: 8,
                name: "StaticReference",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "PreRefData",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UInt32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 0,
                        name: "RefData",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::TemplateReference,
                        instructions: vec![],
                        has_pmap: false,
                    },
                ],
            },
            TestTemplate {
                id: 9,
                name: "DynamicReference",
                dictionary: Dictionary::Global,
                instructions: vec![
                    TestField {
                        id: 1,
                        name: "PreRefData",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::UInt32,
                        instructions: vec![],
                        has_pmap: false,
                    },
                    TestField {
                        id: 0,
                        name: "",
                        presence: Presence::Mandatory,
                        operator: Operator::None,
                        value: ValueType::TemplateReference,
                        instructions: vec![],
                        has_pmap: false,
                    },
                ],
            },
        ],
    );
}

fn do_test(raw: Vec<u8>, data: TemplateData) {
    {
        let mut msg = ModelFactory::new();
        let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
        d.decode_slice(&raw, &mut msg).unwrap();
        assert_eq!(msg.data.unwrap(), data, "decode mismatch");
    }
    {
        let mut msg = ModelVisitor::new(data.clone());
        let mut e = Encoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
        assert_eq!(e.encode_vec(&mut msg).unwrap(), raw, "encode mismatch");
    }
}

#[test]
fn decode_encode_integers() {
    do_test(
        vec![
            0xc0, 0x81, 0x83, 0x85, 0x25, 0x20, 0x2f, 0x47, 0xfe, 0x25, 0x20, 0x2f, 0x48, 0x80,
            0x85, 0x87, 0x8, 0x23, 0x51, 0x57, 0x8d, 0x8, 0x23, 0x51, 0x57, 0x8f,
        ],
        TemplateData {
            name: "Integer".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "MandatoryUint32".to_string(),
                    ValueData::Value(Some(Value::UInt32(3))),
                ),
                (
                    "OptionalUint32".to_string(),
                    ValueData::Value(Some(Value::UInt32(4))),
                ),
                (
                    "MandatoryUint64".to_string(),
                    ValueData::Value(Some(Value::UInt64(9999999998))),
                ),
                (
                    "OptionalUint64".to_string(),
                    ValueData::Value(Some(Value::UInt64(9999999999))),
                ),
                (
                    "MandatoryInt32".to_string(),
                    ValueData::Value(Some(Value::Int32(5))),
                ),
                (
                    "OptionalInt32".to_string(),
                    ValueData::Value(Some(Value::Int32(6))),
                ),
                (
                    "MandatoryInt64".to_string(),
                    ValueData::Value(Some(Value::Int64(2222222221))),
                ),
                (
                    "OptionalInt64".to_string(),
                    ValueData::Value(Some(Value::Int64(2222222222))),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_strings() {
    do_test(
        vec![
            0xc0, 0x82, 0x61, 0x62, 0xe3, 0x64, 0x65, 0xe6, 0x83, 0x67, 0x68, 0x69, 0x84, 0x6b,
            0x6c, 0x6d,
        ],
        TemplateData {
            name: "String".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "MandatoryAscii".to_string(),
                    ValueData::Value(Some(Value::ASCIIString("abc".to_string()))),
                ),
                (
                    "OptionalAscii".to_string(),
                    ValueData::Value(Some(Value::ASCIIString("def".to_string()))),
                ),
                (
                    "MandatoryUnicode".to_string(),
                    ValueData::Value(Some(Value::UnicodeString("ghi".to_string()))),
                ),
                (
                    "OptionalUnicode".to_string(),
                    ValueData::Value(Some(Value::UnicodeString("klm".to_string()))),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_bytes() {
    do_test(
        vec![0xc0, 0x83, 0x81, 0xc1, 0x82, 0xb3],
        TemplateData {
            name: "ByteVector".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "MandatoryVector".to_string(),
                    ValueData::Value(Some(Value::Bytes(vec![193]))),
                ),
                (
                    "OptionalVector".to_string(),
                    ValueData::Value(Some(Value::Bytes(vec![179]))),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_decimals_1() {
    do_test(
        vec![
            0xf8, 0x84, 0xfe, 0x4, 0x83, 0xff, 0xc, 0x8a, 0xfc, 0xa0, 0xff, 0x0, 0xef,
        ],
        TemplateData {
            name: "Decimal".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "CopyDecimal".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-2, 515)))),
                ),
                (
                    "MandatoryDecimal".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-1, 1546)))),
                ),
                (
                    "IndividualDecimal".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-4, 32)))),
                ),
                (
                    "IndividualDecimalOpt".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-1, 111)))),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_decimals_2() {
    do_test(
        vec![
            0xf8, 0x84, 0xfe, 0x4, 0x83, 0xff, 0xc, 0x8a, 0xfc, 0xa0, 0x80,
        ],
        TemplateData {
            name: "Decimal".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "CopyDecimal".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-2, 515)))),
                ),
                (
                    "MandatoryDecimal".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-1, 1546)))),
                ),
                (
                    "IndividualDecimal".to_string(),
                    ValueData::Value(Some(Value::Decimal(Decimal::new(-4, 32)))),
                ),
                ("IndividualDecimalOpt".to_string(), ValueData::Value(None)),
            ])),
        },
    );
}

#[test]
fn decode_encode_sequence_1() {
    do_test(
        vec![
            0xc0, 0x85, 0x81, 0x81, 0x82, 0x83, 0x83, 0x84, 0x81, 0xc0, 0x82,
        ],
        TemplateData {
            name: "Sequence".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "TestData".to_string(),
                    ValueData::Value(Some(Value::UInt32(1))),
                ),
                (
                    "OuterSequence".to_string(),
                    ValueData::Sequence(vec![ValueData::Group(HashMap::from([
                        (
                            "OuterTestData".to_string(),
                            ValueData::Value(Some(Value::UInt32(2))),
                        ),
                        (
                            "InnerSequence".to_string(),
                            ValueData::Sequence(vec![
                                ValueData::Group(HashMap::from([(
                                    "InnerTestData".to_string(),
                                    ValueData::Value(Some(Value::UInt32(3))),
                                )])),
                                ValueData::Group(HashMap::from([(
                                    "InnerTestData".to_string(),
                                    ValueData::Value(Some(Value::UInt32(4))),
                                )])),
                            ]),
                        ),
                    ]))]),
                ),
                (
                    "NextOuterSequence".to_string(),
                    ValueData::Sequence(vec![ValueData::Group(HashMap::from([(
                        "NextOuterTestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(2))),
                    )]))]),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_sequence_2() {
    do_test(
        vec![0xc0, 0x85, 0x81, 0x81, 0x82, 0x80, 0x81, 0xc0, 0x82],
        TemplateData {
            name: "Sequence".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "TestData".to_string(),
                    ValueData::Value(Some(Value::UInt32(1))),
                ),
                (
                    "OuterSequence".to_string(),
                    ValueData::Sequence(vec![ValueData::Group(HashMap::from([(
                        "OuterTestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(2))),
                    )]))]),
                ),
                (
                    "NextOuterSequence".to_string(),
                    ValueData::Sequence(vec![ValueData::Group(HashMap::from([(
                        "NextOuterTestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(2))),
                    )]))]),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_group_1() {
    do_test(
        vec![0xc0, 0x86, 0x81, 0xc0, 0x82, 0x83],
        TemplateData {
            name: "Group".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "TestData".to_string(),
                    ValueData::Value(Some(Value::UInt32(1))),
                ),
                (
                    "OuterGroup".to_string(),
                    ValueData::Group(HashMap::from([
                        (
                            "OuterTestData".to_string(),
                            ValueData::Value(Some(Value::UInt32(2))),
                        ),
                        (
                            "InnerGroup".to_string(),
                            ValueData::Group(HashMap::from([(
                                "InnerTestData".to_string(),
                                ValueData::Value(Some(Value::UInt32(3))),
                            )])),
                        ),
                    ])),
                ),
            ])),
        },
    );
}

#[test]
fn decode_encode_group_2() {
    do_test(
        vec![0xc0, 0x86, 0x81, 0x80, 0x82],
        TemplateData {
            name: "Group".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "TestData".to_string(),
                    ValueData::Value(Some(Value::UInt32(1))),
                ),
                (
                    "OuterGroup".to_string(),
                    ValueData::Group(HashMap::from([(
                        "OuterTestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(2))),
                    )])),
                ),
            ])),
        },
    );
}

#[test]
fn decode_static_reference() {
    do_test(
        vec![0xe0, 0x88, 0x86, 0x87],
        TemplateData {
            name: "StaticReference".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "PreRefData".to_string(),
                    ValueData::Value(Some(Value::UInt32(6))),
                ),
                (
                    "TestData".to_string(),
                    ValueData::Value(Some(Value::UInt32(7))),
                ),
            ])),
        },
    );
}

#[test]
fn decode_dynamic_reference() {
    do_test(
        vec![0xc0, 0x89, 0x86, 0xe0, 0x87, 0x85],
        TemplateData {
            name: "DynamicReference".to_string(),
            value: ValueData::Group(HashMap::from([
                (
                    "PreRefData".to_string(),
                    ValueData::Value(Some(Value::UInt32(6))),
                ),
                (
                    "templateRef:0".to_string(),
                    ValueData::DynamicTemplateRef(Box::new(TemplateData {
                        name: "RefData".to_string(),
                        value: ValueData::Group(HashMap::from([(
                            "TestData".to_string(),
                            ValueData::Value(Some(Value::UInt32(5))),
                        )])),
                    })),
                ),
            ])),
        },
    );
}

#[test]
fn decode_eof() {
    let mut r: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    let res = d.decode_stream(&mut r, &mut msg);
    match res {
        Err(Error::Eof) => {}
        _ => assert!(false, "Expected Eof"),
    }
}

#[test]
fn decode_unexpected_eof() {
    let mut r: Cursor<Vec<u8>> = Cursor::new(vec![0x00]);
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    let res = d.decode_stream(&mut r, &mut msg);
    match res {
        Err(Error::UnexpectedEof) => {}
        Err(e) => assert!(false, "Unexpected error: {:?}", e),
        Ok(_) => assert!(false, "Expected Err(UnexpectedEof)"),
    }
}

#[test]
fn encode_to_buffer() {
    // user data
    let data = TemplateData {
        name: "String".to_string(),
        value: ValueData::Group(HashMap::from([
            (
                "MandatoryAscii".to_string(),
                ValueData::Value(Some(Value::ASCIIString("abc".to_string()))),
            ),
            (
                "OptionalAscii".to_string(),
                ValueData::Value(Some(Value::ASCIIString("def".to_string()))),
            ),
            (
                "MandatoryUnicode".to_string(),
                ValueData::Value(Some(Value::UnicodeString("ghi".to_string()))),
            ),
            (
                "OptionalUnicode".to_string(),
                ValueData::Value(Some(Value::UnicodeString("klm".to_string()))),
            ),
        ])),
    };
    let mut msg = ModelVisitor::new(data);

    // encode user data to buffer
    let mut buffer = [u8::default(); 20];
    let mut e = Encoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    let n = e.encode_buffer(&mut buffer, &mut msg).unwrap();

    // check encoded data
    let encoded: Vec<u8> = vec![
        0xc0, 0x82, 0x61, 0x62, 0xe3, 0x64, 0x65, 0xe6, 0x83, 0x67, 0x68, 0x69, 0x84, 0x6b, 0x6c,
        0x6d,
    ];
    assert_eq!(&buffer[0..n], encoded);
}

#[test]
fn encode_to_buffer_eof() {
    // try to encode user data to buffer with not enough space
    let data = TemplateData {
        name: "ByteVector".to_string(),
        value: ValueData::Group(HashMap::from([
            (
                "MandatoryVector".to_string(),
                ValueData::Value(Some(Value::Bytes(vec![193]))),
            ),
            (
                "OptionalVector".to_string(),
                ValueData::Value(Some(Value::Bytes(vec![179]))),
            ),
        ])),
    };
    let mut msg = ModelVisitor::new(data);

    let mut buffer = [u8::default(); 5]; // actually 6 bytes required
    let mut e = Encoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    match e.encode_buffer(&mut buffer, &mut msg) {
        Err(Error::IoError(_)) => {}
        Err(e) => assert!(false, "Unexpected error: {:?}", e),
        Ok(_) => assert!(false, "Expected Err(UnexpectedEof)"),
    }
}
