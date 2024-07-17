use hashbrown::HashMap;

use crate::{Error, MessageFactory, Value};
use crate::base::message::MessageVisitor;
use crate::utils::stacked::Stacked;
use crate::Result;

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
            let rc = self.ref_num.must_peek_mut();
            self.context.push((format!("templateRef:{}", rc), tpl_ref));
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

/// # Model Visitor
/// Template model for serialization and message encoding.
pub struct ModelVisitor {
    data: TemplateData,

    /// Stores current context value.
    /// Here context value can be `ValueData::Group` or `ValueData::Sequence`.
    context: Stacked<ValueData>,

    // Indicates whether the current reference is dynamic.
    ref_dynamic: Stacked<bool>,

    // Counter for dynamic references.
    // Used to generate unique names for dynamic references within one context.
    ref_num: Stacked<u32>,
}

impl ModelVisitor {
    #[allow(unused)]
    pub fn new(data: TemplateData) -> Self {
        Self {
            data,
            context: Stacked::new_empty(),
            ref_dynamic: Stacked::new_empty(),
            ref_num: Stacked::new(0),
        }
    }
}

impl MessageVisitor for ModelVisitor {
    fn get_template_name(&mut self) -> Result<String> {
        match self.data.value {
            ValueData::Group(_) => {
                self.context.push(self.data.value.clone());
                Ok(self.data.name.clone())
            }
            _ => {
                Err(Error::Runtime(format!("Template {} data expected to be ValueData::Group, got {:?}", self.data.name, self.data.value)))
            }
        }
    }

    fn get_value(&mut self, name: &str) -> Result<Option<Value>> {
        match self.context.must_peek() {
            ValueData::Group(context) => {
                if let Some(v) = context.get(name) {
                    match v {
                        ValueData::Value(v) => {
                            Ok(v.clone())
                        }
                        _ => {
                            Err(Error::Runtime(format!("Field {name} expected to be ValueData::Value, got {:?}", v)))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            _ => unimplemented!(),
        }
    }

    fn select_group(&mut self, name: &str) -> Result<bool> {
        self.ref_num.push(0);
        match self.context.must_peek() {
            ValueData::Group(context) => {
                if let Some(v) = context.get(name) {
                    match v {
                        ValueData::None | ValueData::Value(None) => {
                            Ok(false)
                        }
                        ValueData::Group(_) => {
                            self.context.push(v.clone());
                            Ok(true)
                        }
                        _ => {
                            Err(Error::Runtime(format!("Field {name} expected to be ValueData::Group, got {:?}", v)))
                        }
                    }
                } else {
                    Ok(false)
                }
            }
            _ => unimplemented!(),
        }
    }

    fn release_group(&mut self) -> Result<()> {
        self.context.pop();
        self.ref_num.pop();
        Ok(())
    }

    fn select_sequence(&mut self, name: &str) -> Result<Option<usize>> {
        match self.context.must_peek() {
            ValueData::Group(context) => {
                if let Some(v) = context.get(name) {
                    match v {
                        ValueData::None | ValueData::Value(None) => {
                            Ok(None)
                        }
                        ValueData::Sequence(s) => {
                            let len  = s.len();
                            self.context.push(v.clone());
                            Ok(Some(len))
                        }
                        _ => {
                            Err(Error::Runtime(format!("Field {name} expected to be ValueData::Sequence, got: {:?}", v)))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            _ => unimplemented!(),
        }
    }

    fn select_sequence_item(&mut self, index: usize) -> Result<()> {
        self.ref_num.push(0);
        match self.context.must_peek() {
            ValueData::Sequence(sequence) => {
                if let Some(v) = sequence.get(index) {
                    match v {
                        ValueData::Group(_) => {
                            self.context.push(v.clone());
                            Ok(())
                        }
                        _ => {
                            Err(Error::Runtime(format!("Sequence item #{index} expected to be ValueData::Group, got {:?}", v)))
                        }
                    }
                } else {
                    Err(Error::Runtime(format!("Sequence item #{index} not found")))
                }
            }
            _ => unimplemented!(),
        }
    }

    fn release_sequence_item(&mut self) -> Result<()> {
        self.context.pop();
        self.ref_num.pop();
        Ok(())
    }

    fn release_sequence(&mut self) -> Result<()> {
        self.context.pop();
        Ok(())
    }

    fn select_template_ref(&mut self, _name: &str, dynamic: bool) -> Result<Option<String>> {
        self.ref_dynamic.push(dynamic);
        if dynamic {
            let rc = self.ref_num.must_peek_mut();
            let name = format!("templateRef:{}", rc);
            *rc += 1;
            self.ref_num.push(0);
            match self.context.must_peek() {
                ValueData::Group(context) => {
                    if let Some(v) = context.get(&name) {
                        match v {
                            ValueData::None => {
                                return Ok(None)
                            }
                            ValueData::DynamicTemplateRef(t) => {
                                match t.value {
                                    ValueData::Group(_) => {
                                        let template_name = t.name.clone();
                                        self.context.push(t.value.clone());
                                        Ok(Some(template_name))
                                    }
                                    _ => {
                                        Err(Error::Runtime(format!("Field {name} value expected to be ValueData::Group, got {:?}", t.value)))
                                    }}
                            }
                            _ => {
                                Err(Error::Runtime(format!("Field {name} expected to be ValueData::DynamicTemplateRef, got {:?}", v)))
                            }
                        }
                    } else {
                        Ok(None)
                    }
                }
                _ => unimplemented!(),
            }
        } else {
            self.ref_num.push(0);
            // do nothing because static template ref is embedded into current context
            Ok(None)
        }
    }

    fn release_template_ref(&mut self) -> Result<()> {
        self.ref_num.pop();
        if self.ref_dynamic.pop().unwrap() {
            self.context.pop();
        }
        Ok(())
    }
}
