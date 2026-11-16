//! # Regression tests for FAST 1.1 spec compliance fixes
//!
//! See: https://fixtrading.org/packages/fast-specification-version-1-1/
//!
use super::*;
use crate::common::definitions::Definitions;
use crate::model::template::TemplateData;
use crate::model::value::{Fields, ValueData};
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

    // A delta of 64-bit integers can need 65 bits (2^64 - 1 and -(2^64 - 1) here).
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "uInt64 65-bit delta",
            raw: vec![
                vec![
                    0xc0, 0x8a, 0x01, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
                ],
                vec![
                    0x80, 0x7e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81,
                ],
            ],
            data: vec![
                msg("DeltaU64", Some(Value::UInt64(u64::MAX))),
                msg("DeltaU64", Some(Value::UInt64(0))),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "int64 65-bit delta",
            raw: vec![
                vec![
                    0xc0, 0x8d, 0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
                ],
                vec![
                    0x80, 0x7e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81,
                ],
            ],
            data: vec![
                msg("DeltaI64", Some(Value::Int64(i64::MAX))),
                msg("DeltaI64", Some(Value::Int64(i64::MIN))),
            ],
        },
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

const DEFINITION_REFS: &str = include_str!("templates/compliance_refs.xml");

// Encodes and decodes messages in order, comparing with raw data, then compares contexts.
fn roundtrip_seq(definitions: &str, msgs: Vec<(TemplateData, Vec<u8>)>) {
    let mut d = Decoder::new_from_xml(definitions).unwrap();
    let mut e = Encoder::new_from_xml(definitions).unwrap();
    for (i, (data, raw)) in msgs.into_iter().enumerate() {
        let mut visitor = ModelVisitor::new(data.clone());
        assert_eq!(
            e.encode_vec(&mut visitor).unwrap(),
            raw,
            "#{} encode",
            i + 1
        );
        let mut factory = ModelFactory::new();
        d.decode_slice(&raw, &mut factory).unwrap();
        assert_eq!(factory.data.unwrap(), data, "#{} decode", i + 1);
    }
    assert_eq!(d.context, e.context, "context mismatch");
}

fn group(fields: Vec<(&str, ValueData)>) -> ValueData {
    ValueData::Group(Fields::from_iter(
        fields.into_iter().map(|(k, v)| (k.to_string(), v)),
    ))
}

fn uint32(v: u32) -> ValueData {
    ValueData::Value(Some(Value::UInt32(v)))
}

#[test]
fn static_reference_changes_current_template() {
    // `Header` is defined after the referring templates. Its `template` dictionary is shared by
    // all the references, and differs from the dictionary of `Header2`.
    do_test_seq(
        true,
        true,
        true,
        DEFINITION_REFS,
        TestCaseSeq {
            name: "static reference",
            raw: vec![
                vec![0xe0, 0x81, 0x85],
                vec![0xc0, 0x82],
                vec![0xc0, 0x81],
                vec![0xe0, 0x83, 0x85],
            ],
            data: vec![
                msg("Outer", Some(Value::UInt32(5))),
                msg("Other", Some(Value::UInt32(5))),
                msg("Outer", Some(Value::UInt32(5))),
                msg("Third", Some(Value::UInt32(5))),
            ],
        },
    );
}

#[test]
fn cyclic_static_reference() {
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
        <template id="1" name="A"><templateRef name="B"/></template>
        <template id="2" name="B"><templateRef name="A"/></template>
    </templates>"#;
    assert!(matches!(Decoder::new_from_xml(xml), Err(Error::Static(_))));
}

#[test]
fn dynamic_reference_template_id_is_global() {
    // All template identifiers share the copy operator entry in the `global` dictionary.
    let data = TemplateData {
        name: "DynamicOuter".to_string(),
        value: group(vec![(
            "templateRef:0",
            ValueData::DynamicTemplateRef(Box::new(TemplateData {
                name: "DynamicInner".to_string(),
                value: group(vec![("Value", uint32(5))]),
            })),
        )]),
    };
    roundtrip_seq(
        DEFINITION_REFS,
        vec![
            (data.clone(), vec![0xc0, 0x84, 0xc0, 0x85, 0x85]),
            (data, vec![0xc0, 0x84, 0xc0, 0x85, 0x85]),
        ],
    );
}

#[test]
fn template_dictionary_keys() {
    let data = TemplateData {
        name: "TwoKeys".to_string(),
        value: group(vec![("A", uint32(1)), ("B", uint32(2))]),
    };
    roundtrip_seq(
        DEFINITION_REFS,
        vec![
            (data.clone(), vec![0xf0, 0x88, 0x82, 0x83]),
            (data, vec![0x80]),
        ],
    );
}

#[test]
fn decimal_single_component_operator() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION_REFS,
        TestCaseSeq {
            name: "decimal exponent operator",
            raw: vec![vec![0xc0, 0x86, 0x00, 0xfb], vec![0xa0, 0xff, 0x85]],
            data: vec![
                msg(
                    "DecimalExponent",
                    Some(Value::Decimal(Decimal::new(-2, 123))),
                ),
                msg("DecimalExponent", Some(Value::Decimal(Decimal::new(-1, 5)))),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION_REFS,
        TestCaseSeq {
            name: "decimal mantissa operator",
            raw: vec![vec![0xc0, 0x87, 0xfe, 0x00, 0xe4], vec![0x80, 0xfe, 0x81]],
            data: vec![
                msg(
                    "DecimalMantissa",
                    Some(Value::Decimal(Decimal::new(-2, 100))),
                ),
                msg(
                    "DecimalMantissa",
                    Some(Value::Decimal(Decimal::new(-2, 101))),
                ),
            ],
        },
    );
}

#[test]
fn decimal_increment_is_rejected() {
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
        <template id="1" name="A"><decimal id="1" name="Value"><increment/></decimal></template>
    </templates>"#;
    assert!(matches!(Decoder::new_from_xml(xml), Err(Error::Static(_))));
}

const DEFINITION_XML: &str = include_str!("templates/compliance_xml.xml");

fn load_error(xml: &str) -> bool {
    matches!(Decoder::new_from_xml(xml), Err(Error::Static(_)))
}

fn template_xml(body: &str) -> String {
    format!(
        r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
            <template id="1" name="T">{body}</template>
        </templates>"#
    )
}

#[test]
fn root_template_element() {
    let xml = r#"<?xml version="1.0"?>
        <!-- a comment before the root -->
        <template xmlns="http://www.fixprotocol.org/ns/fast/td/1.1" id="1" name="T">
            <uInt32 id="1" name="Value"/>
        </template>"#;
    do_test(
        true,
        true,
        true,
        xml,
        TestCase {
            name: "root template",
            raw: vec![0xc0, 0x81, 0x85],
            data: msg("T", Some(Value::UInt32(5))),
        },
    );
    assert!(load_error(
        r#"<foo xmlns="http://www.fixprotocol.org/ns/fast/td/1.1"/>"#
    ));
}

#[test]
fn foreign_elements_and_attributes_are_ignored() {
    do_test(
        true,
        true,
        true,
        DEFINITION_XML,
        TestCase {
            name: "foreign",
            raw: vec![0xe0, 0x81, 0x85],
            data: msg("Foreign", Some(Value::UInt32(5))),
        },
    );
}

#[test]
fn template_namespaces() {
    for (template, raw, value) in [
        ("RefB", vec![0xc0, 0x82], 7),
        ("RefA", vec![0xc0, 0x83, 0x85], 5),
    ] {
        do_test(
            true,
            true,
            true,
            DEFINITION_XML,
            TestCase {
                name: template,
                raw,
                data: msg(template, Some(Value::UInt32(value))),
            },
        );
    }

    // Selecting a template by an ambiguous local name is an error.
    let mut e = Encoder::new_from_xml(DEFINITION_XML).unwrap();
    let mut m = ModelVisitor::new(pack_value("Header", "Value", Some(Value::UInt32(5))));
    assert!(matches!(e.encode_vec(&mut m), Err(Error::Dynamic(_))));

    // The same name in the same namespace.
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1" templateNs="urn:a">
        <template id="1" name="T"/>
        <template id="2" name="T"/>
    </templates>"#;
    assert!(load_error(xml));
    // Unknown static reference.
    assert!(load_error(&template_xml(
        r#"<templateRef name="T" templateNs="urn:x"/>"#
    )));
}

#[test]
fn key_and_type_namespaces() {
    let d = Decoder::new_from_xml(DEFINITION_XML).unwrap();
    let slots: Vec<_> = d
        .definitions
        .template_by_id(4)
        .unwrap()
        .instructions
        .iter()
        .map(|i| i.dictionary_slot.get())
        .collect();
    assert_ne!(slots[0], slots[1], "k in urn:x and urn:y");
    assert_eq!(slots[0], slots[2], "explicit key namespace");
    assert_ne!(slots[3], slots[4], "default keys in different namespaces");

    let t1 = &d.definitions.template_by_id(12).unwrap().type_ref;
    let t2 = &d.definitions.template_by_id(13).unwrap().type_ref;
    assert_ne!(Definitions::type_index(t1), Definitions::type_index(t2));
}

#[test]
fn optional_id_attribute() {
    do_test(
        true,
        true,
        true,
        DEFINITION_XML,
        TestCase {
            name: "no id",
            raw: vec![0xc0, 0x85, 0x85],
            data: msg("NoId", Some(Value::UInt32(5))),
        },
    );
    assert!(load_error(&template_xml(
        r#"<uInt32 id="abc" name="Value"/>"#
    )));
    assert!(load_error(&template_xml(r#"<uInt32 id="1"/>"#)));
    assert!(load_error(
        r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
            <template id="x" name="T"/>
        </templates>"#
    ));
}

#[test]
fn byte_vector_length() {
    let decode = |raw: &[u8]| {
        let mut d = Decoder::new_from_xml(DEFINITION_XML).unwrap();
        let mut m = LoggingMessageFactory::new();
        d.decode_slice(raw, &mut m).unwrap();
        m.calls
    };
    assert_eq!(
        decode(&[0xc0, 0x86, 0x83, 0x01, 0x02]),
        vec![
            "start_template: 6:BytesLength",
            "set_value: 9:Len Some(UInt32(2))",
            "set_value: 2:Value Some(Bytes([1, 2]))",
            "stop_template",
        ]
    );
    assert_eq!(
        decode(&[0xc0, 0x86, 0x80]),
        vec![
            "start_template: 6:BytesLength",
            "set_value: 9:Len None",
            "set_value: 2:Value None",
            "stop_template",
        ]
    );

    // The encoder doesn't need the length field.
    do_test(
        true,
        true,
        true,
        DEFINITION_XML,
        TestCase {
            name: "byteVector length",
            raw: vec![0xc0, 0x86, 0x83, 0x01, 0x02],
            data: msg("BytesLength", Some(Value::Bytes(vec![1, 2]))),
        },
    );
    do_test_seq(
        true,
        true,
        true,
        DEFINITION_XML,
        TestCaseSeq {
            name: "unicode length",
            raw: vec![vec![0xe0, 0x87, 0x82, 0xc3, 0xa9], vec![0x80]],
            data: vec![
                msg("UnicodeLength", Some(Value::UnicodeString("é".to_string()))),
                msg("UnicodeLength", Some(Value::UnicodeString("é".to_string()))),
            ],
        },
    );

    // Serde structs don't need to declare the length field.
    #[derive(serde_derive::Deserialize, Debug, PartialEq)]
    enum Message {
        UnicodeLength(UnicodeLength),
    }
    #[derive(serde_derive::Deserialize, Debug, PartialEq)]
    struct UnicodeLength {
        #[serde(rename = "Value")]
        value: String,
    }
    let mut d = Decoder::new_from_xml(DEFINITION_XML).unwrap();
    let m: Message = crate::from_slice(&mut d, &[0xe0, 0x87, 0x82, 0xc3, 0xa9]).unwrap();
    assert_eq!(
        m,
        Message::UnicodeLength(UnicodeLength {
            value: "é".to_string()
        })
    );

    assert!(load_error(&template_xml(
        r#"<string id="1" name="Value"><length name="L"/></string>"#
    )));
    assert!(load_error(&template_xml(
        r#"<byteVector id="1" name="Value"><length/></byteVector>"#
    )));
    assert!(load_error(&template_xml(
        r#"<byteVector id="1" name="Value"><length name="L"/><length name="M"/></byteVector>"#
    )));
    assert!(load_error(&template_xml(
        r#"<uInt32 id="1" name="Value"><copy/><default value="1"/></uInt32>"#
    )));
}

#[test]
fn initial_value_conversion() {
    for (template, raw, value) in [
        ("Trimmed", vec![0xc0, 0x88], Value::UInt32(5)),
        (
            "TrimmedDecimal",
            vec![0xc0, 0x89],
            Value::Decimal(Decimal::new(-1, 15)),
        ),
        (
            "HexWhitespace",
            vec![0xc0, 0x8a],
            Value::Bytes(vec![0x01, 0x02, 0x0a, 0xff]),
        ),
        (
            "LargeDecimal",
            vec![0xc0, 0x8b],
            Value::Decimal(Decimal::new(20, 1)),
        ),
    ] {
        do_test(
            true,
            true,
            true,
            DEFINITION_XML,
            TestCase {
                name: template,
                raw,
                data: msg(template, Some(value)),
            },
        );
    }

    // [ERR S3]
    let tiny = format!("0.{}1", "0".repeat(63));
    for body in [
        r#"<uInt32 id="1" name="Value"><constant value="4294967296"/></uInt32>"#,
        r#"<uInt32 id="1" name="Value"><constant value="-1"/></uInt32>"#,
        r#"<int32 id="1" name="Value"><default value="abc"/></int32>"#,
        r#"<decimal id="1" name="Value"><constant value="1e5"/></decimal>"#,
        r#"<decimal id="1" name="Value"><copy value="9223372036854775808"/></decimal>"#,
        &format!(r#"<decimal id="1" name="Value"><constant value="{tiny}"/></decimal>"#),
        r#"<decimal id="1" name="Value"><exponent><copy value="64"/></exponent></decimal>"#,
        r#"<string id="1" name="Value"><constant value="é"/></string>"#,
        r#"<byteVector id="1" name="Value"><constant value="0"/></byteVector>"#,
        r#"<byteVector id="1" name="Value"><constant value="zz"/></byteVector>"#,
    ] {
        assert!(load_error(&template_xml(body)), "{body}");
    }
}

#[test]
fn string_conversions() {
    for (input, expected) in [
        (" .5 ", (-1, 5)),
        ("-.5", (-1, -5)),
        ("+1", (0, 1)),
        ("1.", (0, 1)),
        ("-0.0", (0, 0)),
        ("1200", (2, 12)),
        ("-9223372036854775808", (0, i64::MIN)),
        (
            "0.000000000000000000000000000000000000000000000000000000000000001",
            (-63, 1),
        ),
    ] {
        let d = Decimal::from_string(input).unwrap();
        assert_eq!((d.exponent, d.mantissa), expected, "{input}");
    }
    let tiny = format!("0.{}1", "0".repeat(63));
    for input in [
        "",
        ".",
        "-",
        "1e5",
        "1.2.3",
        "- 1",
        "9223372036854775808",
        &tiny,
    ] {
        assert!(
            matches!(Decimal::from_string(input), Err(Error::Dynamic(_))),
            "{input:?}"
        );
    }

    let mut v = Value::Int32(0);
    v.set_from_string("\t-42\n").unwrap();
    assert_eq!(v, Value::Int32(-42));
    assert!(matches!(
        v.set_from_string("2147483648"),
        Err(Error::Dynamic(_))
    ));
    assert!(matches!(v.set_from_string("4 2"), Err(Error::Dynamic(_))));
    let mut v = Value::ASCIIString(String::new());
    assert!(matches!(v.set_from_string("é"), Err(Error::Dynamic(_))));
}

fn decode_seq_result(xml: &str, raw: &[&[u8]]) -> crate::Result<()> {
    let mut d = Decoder::new_from_xml(xml).unwrap();
    let (last, init) = raw.split_last().unwrap();
    for r in init {
        d.decode_slice(r, &mut ModelFactory::new()).unwrap();
    }
    d.decode_slice(last, &mut ModelFactory::new())
}

fn is_dynamic(r: crate::Result<()>, text: &str) -> bool {
    matches!(r, Err(Error::Dynamic(m)) if m.contains(text))
}

#[test]
fn exponent_range_after_operators() {
    // [ERR R1] the exponent is out of range after the delta is applied.
    let xml = template_xml(
        r#"<decimal id="1" name="Value"><exponent><delta/></exponent><mantissa><delta/></mantissa></decimal>"#,
    );
    assert!(decode_seq_result(&xml, &[&[0xc0, 0x81, 0xbf, 0x81]]).is_ok());
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x81, 0xbf, 0x81], &[0x80, 0x81, 0x81]]),
        "exponent value is out of range"
    ));

    // The encoder rejects such values too.
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    let mut m = ModelVisitor::new(pack_value(
        "T",
        "Value",
        Some(Value::Decimal(Decimal::new(64, 1))),
    ));
    assert!(matches!(e.encode_vec(&mut m), Err(Error::Dynamic(_))));
}

#[test]
fn overlong_integers() {
    let xml = template_xml(r#"<uInt32 id="1" name="A"/><int32 id="2" name="B"/>"#);
    // 64 needs the sign extension byte.
    assert!(decode_seq_result(&xml, &[&[0xc0, 0x81, 0x81, 0x00, 0xc0]]).is_ok());
    // [ERR R6]
    for raw in [
        &[0xc0, 0x00, 0x81, 0x81, 0x81][..],
        &[0xc0, 0x81, 0x00, 0x81, 0x81],
        &[0xc0, 0x81, 0x81, 0x00, 0x81],
        &[0xc0, 0x81, 0x81, 0x7f, 0xff],
    ] {
        assert!(
            is_dynamic(decode_seq_result(&xml, &[raw]), "overlong integer"),
            "{raw:x?}"
        );
    }
}

#[test]
fn presence_map_reportable_errors() {
    let xml = template_xml(r#"<uInt32 id="1" name="Value"/>"#);
    assert!(decode_seq_result(&xml, &[&[0xc0, 0x81, 0x81]]).is_ok());
    // [ERR R7] overlong presence map.
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0x40, 0x80, 0x81, 0x81]]),
        "overlong presence map"
    ));
    // [ERR R8] a bit not used by the template is set.
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xe0, 0x81, 0x81]]),
        "more bits than required"
    ));
}

#[test]
fn decoder_error_categories() {
    // [ERR D5] mandatory copy without initial value.
    let xml = template_xml(r#"<uInt32 id="1" name="Value"><copy/></uInt32>"#);
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x81]]),
        "no initial value"
    ));
    // [ERR D2] uInt32 out of range.
    let xml = template_xml(r#"<uInt32 id="1" name="Value"/>"#);
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x81, 0x10, 0x00, 0x00, 0x00, 0x80]]),
        "out of range"
    ));
    // Bytes left after the message.
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x81, 0x81, 0x81]]),
        "Bytes left"
    ));
    // [ERR R2] invalid UTF-8.
    let xml = template_xml(r#"<string id="1" name="Value" charset="unicode"/>"#);
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x81, 0x81, 0xff]]),
        "invalid UTF-8"
    ));
}

#[test]
fn text_visitor_dynamic_template_ref() {
    let definitions = include_str!("templates/base.xml");
    let raw = [0xc0, 0x89, 0x86, 0xe0, 0x87, 0x85];
    let mut d = Decoder::new_from_xml(definitions).unwrap();
    let mut factory = crate::TextMessageFactory::new();
    d.decode_slice(&raw, &mut factory).unwrap();
    assert_eq!(
        factory.text,
        "DynamicReference=<PreRefData=6|TemplateReference=<RefData=<TestData=5>>>"
    );

    let mut e = Encoder::new_from_xml(definitions).unwrap();
    let mut visitor = crate::TextMessageVisitor::from_text(&factory.text).unwrap();
    assert_eq!(e.encode_vec(&mut visitor).unwrap(), raw);

    // Missing mandatory dynamic template reference.
    let mut e = Encoder::new_from_xml(definitions).unwrap();
    let mut visitor =
        crate::TextMessageVisitor::from_text("DynamicReference=<PreRefData=6>").unwrap();
    assert!(e.encode_vec(&mut visitor).is_err());
}

#[test]
fn delta_subtraction_length_int32_min() {
    // [ERR D7] the subtraction length i32::MIN is larger than the base value.
    for field in [
        r#"<string id="1" name="Value"><delta/></string>"#,
        r#"<byteVector id="1" name="Value"><delta/></byteVector>"#,
    ] {
        let xml = template_xml(field);
        assert!(
            is_dynamic(
                decode_seq_result(&xml, &[&[0xc0, 0x81, 0x78, 0x00, 0x00, 0x00, 0x80, 0x80]]),
                "subtraction length"
            ),
            "{field}"
        );
    }
}

#[test]
fn optional_int64_delta_65_bits() {
    // The nullable delta 2^64 - 1 is encoded as 2^64.
    let xml = template_xml(r#"<uInt64 id="1" name="Value" presence="optional"><delta/></uInt64>"#);
    do_test_seq(
        true,
        true,
        true,
        &xml,
        TestCaseSeq {
            name: "optional uInt64 65-bit delta",
            raw: vec![
                vec![
                    0xc0, 0x81, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
                ],
                vec![0x80, 0x80],
            ],
            data: vec![msg("T", Some(Value::UInt64(u64::MAX))), msg("T", None)],
        },
    );
}

#[test]
fn character_set_conversion_with_operators() {
    // Values of the other character set are converted to the field type before operators are applied,
    // so the encoding and the previous values are the same as for values of the field type.
    let unicode = |s: &str| Value::UnicodeString(s.to_string());
    let ascii = |s: &str| Value::ASCIIString(s.to_string());
    for op in ["", "<copy/>", "<delta/>", "<tail/>"] {
        for (charset, converted, native) in [
            (
                r#"charset="unicode""#,
                &ascii as &dyn Fn(&str) -> Value,
                &unicode as &dyn Fn(&str) -> Value,
            ),
            ("", &unicode, &ascii),
            (r#"charset="ascii""#, &unicode, &ascii),
        ] {
            let xml = template_xml(&format!(
                r#"<string id="1" name="Value" {charset}>{op}</string>"#
            ));
            let mut e1 = Encoder::new_from_xml(&xml).unwrap();
            let mut e2 = Encoder::new_from_xml(&xml).unwrap();
            for s in ["ab", "ab", "abc"] {
                let r1 = e1
                    .encode_vec(&mut ModelVisitor::new(pack_value(
                        "T",
                        "Value",
                        Some(converted(s)),
                    )))
                    .unwrap();
                let r2 = e2
                    .encode_vec(&mut ModelVisitor::new(pack_value(
                        "T",
                        "Value",
                        Some(native(s)),
                    )))
                    .unwrap();
                assert_eq!(r1, r2, "{charset} {op} {s}");
            }
            assert_eq!(e1.context, e2.context, "{charset} {op}");
        }
    }

    // [ERR R3] a Unicode string with non-ASCII characters can't be converted.
    let xml = template_xml(r#"<string id="1" name="Value"><copy/></string>"#);
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    let mut m = ModelVisitor::new(pack_value("T", "Value", Some(unicode("é"))));
    assert!(matches!(e.encode_vec(&mut m), Err(Error::Runtime(_))));
}

#[test]
fn template_identifiers() {
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
        <template id="0" name="Zero"><uInt32 id="1" name="Value"/></template>
        <template name="NoId"><uInt32 id="1" name="Value"/></template>
        <template id="1" name="Ref"><templateRef name="NoId"/></template>
    </templates>"#;

    // `0` is a valid template identifier.
    do_test(
        true,
        true,
        true,
        xml,
        TestCase {
            name: "template id 0",
            raw: vec![0xc0, 0x80, 0x81],
            data: msg("Zero", Some(Value::UInt32(1))),
        },
    );

    // A template without identifier can be used by a static reference.
    let raw = [0xc0, 0x81, 0x82];
    let mut d = Decoder::new_from_xml(xml).unwrap();
    let mut factory = crate::TextMessageFactory::new();
    d.decode_slice(&raw, &mut factory).unwrap();
    let mut e = Encoder::new_from_xml(xml).unwrap();
    let mut visitor = crate::TextMessageVisitor::from_text(&factory.text).unwrap();
    assert_eq!(e.encode_vec(&mut visitor).unwrap(), raw);

    // But it can't be encoded as a message.
    let mut e = Encoder::new_from_xml(xml).unwrap();
    let mut m = ModelVisitor::new(pack_value("NoId", "Value", Some(Value::UInt32(1))));
    assert!(matches!(e.encode_vec(&mut m), Err(Error::Runtime(m)) if m.contains("has no id")));
}

#[test]
fn sequence_without_instructions() {
    // The sequence has the implicit length field only.
    for body in [
        r#"<sequence name="Value"/>"#,
        r#"<sequence name="Value"><typeRef name="X"/></sequence>"#,
    ] {
        let xml = template_xml(body);
        let raw = [0xc0, 0x81, 0x82];
        let mut d = Decoder::new_from_xml(&xml).unwrap();
        let mut msg = ModelFactory::new();
        d.decode_slice(&raw, &mut msg).unwrap();
        let data = msg.data.unwrap();
        match &data.value {
            ValueData::Group(g) => match g.get("Value") {
                Some(ValueData::Sequence(items)) => assert_eq!(items.len(), 2, "{body}"),
                v => panic!("{body}: sequence expected, got {v:?}"),
            },
            v => panic!("{body}: group expected, got {v:?}"),
        }
        let mut e = Encoder::new_from_xml(&xml).unwrap();
        assert_eq!(
            e.encode_vec(&mut ModelVisitor::new(data)).unwrap(),
            raw,
            "{body}"
        );
    }
}

#[test]
fn unicode_delta_and_tail_on_previous_value() {
    let unicode = |s: &str| Some(Value::UnicodeString(s.to_string()));
    // Byte-level splices can replace a part of a multi-byte character.
    do_test_seq(
        true,
        true,
        true,
        &template_xml(r#"<string id="1" name="Value" charset="unicode"><delta/></string>"#),
        TestCaseSeq {
            name: "unicode delta",
            raw: vec![
                vec![0xc0, 0x81, 0x80, 0x83, 0x61, 0xc3, 0xa9],
                vec![0x80, 0x81, 0x81, 0xa8],
                vec![0x80, 0xff, 0x82, 0xc3, 0xbc],
            ],
            data: vec![
                msg("T", unicode("aé")),
                msg("T", unicode("aè")),
                msg("T", unicode("üaè")),
            ],
        },
    );
    do_test_seq(
        true,
        true,
        true,
        &template_xml(r#"<string id="1" name="Value" charset="unicode"><tail/></string>"#),
        TestCaseSeq {
            name: "unicode tail",
            raw: vec![
                vec![0xe0, 0x81, 0x83, 0x61, 0xc3, 0xa9],
                vec![0xa0, 0x81, 0xa8],
            ],
            data: vec![msg("T", unicode("aé")), msg("T", unicode("aè"))],
        },
    );
    // [ERR R2] the combined value is not valid UTF-8.
    let xml = template_xml(r#"<string id="1" name="Value" charset="unicode"><delta/></string>"#);
    assert!(is_dynamic(
        decode_seq_result(
            &xml,
            &[
                &[0xc0, 0x81, 0x80, 0x83, 0x61, 0xc3, 0xa9],
                &[0x80, 0x81, 0x81, 0x41]
            ]
        ),
        "invalid UTF-8"
    ));
}

#[test]
fn decode_after_failed_message() {
    let xml =
        template_xml(r#"<group name="G"><typeRef name="X"/><uInt32 id="1" name="Value"/></group>"#);
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    // The message ends inside the application type scope of the group.
    assert!(
        d.decode_slice(&[0xc0, 0x81], &mut ModelFactory::new())
            .is_err()
    );
    d.decode_slice(&[0xc0, 0x81, 0x82], &mut ModelFactory::new())
        .unwrap();
    // Scopes of the failed message don't leak into the next one.
    assert_eq!(d.scopes.type_ref.peek(), Some(&0));
    assert!(d.scopes.type_ref.stack.is_empty());
    assert!(d.scopes.template.peek().is_none());
    assert!(d.scopes.presence_map.peek().is_none());
}

fn decode_text(d: &mut Decoder, raw: &[u8]) -> crate::Result<String> {
    let mut factory = crate::TextMessageFactory::new();
    d.decode_slice(raw, &mut factory)?;
    Ok(factory.text)
}

fn encode_text(e: &mut Encoder, text: &str) -> crate::Result<Vec<u8>> {
    e.encode_vec(&mut crate::TextMessageVisitor::from_text(text)?)
}

#[test]
fn misplaced_length_exponent_mantissa_are_rejected() {
    // [ERR S1] <length>, <exponent> and <mantissa> are parts of other fields, not instructions.
    for body in [
        r#"<length name="L"/>"#,
        r#"<group name="G"><length name="L"/></group>"#,
        r#"<sequence name="S"><uInt32 name="a"/><length name="L"/></sequence>"#,
        r#"<sequence name="S"><length name="L"/><length name="M"/><uInt32 name="a"/></sequence>"#,
        r#"<mantissa/>"#,
        r#"<exponent><copy/></exponent>"#,
        r#"<group name="G"><mantissa><copy/></mantissa></group>"#,
        r#"<sequence name="S"><exponent/></sequence>"#,
    ] {
        assert!(load_error(&template_xml(body)), "{body}");
    }
    // <length> follows the optional <typeRef>.
    for body in [
        r#"<sequence name="S"><length name="L"/><uInt32 name="a"/></sequence>"#,
        r#"<sequence name="S"><typeRef name="X"/><length name="L"/><uInt32 name="a"/></sequence>"#,
    ] {
        assert!(Decoder::new_from_xml(&template_xml(body)).is_ok(), "{body}");
    }
}

#[test]
fn foreign_root_namespace_is_rejected() {
    // [ERR S1] elements of other namespaces are foreign, so such a document has no templates.
    let body = r#"<template id="1" name="T"><uInt32 id="1" name="Value"/></template>"#;
    for ns in [
        "http://www.fixprotocol.org/ns/template-definition",
        "urn:foreign",
    ] {
        assert!(
            load_error(&format!(r#"<templates xmlns="{ns}">{body}</templates>"#)),
            "{ns}"
        );
        assert!(
            load_error(&format!(r#"<template xmlns="{ns}" id="1" name="T"/>"#)),
            "{ns}"
        );
    }
    // Documents without namespace are accepted.
    do_test(
        true,
        true,
        true,
        &format!("<templates>{body}</templates>"),
        TestCase {
            name: "no namespace",
            raw: vec![0xc0, 0x81, 0x85],
            data: msg("T", Some(Value::UInt32(5))),
        },
    );
}

#[test]
fn duplicate_template_id_is_rejected() {
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
        <template id="1" name="A"/>
        <template id="1" name="B"/>
    </templates>"#;
    assert!(load_error(xml));
    // Templates without identifiers don't clash.
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
        <template name="A"/>
        <template name="B"/>
    </templates>"#;
    assert!(Decoder::new_from_xml(xml).is_ok());
}

#[test]
fn generated_names_do_not_collide() {
    // The template identifier uses an internal key.
    let xml = template_xml(r#"<uInt32 id="1" name="__template_id__"><copy/></uInt32>"#);
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    assert_eq!(
        decode_text(&mut d, &[0xe0, 0x81, 0x87]).unwrap(),
        "T=<__template_id__=7>"
    );
    assert_eq!(
        decode_text(&mut d, &[0x80]).unwrap(),
        "T=<__template_id__=7>"
    );

    // The generated name of a sequence length never collides with a field name (§6.2.5).
    let xml = template_xml(
        r#"<sequence name="S"><length><copy/></length><uInt32 name="a"/></sequence>
           <uInt32 name="S:length"><copy/></uInt32>"#,
    );
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xe0, 0x81, 0x81, 0x85]]),
        "no initial value"
    ));

    // The same for the generated names of the decimal parts (§6.2.2).
    let xml = template_xml(
        r#"<decimal name="D"><exponent><copy/></exponent><mantissa><copy/></mantissa></decimal>
           <int32 name="D:exponent"><copy/></int32>
           <int64 name="D:mantissa"><copy/></int64>"#,
    );
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xf0, 0x81, 0xfe, 0x85]]),
        "no initial value"
    ));
    let d = Decoder::new_from_xml(&xml).unwrap();
    let t = &d.definitions.template_by_id(1).unwrap().instructions;
    let slot = |i: &Instruction| i.dictionary_slot.get();
    assert_ne!(slot(&t[0].instructions[0]), slot(&t[1]));
    assert_ne!(slot(&t[0].instructions[1]), slot(&t[2]));
}

#[test]
fn decimal_constant_exponent_rescales() {
    // The constant exponent limits the values to the ones representable with it; [ERR D3] is for values
    // that cannot be encoded only.
    let xml = template_xml(
        r#"<decimal id="1" name="Value"><exponent><constant value="-2"/></exponent><mantissa><delta/></mantissa></decimal>"#,
    );
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    let encode = |e: &mut Encoder, exponent: i32, mantissa: i64| {
        e.encode_vec(&mut ModelVisitor::new(pack_value(
            "T",
            "Value",
            Some(Value::Decimal(Decimal::new(exponent, mantissa))),
        )))
    };
    let mut first = true;
    for ((exponent, mantissa), scaled) in [
        ((-1, 15), 150),
        ((-2, 125), 125),
        ((0, 2), 200),
        ((2, 1), 10_000),
        ((-4, 12_300), 123),
        ((0, 0), 0),
        ((5, 0), 0),
        ((1, -3), -3000),
    ] {
        let raw = encode(&mut e, exponent, mantissa).unwrap();
        if first {
            assert_eq!(raw, vec![0xc0, 0x81, 0x01, 0x96]);
            first = false;
        }
        let mut msg = ModelFactory::new();
        d.decode_slice(&raw, &mut msg).unwrap();
        assert_eq!(
            extract_value(msg, "Value", "constant exponent"),
            Some(Value::Decimal(Decimal::new(-2, scaled))),
            "{mantissa}e{exponent}"
        );
    }
    assert_eq!(d.context, e.context);

    // [ERR D3] the precision or the range of the mantissa is not enough.
    for (exponent, mantissa) in [(-3, 1255), (-4, 1), (30, 1), (17, i64::MAX)] {
        assert!(
            matches!(
                encode(&mut e, exponent, mantissa),
                Err(Error::Dynamic(m)) if m.contains("constant exponent")
            ),
            "{mantissa}e{exponent}"
        );
    }
    // The failed messages don't change the previous value of the mantissa.
    assert_eq!(d.context, e.context);
}

#[test]
fn empty_previous_value_of_mandatory_field() {
    // [ERR D6] the optional field sets the shared previous value empty.
    for (field, op, raw) in [
        ("uInt32", "copy", &[0xe0, 0x81, 0x80][..]),
        ("uInt32", "increment", &[0xe0, 0x81, 0x80]),
        ("uInt32", "delta", &[0xe0, 0x81, 0x80, 0x81]),
        ("string", "tail", &[0xe0, 0x81, 0x80]),
    ] {
        let xml = template_xml(&format!(
            r#"<{field} name="a" presence="optional"><copy key="k"/></{field}>
               <{field} name="b"><{op} key="k"/></{field}>"#
        ));
        assert!(
            is_dynamic(decode_seq_result(&xml, &[raw]), "no previous value"),
            "{op}"
        );
    }
}

#[test]
fn dictionary_type_mismatch() {
    // [ERR D4] for every operator that accesses the entry.
    for (op, raw) in [
        ("copy", &[0xe0, 0x81, 0x85][..]),
        ("delta", &[0xe0, 0x81, 0x85, 0x80, 0xc1]),
        ("tail", &[0xe0, 0x81, 0x85]),
        ("tail", &[0xf0, 0x81, 0x85, 0xc1]),
    ] {
        let xml = template_xml(&format!(
            r#"<uInt32 name="a"><copy key="k"/></uInt32><string name="b"><{op} key="k"/></string>"#
        ));
        assert!(
            is_dynamic(decode_seq_result(&xml, &[raw]), "wrong value type"),
            "{op} {raw:x?}"
        );
    }
}

#[test]
fn unknown_template_id() {
    // [ERR D9]
    let xml = template_xml(r#"<uInt32 id="1" name="Value"/>"#);
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x82, 0x81]]),
        "Unknown template id: 2"
    ));
    // The same for a dynamic template reference.
    let xml = template_xml("<templateRef/>");
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xc0, 0x81, 0xc0, 0x82]]),
        "Unknown template id: 2"
    ));
}

#[test]
fn delta_subtraction_length() {
    let xml = template_xml(r#"<string id="1" name="Value"><delta value="abc"/></string>"#);
    // [ERR D7] the subtraction length is larger than the base value: from the back and from the front.
    for raw in [&[0xc0, 0x81, 0x84, 0xc1][..], &[0xc0, 0x81, 0xfb, 0xc1]] {
        assert!(
            is_dynamic(decode_seq_result(&xml, &[raw]), "subtraction length"),
            "{raw:x?}"
        );
    }
    for (name, raw, value) in [
        (
            "remove all from the back",
            vec![0xc0, 0x81, 0x83, 0xc1],
            "A",
        ),
        (
            "remove all from the front",
            vec![0xc0, 0x81, 0xfc, 0xc1],
            "A",
        ),
        (
            "negative zero prepends",
            vec![0xc0, 0x81, 0xff, 0xc1],
            "Aabc",
        ),
        ("zero appends", vec![0xc0, 0x81, 0x80, 0xc1], "abcA"),
        ("remove from the front", vec![0xc0, 0x81, 0xfd, 0x80], "c"),
    ] {
        let mut d = Decoder::new_from_xml(&xml).unwrap();
        let mut m = ModelFactory::new();
        d.decode_slice(&raw, &mut m).unwrap();
        assert_eq!(extract_value(m, "Value", name), ascii(value), "{name}");
    }
}

#[test]
fn tail_with_initial_value() {
    let xml = template_xml(r#"<string id="1" name="Value"><tail value="abc"/></string>"#);
    do_test_seq(
        true,
        true,
        true,
        &xml,
        TestCaseSeq {
            name: "tail",
            raw: vec![
                // Undefined previous value: the value is the initial value.
                vec![0xc0, 0x81],
                vec![0xa0, 0xd8],
                // The tail is longer than the base value: the value is the tail.
                vec![0xa0, 0x77, 0x78, 0x79, 0xfa],
                vec![0x80],
            ],
            data: vec![
                msg("T", ascii("abc")),
                msg("T", ascii("abX")),
                msg("T", ascii("wxyz")),
                msg("T", ascii("wxyz")),
            ],
        },
    );
    // The empty tail removes nothing.
    assert!(decode_seq_result(&xml, &[&[0xe0, 0x81, 0x80]]).is_ok());

    // A value shorter than the base value can't be encoded as a tail; the previous value is kept.
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    assert_eq!(
        encode_text(&mut e, "T=<Value=abcd>").unwrap(),
        vec![0xe0, 0x81, 0x61, 0x62, 0x63, 0xe4]
    );
    assert!(encode_text(&mut e, "T=<Value=ab>").is_err());
    assert_eq!(
        encode_text(&mut e, "T=<Value=abce>").unwrap(),
        vec![0xa0, 0xe5]
    );
}

#[test]
fn optional_group_segment() {
    let xml = template_xml(
        r#"<group name="G" presence="optional"><uInt32 name="a"><copy value="1"/></uInt32></group>"#,
    );
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    // The instructions of an absent group are not processed.
    assert_eq!(decode_text(&mut d, &[0xc0, 0x81]).unwrap(), "T=<>");
    assert_eq!(d.context, {
        let mut other = Decoder::new_from_xml(&xml).unwrap();
        decode_text(&mut other, &[0xc0, 0x81]).unwrap();
        other.context
    });
    assert_eq!(
        decode_text(&mut d, &[0xe0, 0x81, 0x80]).unwrap(),
        "T=<G=<a=1>>"
    );
    assert_eq!(
        decode_text(&mut d, &[0xa0, 0xc0, 0x85]).unwrap(),
        "T=<G=<a=5>>"
    );
    // The absent group doesn't affect the previous values.
    assert_eq!(decode_text(&mut d, &[0x80]).unwrap(), "T=<>");
    assert_eq!(decode_text(&mut d, &[0xa0, 0x80]).unwrap(), "T=<G=<a=5>>");

    // The presence map of the group segment has its own [ERR R8] and [ERR R7] checks.
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xe0, 0x81, 0xa0]]),
        "more bits than required"
    ));
    assert!(is_dynamic(
        decode_seq_result(&xml, &[&[0xe0, 0x81, 0x00, 0x80]]),
        "overlong presence map"
    ));

    // Round trip through the text format, including messages and groups without present fields.
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    for (text, raw) in [
        ("T=<>", vec![0xc0, 0x81]),
        ("T=<G=<a=1>>", vec![0xa0, 0x80]),
        ("T=<G=<a=5>>", vec![0xa0, 0xc0, 0x85]),
        ("T=<>", vec![0x80]),
        ("T=<G=<a=5>>", vec![0xa0, 0x80]),
    ] {
        assert_eq!(encode_text(&mut e, text).unwrap(), raw, "{text}");
        assert_eq!(decode_text(&mut d, &raw).unwrap(), text, "{raw:x?}");
    }
    assert_eq!(d.context, e.context);

    // A present group without present fields.
    let xml = template_xml(
        r#"<group name="G" presence="optional"><uInt32 name="a" presence="optional"/></group>
           <sequence name="S"><uInt32 name="b" presence="optional"/></sequence>"#,
    );
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    for (text, raw) in [
        (
            "T=<G=<>|S=<><b=2><>>",
            vec![0xe0, 0x81, 0x80, 0x83, 0x80, 0x83, 0x80],
        ),
        ("T=<S=<>>", vec![0x80, 0x81, 0x80]),
    ] {
        assert_eq!(encode_text(&mut e, text).unwrap(), raw, "{text}");
        assert_eq!(decode_text(&mut d, &raw).unwrap(), text, "{raw:x?}");
    }
}

#[test]
fn optional_decimal_mantissa_depends_on_exponent() {
    // The presence map bit of the mantissa is present iff the exponent is present (§10.5.1).
    let xml = template_xml(
        r#"<decimal name="D" presence="optional"><exponent><constant value="-2"/></exponent><mantissa><copy/></mantissa></decimal>
           <uInt32 name="z"><copy value="9"/></uInt32>"#,
    );
    let mut d = Decoder::new_from_xml(&xml).unwrap();
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    for (text, raw) in [
        // template id, exponent (absent), z
        ("T=<z=7>", vec![0xd0, 0x81, 0x87]),
        // exponent, mantissa
        ("T=<D=0.05|z=7>", vec![0xb0, 0x85]),
        // exponent
        ("T=<D=0.05|z=7>", vec![0xa0]),
        ("T=<z=7>", vec![0x80]),
        // exponent, z
        ("T=<D=0.05|z=8>", vec![0xa8, 0x88]),
    ] {
        assert_eq!(encode_text(&mut e, text).unwrap(), raw, "{text}");
        assert_eq!(decode_text(&mut d, &raw).unwrap(), text, "{raw:x?}");
    }
    assert_eq!(d.context, e.context);
}

#[test]
fn user_defined_dictionaries() {
    // Operators share a user defined dictionary iff they specify identical dictionary names.
    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
        <template id="1" name="A"><uInt32 id="1" name="Value"><copy dictionary="d1"/></uInt32></template>
        <template id="2" name="B"><uInt32 id="1" name="Value"><copy dictionary="d1"/></uInt32></template>
        <template id="3" name="C"><uInt32 id="1" name="Value"><copy dictionary="d2"/></uInt32></template>
        <template id="4" name="D"><uInt32 id="1" name="Value"><copy/></uInt32></template>
    </templates>"#;
    do_test_seq(
        true,
        true,
        true,
        xml,
        TestCaseSeq {
            name: "user defined dictionaries",
            raw: vec![
                vec![0xe0, 0x81, 0x85],
                vec![0xc0, 0x82],
                vec![0xe0, 0x83, 0x85],
                vec![0xe0, 0x84, 0x85],
                vec![0xc0, 0x81],
            ],
            data: vec![
                msg("A", Some(Value::UInt32(5))),
                msg("B", Some(Value::UInt32(5))),
                msg("C", Some(Value::UInt32(5))),
                msg("D", Some(Value::UInt32(5))),
                msg("A", Some(Value::UInt32(5))),
            ],
        },
    );
}

#[test]
fn dictionary_attribute_of_nearest_element_applies() {
    use crate::common::context::DictionaryType;

    let xml = r#"<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1" dictionary="d0">
        <template id="1" name="T" dictionary="type">
            <uInt32 name="a"><copy/></uInt32>
            <group name="G" dictionary="template">
                <uInt32 name="b"><copy/></uInt32>
                <uInt32 name="c"><copy dictionary="global"/></uInt32>
                <sequence name="S" dictionary="d1">
                    <length name="L"><copy/></length>
                    <uInt32 name="d"><copy/></uInt32>
                    <uInt32 name="e"><copy dictionary="type"/></uInt32>
                </sequence>
            </group>
        </template>
        <template id="2" name="U">
            <uInt32 name="a"><copy/></uInt32>
            <uInt32 name="d"><copy dictionary="d1"/></uInt32>
            <uInt32 name="c"><copy dictionary="global"/></uInt32>
        </template>
    </templates>"#;
    let d = Decoder::new_from_xml(xml).unwrap();
    let slot = |i: &Instruction| i.dictionary_slot.get();
    let t = &d.definitions.template_by_id(1).unwrap().instructions;
    let g = &t[1].instructions;
    let s = &g[2].instructions;
    assert!(matches!(slot(&t[0]), DictionaryType::Type(_)), "template");
    assert!(matches!(slot(&g[0]), DictionaryType::Template(_)), "group");
    assert!(matches!(slot(&g[1]), DictionaryType::Static(_)), "operator");
    assert!(matches!(slot(&s[0]), DictionaryType::Static(_)), "length");
    assert!(matches!(slot(&s[1]), DictionaryType::Static(_)), "sequence");
    assert!(matches!(slot(&s[2]), DictionaryType::Type(_)), "operator");
    assert_ne!(slot(&g[1]), slot(&s[1]), "global and d1");

    let u = &d.definitions.template_by_id(2).unwrap().instructions;
    assert!(
        matches!(slot(&u[0]), DictionaryType::Static(_)),
        "templates"
    );
    assert_ne!(slot(&u[0]), slot(&g[1]), "d0 and global");
    assert_eq!(slot(&u[1]), slot(&s[1]), "d1");
    assert_eq!(slot(&u[2]), slot(&g[1]), "global");
}

#[test]
fn reset_sets_entries_undefined() {
    let xml = template_xml(r#"<uInt32 id="1" name="Value"><copy/></uInt32>"#);
    let data = || ModelVisitor::new(pack_value("T", "Value", Some(Value::UInt32(5))));

    let mut e = Encoder::new_from_xml(&xml).unwrap();
    assert_eq!(e.encode_vec(&mut data()).unwrap(), vec![0xe0, 0x81, 0x85]);
    assert_eq!(e.encode_vec(&mut data()).unwrap(), vec![0x80]);
    e.reset();
    assert_eq!(e.encode_vec(&mut data()).unwrap(), vec![0xe0, 0x81, 0x85]);

    let mut d = Decoder::new_from_xml(&xml).unwrap();
    assert_eq!(
        decode_text(&mut d, &[0xe0, 0x81, 0x85]).unwrap(),
        "T=<Value=5>"
    );
    assert_eq!(decode_text(&mut d, &[0x80]).unwrap(), "T=<Value=5>");
    d.reset();
    // Neither the template identifier nor the value has the previous value.
    assert!(matches!(
        decode_text(&mut d, &[0x80]),
        Err(Error::Dynamic(m)) if m.contains("no initial value")
    ));
    assert!(matches!(
        decode_text(&mut d, &[0xc0, 0x81]),
        Err(Error::Dynamic(m)) if m.contains("no initial value")
    ));
    assert_eq!(
        decode_text(&mut d, &[0xe0, 0x81, 0x86]).unwrap(),
        "T=<Value=6>"
    );
}

#[test]
fn stream_reader_matches_slice_reader() {
    let xml = template_xml(
        r#"<uInt32 name="A"/>
           <int32 name="B" presence="optional"/>
           <string name="C" presence="optional"/>
           <uInt64 name="D" presence="optional"/>
           <int64 name="E"><delta/></int64>
           <byteVector name="F" presence="optional"/>"#,
    );
    let mut valid = 0;
    for raw in [
        &[0xc0, 0x81, 0x81, 0x82, 0x00, 0x80, 0x82, 0x83, 0x82, 0xff][..],
        &[0xc0, 0x81, 0x81, 0x80, 0x80, 0x80, 0xff, 0x80],
        &[0xc0, 0x81, 0x81, 0x00, 0xc1, 0x41, 0xc2, 0x80, 0x80, 0x81],
        // The nullable representations of the maximum values.
        &[
            0xc0, 0x81, 0x81, 0x08, 0x00, 0x00, 0x00, 0x80, 0xc1, 0x02, 0, 0, 0, 0, 0, 0, 0, 0,
            0x80, 0x80, 0x80,
        ],
        // 10 bytes delta.
        &[
            0xc0, 0x81, 0x81, 0x80, 0x80, 0x80, 0x7f, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x80,
        ],
        // [ERR R6] overlong integers.
        &[0xc0, 0x00, 0x81],
        &[0xc0, 0x81, 0x00, 0x81],
        &[0xc0, 0x81, 0x81, 0x00, 0x81],
        &[0xc0, 0x81, 0x81, 0x7f, 0xff],
        &[0xc0, 0x81, 0x81, 0x80, 0x80, 0x80, 0x00, 0x81],
        // [ERR R9] overlong strings.
        &[0xc0, 0x81, 0x81, 0x80, 0x00, 0xc1],
        &[0xc0, 0x81, 0x81, 0x80, 0x00, 0x00, 0xc1],
        // [ERR D2] integers out of range.
        &[0xc0, 0x81, 0x10, 0x00, 0x00, 0x00, 0x80],
        &[0xc0, 0x81, 0x81, 0x10, 0x00, 0x00, 0x00, 0x80],
        &[
            0xc0, 0x81, 0x81, 0x80, 0x80, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0x81,
        ],
        &[
            0xc0, 0x81, 0x81, 0x80, 0x80, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0x80,
        ],
        // [ERR R7] [ERR R8] presence maps.
        &[0x40, 0x80, 0x81, 0x81],
        &[0xe0, 0x81, 0x81, 0x80, 0x80, 0x80, 0x80, 0x80],
        // The end of the stream.
        &[],
        &[0x40],
        &[0xc0, 0x81, 0x81],
        &[0xc0, 0x81, 0x81, 0x80, 0x41],
        &[0xc0, 0x81, 0x81, 0x80, 0x80, 0x80, 0x80, 0x84, 0x01],
    ] {
        let mut d1 = Decoder::new_from_xml(&xml).unwrap();
        let mut f1 = crate::TextMessageFactory::new();
        let r1 = d1.decode_slice(raw, &mut f1).map(|()| f1.text);

        let mut d2 = Decoder::new_from_xml(&xml).unwrap();
        let mut f2 = crate::TextMessageFactory::new();
        let r2 = d2.decode_stream(&mut &raw[..], &mut f2).map(|()| f2.text);

        valid += usize::from(r1.is_ok());
        assert_eq!(format!("{r1:?}"), format!("{r2:?}"), "{raw:x?}");
        assert_eq!(d1.context, d2.context, "{raw:x?}");
    }
    assert_eq!(valid, 5);
}
