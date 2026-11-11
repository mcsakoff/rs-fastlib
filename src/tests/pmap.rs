use std::fmt::Write;

use crate::Error;
use crate::model::template::TemplateData;
use crate::model::value::{Fields, ValueData};

use super::*;

// A template with `n` fields with copy operator, each taking a presence map bit, in a message segment,
// a group segment and a sequence segment.
fn gen_xml(n: usize) -> String {
    let fields: String = (1..=n).fold(String::new(), |mut output, i| {
        let _ = writeln!(
            output,
            r#"        <uInt32 name="Value{i:03}" id="{i}"><copy/></uInt32>"#
        );
        output
    });
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
    <template name="Message" id="100">
{fields}
    </template>
    <template name="Group" id="101">
        <group name="Group">
{fields}
        </group>
    </template>
    <template name="Sequence" id="102">
        <sequence name="Sequence">
{fields}
        </sequence>
    </template>
</templates>
"#
    )
}

fn gen_fields(n: usize, value: impl Fn(usize) -> u32) -> ValueData {
    ValueData::Group(Fields::from_iter((1..=n).map(|i| {
        (
            format!("Value{i:03}"),
            ValueData::Value(Some(Value::UInt32(value(i)))),
        )
    })))
}

fn gen_message(template: &str, n: usize, value: impl Fn(usize) -> u32) -> TemplateData {
    let fields = gen_fields(n, value);
    let value = match template {
        "Message" => fields,
        "Group" => ValueData::Group(Fields::from_iter([("Group".to_string(), fields)])),
        "Sequence" => ValueData::Group(Fields::from_iter([(
            "Sequence".to_string(),
            ValueData::Sequence(vec![fields]),
        )])),
        _ => unreachable!(),
    };
    TemplateData {
        name: template.to_string(),
        value,
    }
}

#[test]
fn long_presence_map() {
    for n in [62, 63, 64, 70, 126, 130] {
        let xml = gen_xml(n);
        let mut d = Decoder::new_from_xml(&xml).unwrap();
        let mut e = Encoder::new_from_xml(&xml).unwrap();
        for template in ["Message", "Group", "Sequence"] {
            // All bits set, no bits set but the template id, the last bit set.
            for msg in [
                gen_message(template, n, |i| i as u32),
                gen_message(template, n, |i| i as u32),
                gen_message(template, n, |i| if i == n { 0 } else { i as u32 }),
            ] {
                let raw = e.encode_vec(&mut ModelVisitor::new(msg.clone())).unwrap();
                let mut f = ModelFactory::new();
                d.decode_slice(&raw, &mut f).unwrap();
                assert_eq!(f.data.unwrap(), msg, "{template} {n}");
            }
        }
        assert_eq!(d.context, e.context, "{n}");
    }
}

#[test]
fn long_presence_map_bytes() {
    let xml = gen_xml(70);
    let mut e = Encoder::new_from_xml(&xml).unwrap();
    let msg = gen_message("Message", 70, |i| i as u32);
    e.encode_vec(&mut ModelVisitor::new(msg)).unwrap();
    // Only the last field changes: the template id bit and 69 bits are not set.
    let msg = gen_message("Message", 70, |i| if i == 70 { 0 } else { i as u32 });
    let raw = e.encode_vec(&mut ModelVisitor::new(msg)).unwrap();
    assert_eq!(raw, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0x80]);

    let mut d = Decoder::new_from_xml(&xml).unwrap();
    let msg = gen_message("Message", 70, |i| i as u32);
    let raw = Encoder::new_from_xml(&xml)
        .unwrap()
        .encode_vec(&mut ModelVisitor::new(msg))
        .unwrap();
    d.decode_slice(&raw, &mut ModelFactory::new()).unwrap();

    // [ERR R7] trailing zero group.
    let overlong = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x80, 0x80];
    assert!(matches!(
        d.decode_slice(&overlong, &mut ModelFactory::new()),
        Err(Error::Dynamic(m)) if m.contains("overlong presence map")
    ));
    // [ERR R8] a bit after the last field is set.
    let excess = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xe0, 0x80];
    assert!(matches!(
        d.decode_slice(&excess, &mut ModelFactory::new()),
        Err(Error::Dynamic(m)) if m.contains("more bits than required")
    ));
}
