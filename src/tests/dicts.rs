use crate::{Decimal, TextMessageFactory, TextMessageVisitor};

use super::*;

const DEFINITION1: &str = include_str!("templates/dicts1.xml");
const DEFINITION2: &str = include_str!("templates/dicts2.xml");

#[test]
fn operator_dictionary() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION1,
        TestCaseSeq {
            name: "Field with own dictionary",
            raw: vec![
                // Message1=<Value=a>, stored in `template` dictionary of Message1
                vec![0xe0, 0x81, 0xe1],
                // Message2=<Value=b>, stored in `global` dictionary
                vec![0xe0, 0x82, 0xe2],
                // Message1=<Value=a>, previous value is taken from `template` dictionary
                vec![0xc0, 0x81],
            ],
            data: vec![
                TestMsgValue {
                    template: "Message1",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
                TestMsgValue {
                    template: "Message2",
                    value: Some(Value::ASCIIString("b".to_string())),
                },
                TestMsgValue {
                    template: "Message1",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
            ],
        },
    );
}

#[test]
fn sequence_length_dictionary() {
    let mut d = Decoder::new_from_xml(DEFINITION1).unwrap();
    let mut e = Encoder::new_from_xml(DEFINITION1).unwrap();
    for text in [
        "Message3=<Items=<Value=1>>",
        "Message4=<Count=1>",
        "Message3=<Items=<Value=1>>",
    ] {
        let raw = e
            .encode_vec(&mut TextMessageVisitor::from_text(text).unwrap())
            .unwrap();
        let mut msg = TextMessageFactory::new();
        d.decode_slice(&raw, &mut msg).unwrap();
        assert_eq!(msg.text, text);
        assert_eq!(d.context, e.context, "{text}: context mismatch");
    }
}

#[test]
fn operator_key() {
    let mut d = Decoder::new_from_xml(DEFINITION1).unwrap();
    let mut e = Encoder::new_from_xml(DEFINITION1).unwrap();
    // Both fields share the previous value via the same key.
    let text = "Message5=<Value1=7|Value2=7>";
    let raw = e
        .encode_vec(&mut TextMessageVisitor::from_text(text).unwrap())
        .unwrap();
    assert_eq!(raw, vec![0xe0, 0x85, 0x87]);

    let mut msg = TextMessageFactory::new();
    d.decode_slice(&raw, &mut msg).unwrap();
    assert_eq!(msg.text, text);
    assert_eq!(d.context, e.context, "{text}: context mismatch");
}

#[test]
fn default_templates_dictionary() {
    do_test_seq(
        false,
        true,
        false,
        DEFINITION2,
        TestCaseSeq {
            name: "Default dictionary of <templates>",
            raw: vec![
                vec![0xe0, 0x81, 0xe1],
                // Message2 has its own `template` dictionary
                vec![0xe0, 0x82, 0xe1],
                vec![0xc0, 0x81],
                // Messag3 and Message4 override the default and share the `global` dictionary
                vec![0xe0, 0x83, 0xe1],
                vec![0xc0, 0x84],
            ],
            data: vec![
                TestMsgValue {
                    template: "Message1",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
                TestMsgValue {
                    template: "Message2",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
                TestMsgValue {
                    template: "Message1",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
                TestMsgValue {
                    template: "Message3",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
                TestMsgValue {
                    template: "Message4",
                    value: Some(Value::ASCIIString("a".to_string())),
                },
            ],
        }
    );
}

#[test]
fn decimal_operator_context() {
    let d = Decoder::new_from_xml(DEFINITION1).unwrap();
    let i = &d.definitions.templates_by_id[&6].instructions[0];
    assert_eq!(i.dictionary, Dictionary::Template);
    assert_eq!(i.key.as_ref(), "px");
    assert_eq!(i.instructions[0].key.as_ref(), "px:exponent");
    assert_eq!(i.instructions[1].key.as_ref(), "px:mantissa");

    do_test_seq(
        false,
        true,
        false,
        DEFINITION1,
        TestCaseSeq {
            name: "Decimal operator dictionary and key",
            raw: vec![
                vec![0xe0, 0x86, 0xff, 0x8f],
                // Message7 uses `global` dictionary
                vec![0xe0, 0x87, 0xff, 0x8f],
                vec![0xc0, 0x86],
            ],
            data: vec![
                TestMsgValue {
                    template: "Message6",
                    value: Some(Value::Decimal(Decimal::new(-1, 15))),
                },
                TestMsgValue {
                    template: "Message7",
                    value: Some(Value::Decimal(Decimal::new(-1, 15))),
                },
                TestMsgValue {
                    template: "Message6",
                    value: Some(Value::Decimal(Decimal::new(-1, 15))),
                },
            ],
        }
    );
}
