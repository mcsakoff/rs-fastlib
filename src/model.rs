use hashbrown::HashMap;
use crate::{MessageFactory, Value};
use crate::utils::stacked::Stacked;

/// # Model Factory
/// Creates a template model that later can be deserialized using Serde.
#[derive(Debug, PartialEq)]
pub struct ModelFactory {
    pub data: Option<TemplateData>,

    /// Stores current context name and value.
    /// Here context value can be `ValueData::Group` or `ValueData::Sequence`.
    context: Stacked<(String, ValueData)>,

    // Reference number for dynamic references
    ref_num: u32,
}

impl ModelFactory {
    pub fn new() -> Self {
        Self {
            data: None,
            context: Stacked::new_empty(),
            ref_num: 0,
        }
    }
}

impl MessageFactory for ModelFactory {
    fn start_template(&mut self, _id: u32, name: &str) {
        self.context.push((
            name.to_string(),
            ValueData::Group(HashMap::new())
        ));
    }

    fn stop_template(&mut self) {
        let (name, value) = self.context.pop().unwrap();
        self.data = Some(TemplateData{ name, value })
    }

    fn set_value(&mut self, _id: u32, name: &str, value: Option<Value>) {
        let (_, context) = self.context.must_peek_mut();
        match context {
            ValueData::Group(group) => {
                group.insert(name.to_string(), ValueData::Value(value));
            }
            _ => unreachable!(),
        }
    }

    fn start_sequence(&mut self, _id: u32, name: &str, length: u32) {
        self.context.push((
            name.to_string(),
            ValueData::Sequence(Vec::with_capacity(length as usize))
        ));
    }

    fn start_sequence_item(&mut self, _index: u32) {
        self.context.push((
            String::new(),
            ValueData::Group(HashMap::new())
        ));
    }

    fn stop_sequence_item(&mut self) {
        let (_, g) = self.context.pop().unwrap();
        let (_, context) = self.context.must_peek_mut();
        match context {
            ValueData::Sequence(seq) => {
                seq.push(g);
            }
            _ => unreachable!(),
        }
    }

    fn stop_sequence(&mut self) {
        let (n, s) = self.context.pop().unwrap();
        let (_, context) = self.context.must_peek_mut();
        match context {
            ValueData::Group(group) => {
                group.insert(n, s);
            }
            _ => unreachable!(),
        }
    }

    fn start_group(&mut self, name: &str) {
        self.context.push((
            name.to_string(),
            ValueData::Group(HashMap::new())
        ));
    }

    fn stop_group(&mut self) {
        let (n, g) = self.context.pop().unwrap();
        let (_, context) = self.context.must_peek_mut();
        match context {
            ValueData::Group(group) => {
                group.insert(n, g);
            }
            _ => unreachable!(),
        }
    }

    fn start_template_ref(&mut self, name: &str, dynamic: bool) {
        if dynamic {
            let tpl_ref = ValueData::DynamicTemplateRef(name.to_string(), Box::new(ValueData::Null));
            self.context.push((format!("DR:{}", self.ref_num), tpl_ref));
        } else {
            let tpl_ref = ValueData::StaticTemplateRef(name.to_string(), Box::new(ValueData::Null));
            self.context.push((format!("SR:{name}"), tpl_ref));
        };
        self.context.push((String::new(), ValueData::Group(HashMap::new())));
    }

    fn stop_template_ref(&mut self) {
        let (_, g) = self.context.pop().unwrap();
        let (n, r) = self.context.pop().unwrap();
        let (_, context) = self.context.must_peek_mut();
        match context {
            ValueData::Group(group) => {
                match r {
                    ValueData::StaticTemplateRef(m, _) => {
                        group.insert(n, ValueData::StaticTemplateRef(m, Box::new(g)));
                    }
                    ValueData::DynamicTemplateRef(m, _) => {
                        group.insert(n, ValueData::DynamicTemplateRef(m, Box::new(g)));
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TemplateData {
    pub name: String,
    pub value: ValueData, // Always Value::Group
}

#[derive(Debug, PartialEq)]
pub enum ValueData {
    Null,
    Value(Option<Value>),
    Group(HashMap<String, ValueData>),
    Sequence(Vec<ValueData>), // Always Vec of Value::Group
    StaticTemplateRef(String, Box<ValueData>),  // Always Value::Group
    DynamicTemplateRef(String, Box<ValueData>),  // Always Value::Group
}
