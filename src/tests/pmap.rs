use std::fmt::Write;

use crate::Error::Static;

use super::*;

fn gen_xml(n: usize) -> String {
    let fields: String = (1..=n).fold(String::new(), |mut output, i| {
        let _ = writeln!(
            output,
            r#"        <uInt32 name="Value{i:02}" id="{i}"><copy/></uInt32>"#
        );
        output
    });
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
    <template name="Message1" id="100">
{fields}
    </template>
</templates>
"#
    )
}

#[test]
fn presence_map_size_limit() {
    // 62 fields + template id = 63 bits
    assert!(Decoder::new_from_xml(&gen_xml(62)).is_ok());

    // 63 fields + template id = 64 bits
    assert!(matches!(
        Decoder::new_from_xml(&gen_xml(63)),
        Err(Static(_))
    ));
}
