//! # Tests based on FAST specs
//!
//! See: https://www.fixtrading.org/standards/fast-online/
//!
use crate::decoder::decoder::Decoder;
use super::*;

#[test]
fn parse_xml_template() {
    let d = Decoder::new_from_xml(include_str!("templates/spec.xml")).unwrap();
    assert_eq!(d.definitions.templates.len(), 16);
}

struct TestCase {
    name: &'static str,
    input: Vec<u8>,
    result: &'static str,
}

fn do_test(test_cases: Vec<TestCase>) {
    for tt in test_cases {
        let mut msg = LoggingMessageFactory::new();
        let mut d = Decoder::new_from_xml(include_str!("templates/spec.xml")).unwrap();
        d.decode_vec(tt.input.clone(), &mut msg).unwrap();
        assert_eq!(msg.calls.len(), 3, "{} failed", tt.name);
        assert_eq!(
            &msg.calls[1],
            &format!("set_value: 1:Value {}", tt.result),
            "{} failed", tt.name);
    }
}

struct TestCaseSeq {
    name: &'static str,
    inputs: Vec<Vec<u8>>,
    results: Vec<&'static str>,
}

fn do_tests_seq(test_cases: Vec<TestCaseSeq>) {
    for tt in test_cases {
        let mut d = Decoder::new_from_xml(include_str!("templates/spec.xml")).unwrap();
        for (i, (input, result)) in tt.inputs.iter().zip(tt.results).enumerate() {
            let mut msg = LoggingMessageFactory::new();
            d.decode_vec(input.clone(), &mut msg).unwrap();
            assert_eq!(msg.calls.len(), 3, "{} failed", tt.name);
            assert_eq!(
                &msg.calls[1],
                &format!("set_value: 1:Value {}", result),
                "{} failed #{}", tt.name, i + 1);
        }
    }
}

#[test]
fn decode_decimals() {
    // Appendix 3.1.5 - Decimal Examples
    do_test(vec![
        TestCase {
            name: "Mandatory Positive Decimal",
            input: vec![0xc0, 0x81, 0x82, 0x39, 0x45, 0xa3],
            result: "Some(Decimal(Decimal { exponent: 2, mantissa: 942755 }))",
        },
        TestCase {
            name: "Mandatory Positive Decimal with Scaled Mantissa",
            input: vec![0xc0, 0x81, 0x81, 0x04, 0x3f, 0x34, 0xde],
            result: "Some(Decimal(Decimal { exponent: 1, mantissa: 9427550 }))",
        },
        TestCase {
            name: "Mandatory Positive Decimal (2)",
            input: vec![0xc0, 0x81, 0xfe, 0x39, 0x45, 0xa3],
            result: "Some(Decimal(Decimal { exponent: -2, mantissa: 942755 }))",
        },
        TestCase {
            name: "Optional Positive Decimal",
            input: vec![0xc0, 0x82, 0x83, 0x39, 0x45, 0xa3],
            result: "Some(Decimal(Decimal { exponent: 2, mantissa: 942755 }))",
        },
        TestCase {
            name: "Optional Negative Decimal",
            input: vec![0xc0, 0x82, 0xfe, 0x46, 0x3a, 0xdd],
            result: "Some(Decimal(Decimal { exponent: -2, mantissa: -942755 }))",
        },
        TestCase {
            name: "Optional Negative Decimal with sign bit extension",
            input: vec![0xc0, 0x82, 0xfd, 0x7f, 0x3f, 0xff],
            result: "Some(Decimal(Decimal { exponent: -3, mantissa: -8193 }))",
        },
        TestCase {
            name: "Optional Positive Decimal with single field operator",
            input: vec![0xe0, 0x83, 0xfe, 0x39, 0x45, 0xa3],
            result: "Some(Decimal(Decimal { exponent: -2, mantissa: 942755 }))",
        },
        TestCase {
            name: "Optional Positive Decimal with individual field operators",
            input: vec![0xe0, 0x84, 0xfe, 0x39, 0x45, 0xa3],
            result: "Some(Decimal(Decimal { exponent: -2, mantissa: 942755 }))",
        },
    ]);
}

#[test]
fn decode_constant_operator() {
    // Appendix 3.2.1 - Constant Operator Examples
    do_test(vec![
        TestCase {
            name: "Mandatory Unsigned Integer",
            input: vec![0xc0, 0x85],
            result: "Some(UInt32(7))",
        },
        TestCase {
            name: "Optional Unsigned Integer - Absent",
            input: vec![0xc0, 0x86],
            result: "None",
        },
        TestCase {
            name: "Optional Unsigned Integer - Present",
            input: vec![0xe0, 0x86],
            result: "Some(UInt32(7))",
        },
    ]);
}

#[test]
fn decode_default_operator() {
    // Appendix 3.2.2 - Default Operator Examples
    do_test(vec![
        TestCase {
            name: "Mandatory Unsigned Integer Default",
            input: vec![0xc0, 0x87],
            result: "Some(UInt32(7))",
        },
        TestCase {
            name: "Mandatory Unsigned Integer Value",
            input: vec![0xe0, 0x87, 0x81],
            result: "Some(UInt32(1))",
        },
        TestCase {
            name: "Mandatory Unsigned Integer Optional",
            input: vec![0xc0, 0x88],
            result: "None",
        },
    ]);
}

#[test]
fn decode_copy_operator() {
    // Appendix 3.2.3 -  Copy Operator Examples
    do_tests_seq(vec![
        TestCaseSeq {
            name: "Mandatory String",
            inputs: vec![
                vec![0xe0, 0x89, 0x43, 0x4d, 0xc5],
                vec![0xc0, 0x89],
                vec![0xe0, 0x89, 0x49, 0x53, 0xc5],
            ],
            results: vec![
                "Some(ASCIIString(\"CME\"))",
                "Some(ASCIIString(\"CME\"))",
                "Some(ASCIIString(\"ISE\"))",
            ],
        },
        TestCaseSeq {
            name: "Optional String",
            inputs: vec![
                vec![0xe0, 0x8a, 0x80],
                vec![0xc0, 0x8a],
                vec![0xe0, 0x8a, 0x43, 0x4d, 0xc5],
            ],
            results: vec![
                "None",
                "None",
                "Some(ASCIIString(\"CME\"))",
            ],
        },
    ]);
}

#[test]
fn decode_increment_operator() {
    // Appendix 3.2.4 - Increment Operator Examples
    do_tests_seq(vec![
        TestCaseSeq {
            name: "Mandatory Unsigned Integer",
            inputs: vec![
                vec![0xe0, 0x8b, 0x80],
                vec![0xc0, 0x8b],
                vec![0xc0, 0x8b],
                vec![0xe0, 0x8b, 0x84],
                vec![0xc0, 0x8b],
            ],
            results: vec![
                "Some(UInt32(0))",
                "Some(UInt32(1))",
                "Some(UInt32(2))",
                "Some(UInt32(4))",
                "Some(UInt32(5))",
            ],
        },
    ]);
}

#[test]
fn decode_delta_operator() {
    // Appendix 3.2.5 - Delta Operator Examples
    do_tests_seq(vec![
        TestCaseSeq {
            name: "Mandatory Signed Integer",
            inputs: vec![
                vec![0xc0, 0x8c, 0x39, 0x45, 0xa3],
                vec![0xc0, 0x8c, 0xfb],
                vec![0xc0, 0x8c, 0xfb],
                vec![0xc0, 0x8c, 0x80],
            ],
            results: vec![
                "Some(Int32(942755))",
                "Some(Int32(942750))",
                "Some(Int32(942745))",
                "Some(Int32(942745))",
            ],
        },
        TestCaseSeq {
            name: "Mandatory Decimal",
            inputs: vec![
                vec![0xc0, 0x8d, 0xfe, 0x39, 0x45, 0xa3],
                vec![0xc0, 0x8d, 0x80, 0xfc],
                vec![0xc0, 0x8d, 0x80, 0xfb],
            ],
            results: vec![
                "Some(Decimal(Decimal { exponent: -2, mantissa: 942755 }))", // 9427.55
                "Some(Decimal(Decimal { exponent: -2, mantissa: 942751 }))", // 9427.51
                "Some(Decimal(Decimal { exponent: -2, mantissa: 942746 }))", // 9427.46
            ],
        },
        TestCaseSeq {
            // This test from official spec looks odd.
            // Instead, using CQG's version of this test.
            name: "Mandatory Decimal with Initial Value",
            inputs: vec![
                vec![0xc0, 0x8e, 0x80, 0x80],
                vec![0xc0, 0x8e, 0xff, 0x00, 0xed],
                vec![0xc0, 0x8e, 0xff, 0x08, 0xc6],
                vec![0xc0, 0x8e, 0x80, 0x81],
            ],
            results: vec![
                "Some(Decimal(Decimal { exponent: 3, mantissa: 12 }))",   // 12000.0
                "Some(Decimal(Decimal { exponent: 2, mantissa: 121 }))",  // 12100.0
                "Some(Decimal(Decimal { exponent: 1, mantissa: 1215 }))", // 12150.0
                "Some(Decimal(Decimal { exponent: 1, mantissa: 1216 }))", // 12160.0
            ],
        },
        TestCaseSeq {
            name: "Mandatory String",
            inputs: vec![
                vec![0xc0, 0x8f, 0x80, 0x47, 0x45, 0x48, 0xb6],
                vec![0xc0, 0x8f, 0x82, 0x4d, 0xb6],
                vec![0xc0, 0x8f, 0xfd, 0x45, 0xd3],
                vec![0xc0, 0x8f, 0xff, 0x52, 0xd3],
            ],
            results: vec![
                "Some(ASCIIString(\"GEH6\"))",
                "Some(ASCIIString(\"GEM6\"))",
                "Some(ASCIIString(\"ESM6\"))",
                "Some(ASCIIString(\"RSESM6\"))",
            ],
        },
    ]);
}

#[test]
fn decode_multiple_pmap() {
    // Appendix 3.2.6 - Extended Example
    do_tests_seq(vec![
        TestCaseSeq {
            // This test has typo in official spec!
            // Instead, using CQG's version of this test.
            name: "Multiple Pmap Slot",
            inputs: vec![
                vec![0xf0, 0x90, 0xfe, 0x39, 0x45, 0xa3],
                vec![0x90, 0x39, 0x45, 0xa9],
                vec![0xa0, 0x80],
            ],
            results: vec![
                "Some(Decimal(Decimal { exponent: -2, mantissa: 942755 }))", // 9427.55
                "Some(Decimal(Decimal { exponent: -2, mantissa: 942761 }))", // 9427.61
                "None",
            ],
        }
    ]);
}
