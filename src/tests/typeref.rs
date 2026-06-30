use crate::Encoder;

const DEFINITION: &str = include_str!("templates/typeref.xml");

#[test]
fn template_with_type_ref() {
    Encoder::new_from_xml(DEFINITION).expect("expected typeRef to be parsed successfully");
}
