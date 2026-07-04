use super::*;

const DEFINITION: &str = include_str!("templates/typeref.xml");

#[test]
fn decode_encode_with_type_ref() {
    do_test_seq(
        true,
        true,
        true,
        DEFINITION,
        TestCaseSeq {
            name: "Decoding templates with typeRef and 'type' context",
            raw: vec![
                // Message1=<Value=data1>
                // Decoding the message sets the previous value in `type` context.
                vec![0xe0, 0x81, 0x64, 0x61, 0x74, 0x61, 0xb1],
                // Message2=<Value=data2>
                // Decoding sets message the previous value in global context.
                vec![0xe0, 0x82, 0x64, 0x61, 0x74, 0x61, 0xb2],
                // Message3=<Value=data1>
                // This message must use the previous value from the `type` context.
                vec![0xc0, 0x83],
            ],
            data: vec![
                TestMsgValue {
                    template: "Message1",
                    value: Some(Value::ASCIIString("data1".to_string())),
                },
                TestMsgValue {
                    template: "Message2",
                    value: Some(Value::ASCIIString("data2".to_string())),
                },
                TestMsgValue {
                    template: "Message3",
                    value: Some(Value::ASCIIString("data1".to_string())),
                },
            ],
        },
    );
}
