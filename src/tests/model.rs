use hashbrown::HashMap;

use crate::{Decoder, Value};
use crate::model::{ModelFactory, template::TemplateData, value::ValueData};

struct TestCase {
    input: Vec<u8>,
    result: TemplateData,
}

fn do_test(test_cases: Vec<TestCase>) {
    for tt in test_cases {
        let mut msg = ModelFactory::new();
        let mut d = Decoder::new_from_xml(include_str!("templates/base.xml")).unwrap();
        d.decode_vec(tt.input.clone(), &mut msg).unwrap();
        assert_eq!(msg.data.unwrap(), tt.result, "{} failed", tt.result.name);
    }
}

#[test]
fn test_model_data() {
    do_test(vec![
        TestCase {
            input: vec![0xc0, 0x82, 0x61, 0x62, 0xe3, 0x64, 0x65, 0xe6, 0x83, 0x67, 0x68, 0x69, 0x84, 0x6b, 0x6c, 0x6d],
            result: TemplateData {
                name: "String".to_string(),
                value: ValueData::Group(HashMap::from([
                    (
                        "MandatoryAscii".to_string(),
                        ValueData::Value(Some(Value::ASCIIString("abc".to_string()))),
                    ), (
                        "OptionalAscii".to_string(),
                        ValueData::Value(Some(Value::ASCIIString("def".to_string()))),
                    ), (
                        "MandatoryUnicode".to_string(),
                        ValueData::Value(Some(Value::UnicodeString("ghi".to_string()))),
                    ), (
                        "OptionalUnicode".to_string(),
                        ValueData::Value(Some(Value::UnicodeString("klm".to_string()))),
                    )
                ])),
            },
        },
        TestCase {
            input: vec![0xc0, 0x83, 0x81, 0xc1, 0x82, 0xb3],
            result: TemplateData {
                name: "ByteVector".to_string(),
                value: ValueData::Group(HashMap::from([
                    (
                        "MandatoryVector".to_string(),
                        ValueData::Value(Some(Value::Bytes(vec![193]))),
                    ), (
                        "OptionalVector".to_string(),
                        ValueData::Value(Some(Value::Bytes(vec![179]))),
                    )
                ])),
            },
        },
        TestCase {
            input: vec![0xc0, 0x85, 0x81, 0x81, 0x82, 0x83, 0x83, 0x84, 0x81, 0xc0, 0x82],
            result: TemplateData {
                name: "Sequence".to_string(),
                value: ValueData::Group(HashMap::from([
                    (
                        "TestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(1))),
                    ), (
                        "OuterSequence".to_string(),
                        ValueData::Sequence(vec![
                            ValueData::Group(HashMap::from([
                                (
                                    "OuterTestData".to_string(),
                                    ValueData::Value(Some(Value::UInt32(2))),
                                ), (
                                    "InnerSequence".to_string(),
                                    ValueData::Sequence(vec![
                                        ValueData::Group(HashMap::from([
                                            (
                                                "InnerTestData".to_string(),
                                                ValueData::Value(Some(Value::UInt32(3))),
                                            )
                                        ])),
                                        ValueData::Group(HashMap::from([
                                            (
                                                "InnerTestData".to_string(),
                                                ValueData::Value(Some(Value::UInt32(4))),
                                            )
                                        ])),
                                    ])
                                )
                            ])),
                        ]),
                    ), (
                        "NextOuterSequence".to_string(),
                        ValueData::Sequence(vec![
                            ValueData::Group(HashMap::from([
                                (
                                    "NextOuterTestData".to_string(),
                                    ValueData::Value(Some(Value::UInt32(2))),
                                )
                            ])),
                        ])
                    )
                ])),
            },
        },
        TestCase {
            input: vec![0xc0, 0x86, 0x81, 0xc0, 0x82, 0x83],
            result: TemplateData {
                name: "Group".to_string(),
                value: ValueData::Group(HashMap::from([
                    (
                        "TestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(1))),
                    ), (
                        "OuterGroup".to_string(),
                        ValueData::Group(HashMap::from([
                            (
                                "OuterTestData".to_string(),
                                ValueData::Value(Some(Value::UInt32(2))),
                            ), (
                                "InnerGroup".to_string(),
                                ValueData::Group(HashMap::from([
                                    (
                                        "InnerTestData".to_string(),
                                        ValueData::Value(Some(Value::UInt32(3))),
                                    )
                                ])),
                            )
                        ])),
                    )
                ])),
            },
        },
        TestCase {
            input: vec![0xe0, 0x88, 0x86, 0x87],
            result: TemplateData {
                name: "StaticReference".to_string(),
                value: ValueData::Group(HashMap::from([
                    (
                        "PreRefData".to_string(),
                        ValueData::Value(Some(Value::UInt32(6))),
                    ), (
                        "TestData".to_string(),
                        ValueData::Value(Some(Value::UInt32(7))),
                    )
                ])),
            },
        },
        TestCase {
            input: vec![0xc0, 0x89, 0x86, 0xe0, 0x87, 0x85],
            result: TemplateData {
                name: "DynamicReference".to_string(),
                value: ValueData::Group(HashMap::from([
                    (
                        "PreRefData".to_string(),
                        ValueData::Value(Some(Value::UInt32(6))),
                    ), (
                        "templateRef:0".to_string(),
                        ValueData::DynamicTemplateRef(
                            Box::new(TemplateData {
                                name: "RefData".to_string(),
                                value: ValueData::Group(HashMap::from([
                                    (
                                        "TestData".to_string(),
                                        ValueData::Value(Some(Value::UInt32(5))),
                                    )
                                ])),
                            }),
                        ),
                    )
                ])),
            },
        },
    ]);
}
