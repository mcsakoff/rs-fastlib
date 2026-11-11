//! # Regression tests for FAST 1.1 spec compliance fixes
//!
//! See: https://fixtrading.org/packages/fast-specification-version-1-1/
//!
use super::*;
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
    assert_ne!(d.definitions.type_index(t1), d.definitions.type_index(t2));
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
