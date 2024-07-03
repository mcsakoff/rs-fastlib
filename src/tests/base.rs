//! # Tests partially adopted from GoFAST library
//!
//! See: https://github.com/mcsakoff/goFAST/tree/main
//!
use crate::decoder::decoder::Decoder;
use super::*;

#[test]
fn parse_xml_template() {
    let d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
        test_templates(&d, &vec![
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
            ]
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
            ]
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
            ]
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
                }
            ]
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
                            instructions: vec![
                                TestField {
                                    id: 3,
                                    name: "InnerTestData",
                                    presence: Presence::Mandatory,
                                    operator: Operator::None,
                                    value: ValueType::UInt32,
                                    instructions: vec![],
                                    has_pmap: false,
                                },
                            ],
                            has_pmap: false,
                        }
                    ],
                    has_pmap: true,
                },
            ]
        },
        TestTemplate {
            id: 7,
            name: "RefData",
            dictionary: Dictionary::Global,
            instructions: vec![
                TestField {
                    id: 1,
                    name: "TestData",
                    presence: Presence::Mandatory,
                    operator: Operator::Copy,
                    value: ValueType::UInt32,
                    instructions: vec![],
                    has_pmap: false,
                },
            ]
        },
        TestTemplate {
            id: 8,
            name: "StaticReference",
            dictionary: Dictionary::Global,
            instructions: vec![
                TestField {
                    id: 0,
                    name: "RefData",
                    presence: Presence::Mandatory,
                    operator: Operator::None,
                    value: ValueType::TemplateReference,
                    instructions: vec![],
                    has_pmap: false,
                },
            ]
        },
        TestTemplate {
            id: 9,
            name: "DynamicReference",
            dictionary: Dictionary::Global,
            instructions: vec![
                TestField {
                    id: 0,
                    name: "",
                    presence: Presence::Mandatory,
                    operator: Operator::None,
                    value: ValueType::TemplateReference,
                    instructions: vec![],
                    has_pmap: false,
                },
            ]
        },
    ]);
}

#[test]
fn decode_integers() {
    let r = vec![0xc0, 0x81, 0x83, 0x85, 0x25, 0x20, 0x2f, 0x47, 0xfe, 0x25, 0x20, 0x2f, 0x48, 0x80, 0x85, 0x87, 0x8, 0x23, 0x51, 0x57, 0x8d, 0x8, 0x23, 0x51, 0x57, 0x8f];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 1:Integer",
        "set_value: 1:MandatoryUint32 Some(UInt32(3))",
        "set_value: 2:OptionalUint32 Some(UInt32(4))",
        "set_value: 3:MandatoryUint64 Some(UInt64(9999999998))",
        "set_value: 4:OptionalUint64 Some(UInt64(9999999999))",
        "set_value: 5:MandatoryInt32 Some(Int32(5))",
        "set_value: 6:OptionalInt32 Some(Int32(6))",
        "set_value: 7:MandatoryInt64 Some(Int64(2222222221))",
        "set_value: 8:OptionalInt64 Some(Int64(2222222222))",
        "stop_template",
    ]);
}

#[test]
fn decode_strings() {
    let r = vec![0xc0, 0x82, 0x61, 0x62, 0xe3, 0x64, 0x65, 0xe6, 0x83, 0x67, 0x68, 0x69, 0x84, 0x6b, 0x6c, 0x6d];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 2:String",
        "set_value: 1:MandatoryAscii Some(ASCIIString(\"abc\"))",
        "set_value: 2:OptionalAscii Some(ASCIIString(\"def\"))",
        "set_value: 3:MandatoryUnicode Some(UnicodeString(\"ghi\"))",
        "set_value: 4:OptionalUnicode Some(UnicodeString(\"klm\"))",
        "stop_template",
    ]);
}

#[test]
fn decode_bytes() {
    let r = vec![0xc0, 0x83, 0x81, 0xc1, 0x82, 0xb3];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 3:ByteVector",
        "set_value: 1:MandatoryVector Some(Bytes([193]))",
        "set_value: 2:OptionalVector Some(Bytes([179]))",
        "stop_template",
    ]);
}

#[test]
fn decode_decimals_1() {
    let r = vec![0xf8, 0x84, 0xfe, 0x4, 0x83, 0xff, 0xc, 0x8a, 0xfc, 0xa0, 0xff, 0x0, 0xef];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 4:Decimal",
        "set_value: 1:CopyDecimal Some(Decimal(5.15))",
        "set_value: 2:MandatoryDecimal Some(Decimal(154.6))",
        "set_value: 3:IndividualDecimal Some(Decimal(0.0032))",
        "set_value: 4:IndividualDecimalOpt Some(Decimal(11.1))",
        "stop_template",
    ]);
}

#[test]
fn decode_decimals_2() {
    let r = vec![0xf8, 0x84, 0xfe, 0x4, 0x83, 0xff, 0xc, 0x8a, 0xfc, 0xa0, 0x80];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 4:Decimal",
        "set_value: 1:CopyDecimal Some(Decimal(5.15))",
        "set_value: 2:MandatoryDecimal Some(Decimal(154.6))",
        "set_value: 3:IndividualDecimal Some(Decimal(0.0032))",
        "set_value: 4:IndividualDecimalOpt None",
        "stop_template",
    ]);
}

#[test]
fn decode_sequence() {
    let r = vec![0xc0, 0x85, 0x81, 0x81, 0x82, 0x83, 0x83, 0x84, 0x81, 0xc0, 0x82];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 5:Sequence",
        "set_value: 1:TestData Some(UInt32(1))",
        "start_sequence: 0:OuterSequence 1",
        "start_sequence_item: 0",
        "set_value: 3:OuterTestData Some(UInt32(2))",
        "start_sequence: 0:InnerSequence 2",
        "start_sequence_item: 0",
        "set_value: 5:InnerTestData Some(UInt32(3))",
        "stop_sequence_item",
        "start_sequence_item: 1",
        "set_value: 5:InnerTestData Some(UInt32(4))",
        "stop_sequence_item",
        "stop_sequence",
        "stop_sequence_item",
        "stop_sequence",
        "start_sequence: 0:NextOuterSequence 1",
        "start_sequence_item: 0",
        "set_value: 7:NextOuterTestData Some(UInt32(2))",
        "stop_sequence_item",
        "stop_sequence",
        "stop_template",
    ]);
}

#[test]
fn decode_group() {
    let r = vec![0xc0, 0x86, 0x81, 0xc0, 0x82, 0x83];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 6:Group",
        "set_value: 1:TestData Some(UInt32(1))",
        "start_group: OuterGroup",
        "set_value: 2:OuterTestData Some(UInt32(2))",
        "start_group: InnerGroup",
        "set_value: 3:InnerTestData Some(UInt32(3))",
        "stop_group",
        "stop_group",
        "stop_template",
    ]);
}

#[test]
fn decode_static_reference() {
    let r = vec![0xe0, 0x88, 0x87];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 8:StaticReference",
        "start_template_ref: RefData:false",
        "set_value: 1:TestData Some(UInt32(7))",
        "stop_template_ref",
        "stop_template",
    ]);
}

#[test]
fn decode_dynamic_reference() {
    let r = vec![0xc0, 0x89, 0xe0, 0x87, 0x85];
    let mut msg = LoggingMessageFactory::new();
    let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
    d.decode_vec(r, &mut msg).unwrap();
    assert_eq!(&msg.calls, &vec![
        "start_template: 9:DynamicReference",
        "start_template_ref: RefData:true",
        "set_value: 1:TestData Some(UInt32(5))",
        "stop_template_ref",
        "stop_template",
    ]);
}
