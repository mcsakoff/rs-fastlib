use crate::{MessageFactory, Value};
use crate::utils::bytes::bytes_to_string;

/// Message factory implementation that formats decoded messages as a human-readable text.
pub struct TextMessageFactory {
    pub text: String,
    block_start: bool,
    dynamic: Vec<bool>,
}

impl TextMessageFactory {
    /// Creates a new message factory.
    pub fn new() -> Self {
        Self {
            text: String::with_capacity(4096),
            block_start: false,
            dynamic: Vec::new(),
        }
    }

    /// Resets the state of the message factory.
    /// Called everytime a new message decoding started.
    pub fn reset(&mut self) {
        self.text.clear();
        self.block_start = false;
        self.dynamic.clear();
    }

    fn delimiter(&mut self) {
        if !self.block_start {
            self.text += "|";
        } else {
            self.block_start = false;
        }
    }
}

impl MessageFactory for TextMessageFactory {
    fn start_template(&mut self, _id: u32, name: &str) {
        self.reset();
        self.text = format!("{name}=<");
        self.block_start = true;
    }

    fn stop_template(&mut self) {
        self.text += ">";
    }

    fn set_value(&mut self, _id: u32, name: &str, value: Option<Value>) {
        if let Some(value) = value {
            self.delimiter();
            let value = match value {
                Value::UInt32(v) => format!("{v}"),
                Value::Int32(v) => format!("{v}"),
                Value::UInt64(v) => format!("{v}"),
                Value::Int64(v) => format!("{v}"),
                Value::Decimal(v) => v.to_string(),
                Value::ASCIIString(v) => v.clone(),
                Value::UnicodeString(v) => v.clone(),
                Value::Bytes(b) => bytes_to_string(&b),
            };
            self.text += &format!("{name}={value}");
        }
    }

    fn start_sequence(&mut self, _id: u32, name: &str, _length: u32) {
        self.delimiter();
        self.text += &format!("{name}=");
    }

    fn start_sequence_item(&mut self, _index: u32) {
        self.text += "<";
        self.block_start = true;
    }

    fn stop_sequence_item(&mut self) {
        self.text += ">";
    }

    fn stop_sequence(&mut self) {
        self.block_start = false;
    }

    fn start_group(&mut self, name: &str) {
        self.delimiter();
        self.text += &format!("{name}=<");
        self.block_start = true;
    }

    fn stop_group(&mut self) {
        self.text += ">";
        self.block_start = false
    }

    fn start_template_ref(&mut self, name: &str, dynamic: bool) {
        self.dynamic.push(dynamic);
        if dynamic {
            self.delimiter();
            self.text += &format!("TemplateReference=<{name}=<");
            self.block_start = true;
        }
    }

    fn stop_template_ref(&mut self) {
        let dynamic = self.dynamic.pop().unwrap();
        if dynamic {
            self.text += ">>";
        }
    }
}


/// Message factory implementation that formats decoded messages as JSON encoded `String`.
pub struct JsonMessageFactory {
    pub json: String,
    block_start: bool,
    dynamic: Vec<bool>,
}

impl JsonMessageFactory {
    /// Creates a new message factory.
    pub fn new() -> Self {
        Self {
            json: String::with_capacity(4096),
            block_start: false,
            dynamic: Vec::new(),
        }
    }

    /// Resets the state of the message factory.
    /// Called every time a new message decoding started.
    pub fn reset(&mut self) {
        self.json.clear();
        self.block_start = false;
        self.dynamic.clear();
    }

    fn delimiter(&mut self) {
        if !self.block_start {
            self.json += ",";
        } else {
            self.block_start = false;
        }
    }
}

impl MessageFactory for JsonMessageFactory {
    fn start_template(&mut self, _id: u32, name: &str) {
        self.reset();
        self.json = format!("{{\"{name}\":{{");
        self.block_start = true;
    }

    fn stop_template(&mut self) {
        self.json += "}}";
    }

    fn set_value(&mut self, _id: u32, name: &str, value: Option<Value>) {
        if let Some(value) = value {
            self.delimiter();
            let value = match value {
                Value::UInt32(v) => format!("{v}"),
                Value::Int32(v) => format!("{v}"),
                Value::UInt64(v) => format!("{v}"),
                Value::Int64(v) => format!("{v}"),
                Value::Decimal(v) => format!("{v}"),
                Value::ASCIIString(v) => format!("\"{v}\""),
                Value::UnicodeString(v) => format!("\"{v}\""),
                Value::Bytes(b) => bytes_to_string(&b),
            };
            self.json += &format!("\"{name}\":{value}");
        }
    }

    fn start_sequence(&mut self, _id: u32, name: &str, _length: u32) {
        self.delimiter();
        self.json += &format!("\"{name}\":[");
        self.block_start = true;
    }

    fn start_sequence_item(&mut self, _index: u32) {
        self.delimiter();
        self.json += "{";
        self.block_start = true;
    }

    fn stop_sequence_item(&mut self) {
        self.json += "}";
    }

    fn stop_sequence(&mut self) {
        self.json += "]";
        self.block_start = false;
    }

    fn start_group(&mut self, name: &str) {
        self.delimiter();
        self.json += &format!("\"{name}\":{{");
        self.block_start = true;
    }

    fn stop_group(&mut self) {
        self.json += "}";
        self.block_start = false
    }

    fn start_template_ref(&mut self, name: &str, dynamic: bool) {
        self.dynamic.push(dynamic);
        if dynamic {
            self.delimiter();
            self.json += &format!("\"TemplateReference\":{{\"{name}\":{{");
            self.block_start = true;
        }
    }

    fn stop_template_ref(&mut self) {
        let dynamic = self.dynamic.pop().unwrap();
        if dynamic {
            self.json += "}}";
        }
    }
}
