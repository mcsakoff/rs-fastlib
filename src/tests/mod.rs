use crate::base::types::Dictionary;
use crate::base::instruction::Instruction;
use crate::base::message::MessageFactory;
use crate::base::types::Operator;
use crate::base::types::Presence;
use crate::base::value::ValueType;
use crate::decoder::decoder::Decoder;
use crate::Value;

mod base;
mod base_serde;
mod spec;
mod model;

struct TestField {
    id: u32,
    name: &'static str,
    presence: Presence,
    operator: Operator,
    value: ValueType,
    instructions: Vec<TestField>,
    has_pmap: bool,
}
struct TestTemplate {
    id: u32,
    name: &'static str,
    dictionary: Dictionary,
    instructions: Vec<TestField>,
}

fn test_templates(d: &Decoder, tts: &Vec<TestTemplate>) {
    assert_eq!(d.definitions.templates.len(), tts.len(), "templates count mismatch");
    for (t, tt) in d.definitions.templates.iter().zip(tts) {
        assert_eq!(t.id, tt.id, "{} id mismatch", t.name);
        assert_eq!(t.name, t.name, "{} name mismatch", t.name);
        assert_eq!(t.dictionary, tt.dictionary, "{} dictionary mismatch", t.name);
        test_instructions(&t.instructions, &tt.instructions, &tt.name);
    }
}

fn test_instructions(iss: &Vec<Instruction>, tis: &Vec<TestField>, name: &str) {
    assert_eq!(iss.len(), tis.len(), "{name} fields count mismatch");
    for (t, tt) in iss.iter().zip(tis) {
        assert_eq!(t.id, tt.id, "{} id mismatch", tt.name);
        assert_eq!(t.name, t.name, "{} name mismatch", tt.name);
        assert_eq!(t.presence, tt.presence, "{} presence mismatch", tt.name);
        assert_eq!(t.operator, tt.operator, "{} operator mismatch", tt.name);
        assert_eq!(t.value_type, tt.value, "{} value mismatch", tt.name);
        assert_eq!(t.has_pmap.get(), tt.has_pmap, "{} has_pmap mismatch", tt.name);
        test_instructions(&t.instructions, &tt.instructions, &tt.name);
    }
}

pub struct LoggingMessageFactory {
    pub calls: Vec<String>,
}

impl LoggingMessageFactory {
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
        }
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
        self.calls.push(format!("set_value: {id}:{name} {:?}", value));
    }

    fn start_sequence(&mut self, id: u32, name: &str, length: u32) {
        self.calls.push(format!("start_sequence: {id}:{name} {length}"));
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
        self.calls.push(format!("start_template_ref: {name}:{dynamic}"));
    }

    fn stop_template_ref(&mut self) {
        self.calls.push("stop_template_ref".to_string());
    }
}
