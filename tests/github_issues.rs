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
// https://github.com/mcsakoff/rs-fastlib/issues/11
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

// A sequence length inherits optional presence (§6.2.5).
// https://github.com/mcsakoff/rs-fastlib/issues/12
#[test]
fn optional_sequence_length_accepts_default_without_value() {
    let xml = r#"
        <templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
            <template id="1" name="Message">
                <uInt32 name="Id"/>
                <sequence name="Items" presence="optional">
                    <length><default/></length>
                    <uInt32 name="X"/>
                </sequence>
            </template>
        </templates>
    "#;

    // The absent length is the (absent) initial value, so it takes no bytes in the stream.
    do_tests_seq(
        xml,
        &[&[0xc0, 0x81, 0x81], &[0xa0, 0x82, 0x82, 0x85]],
        &["Message=<Id=1>", "Message=<Id=2|Items=<X=5>>"],
    );
}

// A decimal exponent inherits optional presence (§10.5.1).
// https://github.com/mcsakoff/rs-fastlib/issues/12
#[test]
fn optional_decimal_exponent_accepts_default_without_value() {
    let xml = r#"
        <templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
            <template id="1" name="Message">
                <uInt32 name="Id"/>
                <decimal name="Price" presence="optional">
                    <exponent><default/></exponent>
                </decimal>
            </template>
        </templates>
    "#;

    // The absent exponent is the (absent) initial value, so the decimal takes no bytes in the stream.
    do_tests_seq(
        xml,
        &[&[0xc0, 0x81, 0x81], &[0xa0, 0x82, 0xff, 0x8f]],
        &["Message=<Id=1>", "Message=<Id=2|Price=1.5>"],
    );
}
