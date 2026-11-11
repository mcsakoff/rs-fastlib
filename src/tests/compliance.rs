//! # Regression tests for FAST 1.1 spec compliance fixes
//!
//! See: https://fixtrading.org/packages/fast-specification-version-1-1/
//!
use super::*;
use crate::{Decimal, Error};

const DEFINITION: &str = include_str!("templates/compliance.xml");

fn msg(template: &str, value: Option<Value>) -> TestMsgValue<'_> {
    TestMsgValue { template, value }
}

fn ascii(s: &str) -> Option<Value> {
    Some(Value::ASCIIString(s.to_string()))
}

// Decodes messages one by one and returns the result of the last one.
fn decode_last(raw: &[&[u8]]) -> crate::Result<()> {
    let mut d = Decoder::new_from_xml(DEFINITION).unwrap();
    let (last, init) = raw.split_last().unwrap();
    for r in init {
        d.decode_slice(r, &mut ModelFactory::new()).unwrap();
    }
    d.decode_slice(last, &mut ModelFactory::new())
}

// Encodes messages one by one and returns the result of the last one.
fn encode_last(template: &str, values: &[Value]) -> crate::Result<Vec<u8>> {
    let mut e = Encoder::new_from_xml(DEFINITION).unwrap();
    let (last, init) = values.split_last().unwrap();
    for v in init {
        let mut m = ModelVisitor::new(pack_value(template, "Value", Some(v.clone())));
        e.encode_vec(&mut m).unwrap();
    }
    let mut m = ModelVisitor::new(pack_value(template, "Value", Some(last.clone())));
    e.encode_vec(&mut m)
}

#[test]
fn increment_undefined_previous_value() {
    // Undefined previous value: the value of the field is the initial value.
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "increment initial value",
            raw: vec![vec![0xc0, 0x81], vec![0x80], vec![0xa0, 0x8a], vec![0x80]],
            data: vec![
                msg("IncrementInitial", Some(Value::UInt32(5))),
                msg("IncrementInitial", Some(Value::UInt32(6))),
                msg("IncrementInitial", Some(Value::UInt32(10))),
                msg("IncrementInitial", Some(Value::UInt32(11))),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "increment initial value explicit",
            raw: vec![vec![0xe0, 0x81, 0x86], vec![0x80]],
            data: vec![
                msg("IncrementInitial", Some(Value::UInt32(6))),
                msg("IncrementInitial", Some(Value::UInt32(7))),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "optional increment initial value",
            raw: vec![
                vec![0xc0, 0x94],
                vec![0x80],
                vec![0xa0, 0x80],
                vec![0x80],
                vec![0xa0, 0x88],
                vec![0x80],
            ],
            data: vec![
                msg("OptionalIncrementInitial", Some(Value::UInt32(5))),
                msg("OptionalIncrementInitial", Some(Value::UInt32(6))),
                msg("OptionalIncrementInitial", None),
                msg("OptionalIncrementInitial", None),
                msg("OptionalIncrementInitial", Some(Value::UInt32(7))),
                msg("OptionalIncrementInitial", Some(Value::UInt32(8))),
            ],
        },
    );
}

#[test]
fn increment_wraps_around() {
    let cases = [
        (
            "IncrementU32",
            vec![0xe0, 0x82, 0x0f, 0x7f, 0x7f, 0x7f, 0xff],
            Value::UInt32(u32::MAX),
            Value::UInt32(0),
        ),
        (
            "IncrementI32",
            vec![0xe0, 0x83, 0x07, 0x7f, 0x7f, 0x7f, 0xff],
            Value::Int32(i32::MAX),
            Value::Int32(i32::MIN),
        ),
        (
            "IncrementU64",
            vec![
                0xe0, 0x84, 0x01, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
            ],
            Value::UInt64(u64::MAX),
            Value::UInt64(0),
        ),
        (
            "IncrementI64",
            vec![
                0xe0, 0x85, 0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
            ],
            Value::Int64(i64::MAX),
            Value::Int64(i64::MIN),
        ),
    ];
    for (template, raw, max, min) in cases {
        do_test_seq(
            true,
            true,
            true,
            DEFINITION,
            TestCaseSeq {
                name: template,
                raw: vec![raw, vec![0x80]],
                data: vec![msg(template, Some(max)), msg(template, Some(min))],
            },
        );
    }
}

#[test]
fn tail_null_sets_previous_value_empty() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "tail NULL",
            raw: vec![
                vec![0xe0, 0x86, 0x41, 0xc2],
                vec![0xa0, 0x80],
                vec![0x80],
                vec![0xa0, 0x43, 0xc4],
                vec![0x80],
            ],
            data: vec![
                msg("OptionalTail", ascii("AB")),
                msg("OptionalTail", None),
                msg("OptionalTail", None),
                msg("OptionalTail", ascii("CD")),
                msg("OptionalTail", ascii("CD")),
            ],
        },
    );
}

#[test]
fn decimal_operator_initial_value() {
    let d15 = Some(Value::Decimal(Decimal::new(-1, 15)));
    let d25 = Some(Value::Decimal(Decimal::new(-1, 25)));
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "decimal constant",
            raw: vec![vec![0xc0, 0x87], vec![0x80]],
            data: vec![
                msg("DecimalConstant", d15.clone()),
                msg("DecimalConstant", d15.clone()),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "decimal default",
            raw: vec![vec![0xc0, 0x88], vec![0xa0, 0xff, 0x99], vec![0x80]],
            data: vec![
                msg("DecimalDefault", d15.clone()),
                msg("DecimalDefault", d25.clone()),
                msg("DecimalDefault", d15.clone()),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "decimal copy",
            raw: vec![
                vec![0xc0, 0x89],
                vec![0x80],
                vec![0xa0, 0xff, 0x99],
                vec![0x80],
            ],
            data: vec![
                msg("DecimalCopy", d15.clone()),
                msg("DecimalCopy", d15),
                msg("DecimalCopy", d25.clone()),
                msg("DecimalCopy", d25),
            ],
        },
    );
}

#[test]
fn integer_delta_range() {
    // The delta may need more bits than the field type.
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "int32 delta",
            raw: vec![
                vec![0xc0, 0x8c, 0x07, 0x7f, 0x7f, 0x7f, 0xff],
                vec![0x80, 0x70, 0x00, 0x00, 0x00, 0x81],
            ],
            data: vec![
                msg("DeltaI32", Some(Value::Int32(i32::MAX))),
                msg("DeltaI32", Some(Value::Int32(i32::MIN))),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "uInt64 delta",
            raw: vec![
                vec![
                    0xc0, 0x8a, 0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
                ],
                vec![
                    0x80, 0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
                ],
            ],
            data: vec![
                msg("DeltaU64", Some(Value::UInt64(u64::MAX >> 1))),
                msg("DeltaU64", Some(Value::UInt64(u64::MAX - 1))),
            ],
        },
    );

    // [ERR R4] the combined value is out of the type range.
    for raw in [
        &[&[0xc0, 0x8b, 0xff][..]][..],
        &[&[0xc0, 0x8a, 0xff]],
        &[&[0xc0, 0x8c, 0x07, 0x7f, 0x7f, 0x7f, 0xff], &[0x80, 0x81]],
        &[&[0xc0, 0x8c, 0x78, 0x00, 0x00, 0x00, 0x80], &[0x80, 0xff]],
        &[
            &[
                0xc0, 0x8d, 0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
            ],
            &[0x80, 0x81],
        ],
    ] {
        assert!(
            matches!(decode_last(raw), Err(Error::Dynamic(_))),
            "{raw:x?}"
        );
    }

    // The delta does not fit into int64.
    assert!(encode_last("DeltaU64", &[Value::UInt64(u64::MAX)]).is_err());
    assert!(
        encode_last(
            "DeltaU64",
            &[Value::UInt64(u64::MAX >> 1), Value::UInt64(u64::MAX)]
        )
            .is_err()
    );
    assert!(
        encode_last(
            "DeltaI64",
            &[Value::Int64(i64::MAX), Value::Int64(i64::MIN)]
        )
            .is_err()
    );
}

#[test]
fn optional_delta_vector_part_is_not_nullable() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "optional byteVector delta",
            raw: vec![
                vec![0xc0, 0x8e, 0x81, 0x81, 0x01],
                vec![0x80, 0x80],
                vec![0x80, 0x81, 0x81, 0x02],
            ],
            data: vec![
                msg("OptionalDeltaBytes", Some(Value::Bytes(vec![0x01]))),
                msg("OptionalDeltaBytes", None),
                msg("OptionalDeltaBytes", Some(Value::Bytes(vec![0x01, 0x02]))),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "optional string delta",
            raw: vec![
                vec![0xc0, 0x8f, 0x81, 0x80],
                vec![0x80, 0x80],
                vec![0x80, 0x81, 0x41, 0xc2],
            ],
            data: vec![
                msg("OptionalDeltaAscii", ascii("")),
                msg("OptionalDeltaAscii", None),
                msg("OptionalDeltaAscii", ascii("AB")),
            ],
        },
    );
}

#[test]
fn ascii_zero_preamble() {
    let cases: Vec<(&str, Vec<u8>, Option<Value>)> = vec![
        ("Ascii", vec![0xc0, 0x90, 0x80], ascii("")),
        ("Ascii", vec![0xc0, 0x90, 0x00, 0x80], ascii("\0")),
        ("Ascii", vec![0xc0, 0x90, 0x00, 0x00, 0x80], ascii("\0\0")),
        ("Ascii", vec![0xc0, 0x90, 0x00, 0x00, 0xc1], ascii("\0A")),
        ("Ascii", vec![0xc0, 0x90, 0xc1], ascii("A")),
        ("Ascii", vec![0xc0, 0x90, 0x41, 0x00, 0x80], ascii("A\0\0")),
        ("OptionalAscii", vec![0xc0, 0x91, 0x80], None),
        ("OptionalAscii", vec![0xc0, 0x91, 0x00, 0x80], ascii("")),
        (
            "OptionalAscii",
            vec![0xc0, 0x91, 0x00, 0x00, 0x80],
            ascii("\0"),
        ),
        (
            "OptionalAscii",
            vec![0xc0, 0x91, 0x00, 0x00, 0x00, 0xc1],
            ascii("\0A"),
        ),
        ("OptionalAscii", vec![0xc0, 0x91, 0xc1], ascii("A")),
    ];
    for (template, raw, value) in cases {
        do_test(
            true,
            true,
            true,
            DEFINITION,
            TestCase {
                name: &format!("{template} {value:?}"),
                raw,
                data: msg(template, value),
            },
        );
    }

    // [ERR R9] overlong strings.
    for raw in [
        &[0xc0, 0x90, 0x00, 0xc1][..],
        &[0xc0, 0x91, 0x00, 0xc1],
        &[0xc0, 0x91, 0x00, 0x00, 0xc1],
    ] {
        assert!(
            matches!(decode_last(&[raw]), Err(Error::Dynamic(_))),
            "{raw:x?}"
        );
    }
}

#[test]
fn nullable_integer_max_values() {
    let cases = [
        (
            "OptionalU64",
            vec![0xc0, 0x92, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0x80],
            Value::UInt64(u64::MAX),
        ),
        (
            "OptionalU64",
            vec![
                0xc0, 0x92, 0x01, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
            ],
            Value::UInt64(u64::MAX - 1),
        ),
        (
            "OptionalI64",
            vec![0xc0, 0x93, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x80],
            Value::Int64(i64::MAX),
        ),
        (
            "OptionalI64",
            vec![
                0xc0, 0x93, 0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
            ],
            Value::Int64(i64::MAX - 1),
        ),
        (
            "OptionalI64",
            vec![0xc0, 0x93, 0x7f, 0, 0, 0, 0, 0, 0, 0, 0, 0x80],
            Value::Int64(i64::MIN),
        ),
    ];
    for (template, raw, value) in cases {
        do_test(
            true,
            true,
            true,
            DEFINITION,
            TestCase {
                name: &format!("{template} {value:?}"),
                raw,
                data: msg(template, Some(value)),
            },
        );
    }

    // Values above 2^64 / 2^63 still don't fit.
    for raw in [
        &[0xc0, 0x92, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0x81][..],
        &[0xc0, 0x93, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x81],
    ] {
        assert!(
            matches!(decode_last(&[raw]), Err(Error::Dynamic(_))),
            "{raw:x?}"
        );
    }
}
