//! # Tests based on FAST spec descriptions
//!
//! See: https://www.fixtrading.org/standards/fast-online/
//!
use super::*;

const DEFINITION: &str = include_str!("templates/spec2.xml");

#[test]
fn operator_none_context() {
    // Test a filed with no operator doesn't write its value to context.
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "NONE operator context",
            raw: vec![
                vec![0xc0, 0x81, 0x84],
                vec![0xe0, 0x83, 0x84],
                vec![0x80],
                vec![0xc0, 0x81, 0x85],
                vec![0xc0, 0x83],
            ],
            data: vec![
                TestMsgValue {
                    template: "Mandatory",
                    value: Some(Value::UInt32(4)),
                },
                TestMsgValue {
                    template: "MandatoryCopy",
                    value: Some(Value::UInt32(4)),
                },
                TestMsgValue {
                    template: "MandatoryCopy",
                    value: Some(Value::UInt32(4)),
                },
                TestMsgValue {
                    template: "Mandatory",
                    value: Some(Value::UInt32(5)),
                },
                TestMsgValue {
                    template: "MandatoryCopy",
                    value: Some(Value::UInt32(4)),
                },
            ],
        },
    );
}

#[test]
fn operator_default() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "DEFAULT operator",
            raw: vec![
                vec![0xc0, 0x87],
                vec![0xa0, 0x85],
                vec![0xc0, 0x88],
                vec![0xa0, 0x85],
                vec![0xe0, 0x89, 0x80],
                vec![0x80],
                vec![0xa0, 0x86],
            ],
            data: vec![
                TestMsgValue {
                    template: "MandatoryDefault",
                    value: Some(Value::UInt32(4)),
                },
                TestMsgValue {
                    template: "MandatoryDefault",
                    value: Some(Value::UInt32(5)),
                },
                TestMsgValue {
                    template: "OptionalDefaultNone",
                    value: None,
                },
                TestMsgValue {
                    template: "OptionalDefaultNone",
                    value: Some(Value::UInt32(4)),
                },
                TestMsgValue {
                    template: "OptionalDefaultValue",
                    value: None,
                },
                TestMsgValue {
                    template: "OptionalDefaultValue",
                    value: Some(Value::UInt32(4)),
                },
                TestMsgValue {
                    template: "OptionalDefaultValue",
                    value: Some(Value::UInt32(5)),
                },
            ],
        },
    );
}

#[test]
fn operator_tail() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "TAIL operator",
            raw: vec![
                vec![0xe0, 0x8a, 0x41, 0x42, 0xc3],
                vec![0xa0, 0xda],
                vec![0xa0, 0x41, 0x42, 0x5a, 0xd9],
                vec![0x80],
            ],
            data: vec![
                TestMsgValue {
                    template: "MandatoryTail",
                    value: Some(Value::ASCIIString("ABC".to_string())),
                },
                TestMsgValue {
                    template: "MandatoryTail",
                    value: Some(Value::ASCIIString("ABZ".to_string())),
                },
                TestMsgValue {
                    template: "MandatoryTail",
                    value: Some(Value::ASCIIString("ABZY".to_string())),
                },
                TestMsgValue {
                    template: "MandatoryTail",
                    value: Some(Value::ASCIIString("ABZY".to_string())),
                },
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "TAIL operator",
            raw: vec![
                vec![0xc0, 0x8b],
                vec![0xa0, 0x41, 0x42, 0xc3],
                vec![0xa0, 0x59, 0xd9],
                vec![0x80],
                vec![0xa0, 0x80],
            ],
            data: vec![
                TestMsgValue {
                    template: "OptionalTail",
                    value: None,
                },
                TestMsgValue {
                    template: "OptionalTail",
                    value: Some(Value::ASCIIString("ABC".to_string())),
                },
                TestMsgValue {
                    template: "OptionalTail",
                    value: Some(Value::ASCIIString("AYY".to_string())),
                },
                TestMsgValue {
                    template: "OptionalTail",
                    value: Some(Value::ASCIIString("AYY".to_string())),
                },
                TestMsgValue {
                    template: "OptionalTail",
                    value: None,
                },
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "TAIL operator",
            raw: vec![vec![0xe0, 0x8c, 0x80], vec![0x80], vec![0xa0, 0x59, 0xd9]],
            data: vec![
                TestMsgValue {
                    template: "OptionalTailDefault",
                    value: None,
                },
                TestMsgValue {
                    template: "OptionalTailDefault",
                    value: Some(Value::ASCIIString("ABC".to_string())),
                },
                TestMsgValue {
                    template: "OptionalTailDefault",
                    value: Some(Value::ASCIIString("AYY".to_string())),
                },
            ],
        },
    );
}
