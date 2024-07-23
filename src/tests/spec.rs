//! # Tests based on FAST specs examples
//!
//! See: https://www.fixtrading.org/standards/fast-online/
//!
use crate::Decimal;
use crate::decoder::decoder::Decoder;

use super::*;

const DEFINITION: &str = include_str!("templates/spec.xml");

#[test]
fn parse_xml_template() {
    let d = Decoder::new_from_xml(DEFINITION).unwrap();
    assert_eq!(d.definitions.templates.len(), 16);
}

#[test]
fn decode_encode_decimals() {
    // Appendix 3.1.5 - Decimal Examples
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Positive Decimal",
        raw: vec![0xc0, 0x81, 0x82, 0x39, 0x45, 0xa3],
        data: TestMsgValue {
            template: "MandatoryDecimal",
            value: Some(Value::Decimal(Decimal::new(2, 942755))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Positive Decimal with Scaled Mantissa",
        raw: vec![0xc0, 0x81, 0x81, 0x04, 0x3f, 0x34, 0xde],
        data: TestMsgValue {
            template: "MandatoryDecimal",
            value: Some(Value::Decimal(Decimal::new(1, 9427550))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Positive Decimal (2)",
        raw: vec![0xc0, 0x81, 0xfe, 0x39, 0x45, 0xa3],
        data: TestMsgValue {
            template: "MandatoryDecimal",
            value: Some(Value::Decimal(Decimal::new(-2, 942755))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Positive Decimal",
        raw: vec![0xc0, 0x82, 0x83, 0x39, 0x45, 0xa3],
        data: TestMsgValue {
            template: "OptionalDecimal",
            value: Some(Value::Decimal(Decimal::new(2, 942755))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Negative Decimal",
        raw: vec![0xc0, 0x82, 0xfe, 0x46, 0x3a, 0xdd],
        data: TestMsgValue {
            template: "OptionalDecimal",
            value: Some(Value::Decimal(Decimal::new(-2, -942755))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Negative Decimal with sign bit extension",
        raw: vec![0xc0, 0x82, 0xfd, 0x7f, 0x3f, 0xff],
        data: TestMsgValue {
            template: "OptionalDecimal",
            value: Some(Value::Decimal(Decimal::new(-3, -8193))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Positive Decimal with single field operator",
        raw: vec![0xe0, 0x83, 0xfe, 0x39, 0x45, 0xa3],
        data: TestMsgValue {
            template: "OptionalDecimalWithOperator",
            value: Some(Value::Decimal(Decimal::new(-2, 942755))),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Positive Decimal with individual field operators",
        raw: vec![0xe0, 0x84, 0xfe, 0x39, 0x45, 0xa3],
        data: TestMsgValue {
            template: "OptionalDecimalWithIndividual",
            value: Some(Value::Decimal(Decimal::new(-2, 942755))),
        },
    });
}

#[test]
fn decode_encode_constant_operator() {
    // Appendix 3.2.1 - Constant Operator Examples
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Unsigned Integer",
        raw: vec![0xc0, 0x85],
        data: TestMsgValue {
            template: "ConstantOperatorMandatory",
            value: Some(Value::UInt32(7)),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Unsigned Integer - Absent",
        raw: vec![0xc0, 0x86],
        data: TestMsgValue {
            template: "ConstantOperatorOptional",
            value: None,
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Optional Unsigned Integer - Present",
        raw: vec![0xe0, 0x86],
        data: TestMsgValue {
            template: "ConstantOperatorOptional",
            value: Some(Value::UInt32(7)),
        },
    });
}

#[test]
fn decode_encode_default_operator() {
    // Appendix 3.2.2 - Default Operator Examples
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Unsigned Integer Default",
        raw: vec![0xc0, 0x87],
        data: TestMsgValue {
            template: "DefaultOperatorMandatory",
            value: Some(Value::UInt32(7)),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Unsigned Integer Value",
        raw: vec![0xe0, 0x87, 0x81],
        data: TestMsgValue {
            template: "DefaultOperatorMandatory",
            value: Some(Value::UInt32(1)),
        },
    });
    do_test(true, true, true, DEFINITION, TestCase {
        name: "Mandatory Unsigned Integer Optional",
        raw: vec![0xc0, 0x88],
        data: TestMsgValue {
            template: "DefaultOperatorOptional",
            value: None,
        },
    });
}

#[test]
fn decode_encode_copy_operator() {
    // Appendix 3.2.3 -  Copy Operator Examples
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        name: "Mandatory String",
        raw: vec![
            vec![0xe0, 0x89, 0x43, 0x4d, 0xc5],
            vec![0x80],
            vec![0xa0, 0x49, 0x53, 0xc5],
        ],
        data: vec![
            TestMsgValue {
                template: "CopyOperatorMandatory",
                value: Some(Value::ASCIIString("CME".to_string())),
            },
            TestMsgValue {
                template: "CopyOperatorMandatory",
                value: Some(Value::ASCIIString("CME".to_string())),
            },
            TestMsgValue {
                template: "CopyOperatorMandatory",
                value: Some(Value::ASCIIString("ISE".to_string())),
            },
        ],
    });
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        name: "Optional String",
        raw: vec![
            vec![0xc0, 0x8a],
            vec![0xa0, 0x43, 0x4d, 0xc5],
            vec![0xa0, 0x80],
            vec![0xa0, 0x43, 0x4d, 0xc5],
        ],
        data: vec![
            TestMsgValue {
                template: "CopyOperatorOptional",
                value: None,
            },
            TestMsgValue {
                template: "CopyOperatorOptional",
                value: Some(Value::ASCIIString("CME".to_string())),
            },
            TestMsgValue {
                template: "CopyOperatorOptional",
                value: None,
            },
            TestMsgValue {
                template: "CopyOperatorOptional",
                value: Some(Value::ASCIIString("CME".to_string())),
            },
        ],
    });
}

#[test]
fn decode_encode_increment_operator() {
    // Appendix 3.2.4 - Increment Operator Examples
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        name: "Mandatory Unsigned Integer",
        raw: vec![
            vec![0xe0, 0x8b, 0x80],
            vec![0x80],
            vec![0x80],
            vec![0xa0, 0x84],
            vec![0x80],
        ],
        data: vec![
            TestMsgValue {
                template: "IncrementOperatorMandatory",
                value: Some(Value::UInt32(0)),
            },
            TestMsgValue {
                template: "IncrementOperatorMandatory",
                value: Some(Value::UInt32(1)),
            },
            TestMsgValue {
                template: "IncrementOperatorMandatory",
                value: Some(Value::UInt32(2)),
            },
            TestMsgValue {
                template: "IncrementOperatorMandatory",
                value: Some(Value::UInt32(4)),
            },
            TestMsgValue {
                template: "IncrementOperatorMandatory",
                value: Some(Value::UInt32(5)),
            },
        ],
    },
    );
}

#[test]
fn decode_encode_delta_operator() {
    // Appendix 3.2.5 - Delta Operator Examples
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        name: "Mandatory Signed Integer",
        raw: vec![
            vec![0xc0, 0x8c, 0x39, 0x45, 0xa3],
            vec![0x80, 0xfb],
            vec![0x80, 0xfb],
            vec![0x80, 0x80],
        ],
        data: vec![
            TestMsgValue {
                template: "DeltaOperatorMandatorySignedInteger",
                value: Some(Value::Int32(942755)),
            },
            TestMsgValue {
                template: "DeltaOperatorMandatorySignedInteger",
                value: Some(Value::Int32(942750)),
            },
            TestMsgValue {
                template: "DeltaOperatorMandatorySignedInteger",
                value: Some(Value::Int32(942745)),
            },
            TestMsgValue {
                template: "DeltaOperatorMandatorySignedInteger",
                value: Some(Value::Int32(942745)),
            },
        ],
    });
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        name: "Mandatory Decimal",
        raw: vec![
            vec![0xc0, 0x8d, 0xfe, 0x39, 0x45, 0xa3],
            vec![0x80, 0x80, 0xfc],
            vec![0x80, 0x80, 0xfb],
        ],
        data: vec![
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimal",
                value: Some(Value::Decimal(Decimal { exponent: -2, mantissa: 942755 })), // 9427.55
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimal",
                value: Some(Value::Decimal(Decimal { exponent: -2, mantissa: 942751 })), // 9427.51
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimal",
                value: Some(Value::Decimal(Decimal { exponent: -2, mantissa: 942746 })), // 9427.46
            },
        ],
    });
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        // This test from official spec looks odd.
        // Instead, using CQG's version of this test.
        name: "Mandatory Decimal with Initial Value",
        raw: vec![
            vec![0xc0, 0x8e, 0x80, 0x80],
            vec![0x80, 0xff, 0x00, 0xed],
            vec![0x80, 0xff, 0x08, 0xc6],
            vec![0x80, 0x80, 0x81],
        ],
        data: vec![
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimalWithInit",
                value: Some(Value::Decimal(Decimal { exponent: 3, mantissa: 12 })),   // 12000.0
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimalWithInit",
                value: Some(Value::Decimal(Decimal { exponent: 2, mantissa: 121 })),  // 12100.0
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimalWithInit",
                value: Some(Value::Decimal(Decimal { exponent: 1, mantissa: 1215 })), // 12150.0
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryDecimalWithInit",
                value: Some(Value::Decimal(Decimal { exponent: 1, mantissa: 1216 })), // 12160.0
            },
        ],
    });
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        name: "Mandatory String",
        raw: vec![
            vec![0xc0, 0x8f, 0x80, 0x47, 0x45, 0x48, 0xb6],
            vec![0x80, 0x82, 0x4d, 0xb6],
            vec![0x80, 0xfd, 0x45, 0xd3],
            vec![0x80, 0xff, 0x52, 0xd3],
        ],
        data: vec![
            TestMsgValue {
                template: "DeltaOperatorMandatoryString",
                value: Some(Value::ASCIIString("GEH6".to_string())),
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryString",
                value: Some(Value::ASCIIString("GEM6".to_string())),
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryString",
                value: Some(Value::ASCIIString("ESM6".to_string())),
            },
            TestMsgValue {
                template: "DeltaOperatorMandatoryString",
                value: Some(Value::ASCIIString("RSESM6".to_string())),
            },
        ],
    });
}

#[test]
fn decode_encode_multiple_pmap() {
    // Appendix 3.2.6 - Extended Example
    do_test_seq(true, true, true, DEFINITION, TestCaseSeq {
        // This test has typo in official spec!
        // Instead, using CQG's version of this test.
        name: "Multiple PMap Slot",
        raw: vec![
            vec![0xf0, 0x90, 0xfe, 0x39, 0x45, 0xa3],
            vec![0x90, 0x39, 0x45, 0xa9],
            vec![0xa0, 0x80],
        ],
        data: vec![
            TestMsgValue {
                template: "MultiplePMapSlot",
                value: Some(Value::Decimal(Decimal::new(-2, 942755))), // 9427.55
            },
            TestMsgValue {
                template: "MultiplePMapSlot",
                value: Some(Value::Decimal(Decimal::new(-2, 942761))), // 9427.61
            },
            TestMsgValue {
                template: "MultiplePMapSlot",
                value: None,
            },
        ],
    });
}
