use fastlib::{Decoder, Encoder, TextMessageFactory, TextMessageVisitor};

fn do_tests_seq(templates: &str, raw: &[&[u8]], data: &[&str]) {
    let mut e = Encoder::new_from_xml(templates).unwrap();
    let mut d = Decoder::new_from_xml(templates).unwrap();

    for (i, (raw, data)) in raw.iter().zip(data).enumerate() {
        let mut msg = TextMessageFactory::new();
        d.decode_slice(raw, &mut msg).unwrap();
        assert_eq!(&msg.text, data, "decode failed #{}", i + 1);

        let mut msg = TextMessageVisitor::from_text(data).unwrap();
        let res = e.encode_vec(&mut msg).unwrap();
        assert_eq!(&res, raw, "encode failed #{}", i + 1);
    }
}

// After a decimal copy stores 1.5, a delta with the same key and components (0, 1) should produce 1.6.
//
// https://github.com/mcsakoff/rs-fastlib/issues/11
//
#[test]
fn decimal_delta_uses_shared_previous_value() {
    let xml = r#"
        <templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
            <template id="1" name="Copy">
                <decimal name="Value"><copy key="px"/></decimal>
            </template>
            <template id="2" name="Delta">
                <decimal name="Value"><delta key="px"/></decimal>
            </template>
        </templates>
    "#;

    do_tests_seq(
        xml,
        &[&[0xe0, 0x81, 0xff, 0x8f], &[0xc0, 0x82, 0x80, 0x81]],
        &["Copy=<Value=1.5>", "Delta=<Value=1.6>"],
    );
}
