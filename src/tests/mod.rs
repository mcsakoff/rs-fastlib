use rustc_hash::FxHashMap as HashMap;

use crate::Value;
use crate::base::instruction::Instruction;
use crate::base::message::MessageFactory;
use crate::base::types::{Dictionary, Operator, Presence};
use crate::base::value::ValueType;
use crate::decoder::decoder::Decoder;
use crate::encoder::encoder::Encoder;
use crate::model::template::TemplateData;
use crate::model::value::ValueData;
use crate::model::{ModelFactory, ModelVisitor};

mod base;
mod base_serde;
mod model;
mod spec;
mod spec2;

pub struct TestField {
    id: u32,
    name: &'static str,
    presence: Presence,
    operator: Operator,
    value: ValueType,
    instructions: Vec<TestField>,
    has_pmap: bool,
}

pub struct TestTemplate {
    id: u32,
    name: &'static str,
    dictionary: Dictionary,
    instructions: Vec<TestField>,
}

pub fn test_templates(d: &Decoder, tts: &Vec<TestTemplate>) {
    assert_eq!(
        d.definitions.templates.len(),
        tts.len(),
        "templates count mismatch"
    );
    for (t, tt) in d.definitions.templates.iter().zip(tts) {
        assert_eq!(t.id, tt.id, "{} id mismatch", t.name);
        assert_eq!(t.name, t.name, "{} name mismatch", t.name);
        assert_eq!(
            t.dictionary, tt.dictionary,
            "{} dictionary mismatch",
            t.name
        );
        test_instructions(&t.instructions, &tt.instructions, &tt.name);
    }
}

pub fn test_instructions(iss: &Vec<Instruction>, tis: &Vec<TestField>, name: &str) {
    assert_eq!(iss.len(), tis.len(), "{name} fields count mismatch");
    for (t, tt) in iss.iter().zip(tis) {
        assert_eq!(t.id, tt.id, "{} id mismatch", tt.name);
        assert_eq!(t.name, t.name, "{} name mismatch", tt.name);
        assert_eq!(t.presence, tt.presence, "{} presence mismatch", tt.name);
        assert_eq!(t.operator, tt.operator, "{} operator mismatch", tt.name);
        assert_eq!(t.value_type, tt.value, "{} value mismatch", tt.name);
        assert_eq!(
            t.has_pmap.get(),
            tt.has_pmap,
            "{} has_pmap mismatch",
            tt.name
        );
        test_instructions(&t.instructions, &tt.instructions, &tt.name);
    }
}

pub struct LoggingMessageFactory {
    pub calls: Vec<String>,
}

impl LoggingMessageFactory {
    pub fn new() -> Self {
        Self { calls: Vec::new() }
    }
}

impl MessageFactory for LoggingMessageFactory {
    fn start_template(&mut self, id: u32, name: &str) {
        self.calls.push(format!("start_template: {id}:{name}"));
    }

    fn stop_template(&mut self) {
        self.calls.push("stop_template".to_string());
    }

    fn set_value(&mut self, id: u32, name: &str, value: Option<Value>) {
        self.calls
            .push(format!("set_value: {id}:{name} {:?}", value));
    }

    fn start_sequence(&mut self, id: u32, name: &str, length: u32) {
        self.calls
            .push(format!("start_sequence: {id}:{name} {length}"));
    }

    fn start_sequence_item(&mut self, index: u32) {
        self.calls.push(format!("start_sequence_item: {index}"));
    }

    fn stop_sequence_item(&mut self) {
        self.calls.push("stop_sequence_item".to_string());
    }

    fn stop_sequence(&mut self) {
        self.calls.push("stop_sequence".to_string());
    }

    fn start_group(&mut self, name: &str) {
        self.calls.push(format!("start_group: {name}"));
    }

    fn stop_group(&mut self) {
        self.calls.push("stop_group".to_string());
    }

    fn start_template_ref(&mut self, name: &str, dynamic: bool) {
        self.calls
            .push(format!("start_template_ref: {name}:{dynamic}"));
    }

    fn stop_template_ref(&mut self) {
        self.calls.push("stop_template_ref".to_string());
    }
}

pub struct TestMsgValue<'a> {
    pub template: &'a str,
    pub value: Option<Value>,
}

pub struct TestCase<'a> {
    name: &'a str,
    raw: Vec<u8>,
    data: TestMsgValue<'a>,
}

pub struct TestCaseSeq<'a> {
    name: &'a str,
    raw: Vec<Vec<u8>>,
    data: Vec<TestMsgValue<'a>>,
}

fn extract_value(msg: ModelFactory, name: &str, test_name: &str) -> Option<Value> {
    match msg.data.unwrap().value {
        ValueData::Group(g) => match g.get(name).unwrap() {
            ValueData::Value(v) => v.clone(),
            _ => panic!("{} failed (Value expected)", test_name),
        },
        _ => panic!("{} failed (Group expected)", test_name),
    }
}

fn pack_value(templaet_name: &str, name: &str, value: Option<Value>) -> TemplateData {
    TemplateData {
        name: templaet_name.to_string(),
        value: ValueData::Group(HashMap::from_iter([(
            name.to_string(),
            ValueData::Value(value),
        )])),
    }
}

fn do_test(decode: bool, encode: bool, context: bool, definitions: &str, tt: TestCase) {
    let mut d = Decoder::new_from_xml(definitions).unwrap();
    let mut e = Encoder::new_from_xml(definitions).unwrap();
    if decode {
        test_decode(&mut d, &tt);
    }
    if encode {
        test_encode(&mut e, &tt);
    }
    if decode && encode && context {
        assert_eq!(d.context, e.context, "{} context mismatch", tt.name)
    }
}

fn do_test_seq(decode: bool, encode: bool, context: bool, definitions: &str, tt: TestCaseSeq) {
    let mut d = Decoder::new_from_xml(definitions).unwrap();
    let mut e = Encoder::new_from_xml(definitions).unwrap();
    for (i, (raw, data)) in tt.raw.into_iter().zip(tt.data).enumerate() {
        let name = format!("{} #{}", tt.name, i + 1);
        let tt = TestCase {
            name: &name,
            raw,
            data,
        };
        if decode {
            test_decode(&mut d, &tt);
        }
        if encode {
            test_encode(&mut e, &tt);
        }
        if decode && encode && context {
            assert_eq!(d.context, e.context, "{} context mismatch", tt.name)
        }
    }
}

fn test_decode(decoder: &mut Decoder, tt: &TestCase) {
    let mut msg = ModelFactory::new();
    decoder.decode_slice(&tt.raw, &mut msg).unwrap();
    let data = extract_value(msg, "Value", tt.name);
    assert_eq!(data, tt.data.value, "{} decode failed", tt.name);
}

fn test_encode(encoder: &mut Encoder, tt: &TestCase) {
    let mut msg = ModelVisitor::new(pack_value(tt.data.template, "Value", tt.data.value.clone()));
    let raw = encoder.encode_vec(&mut msg).unwrap();
    assert_eq!(raw, tt.raw, "{} encode failed", tt.name);
}
