use hashbrown::HashMap;

use crate::{MessageFactory, Value};
use crate::utils::stacked::Stacked;

use self::template::TemplateData;
use self::value::ValueData;

pub(crate) mod template;
pub(crate) mod value;
mod decimal;

/// # Model Factory
/// Creates a template model that later can be deserialized using Serde.
#[derive(Debug, PartialEq)]
pub struct ModelFactory {
    pub data: Option<TemplateData>,

    /// Stores current context name and value.
    /// Here context value can be `ValueData::Group` or `ValueData::Sequence`.
    context: Stacked<(String, ValueData)>,

    // Counter for dynamic references.
    // Used to generate unique names for dynamic references within one context.
    ref_num: Stacked<u32>,
}

impl ModelFactory {
    pub fn new() -> Self {
        Self {
            data: None,
            context: Stacked::new_empty(),
            ref_num: Stacked::new(0),
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
        self.data = Some(TemplateData { name, value })
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
        self.ref_num.push(0);
    }

    fn stop_sequence_item(&mut self) {
        _ = self.ref_num.pop();
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
        self.ref_num.push(0);
    }

    fn stop_group(&mut self) {
        _ = self.ref_num.pop();
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
            let tpl_ref = ValueData::DynamicTemplateRef(Box::new(TemplateData{
                name: name.to_string(),
                value: ValueData::None,
            }));
            self.context.push((format!("templateRef:{}", self.ref_num.must_peek()), tpl_ref));
            let rc = self.ref_num.must_peek_mut();
            *rc += 1;
        } else {
            let tpl_ref = ValueData::StaticTemplateRef(name.to_string(), Box::new(ValueData::None));
            self.context.push((name.to_string(), tpl_ref));
        };
        self.context.push((String::new(), ValueData::Group(HashMap::new())));
        self.ref_num.push(0);
    }

    fn stop_template_ref(&mut self) {
        _ = self.ref_num.pop();
        let (_, vg) = self.context.pop().unwrap();
        let (n, vr) = self.context.pop().unwrap();
        let (_, context) = self.context.must_peek_mut();
        match context {
            ValueData::Group(group) => {
                match vr {
                    ValueData::StaticTemplateRef(_m, _) => {
                        // Instead of inserting the group as standalone object, we inline the group into parent's context:
                        if let ValueData::Group(g) = vg {
                            group.extend(g)
                        } else {
                            unreachable!()
                        }
                    }
                    ValueData::DynamicTemplateRef(mut t) => {
                        t.value = vg;
                        group.insert(n, ValueData::DynamicTemplateRef(t));
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
