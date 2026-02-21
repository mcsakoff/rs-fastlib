use std::fmt::Write;

use hashbrown::HashMap;

use crate::Result;
use crate::utils::bytes::bytes_to_string;
use crate::utils::stacked::Stacked;
use crate::{Error, MessageFactory, MessageVisitor, Value, ValueType};

/// Message factory implementation that formats decoded messages as a human-readable text.
pub struct TextMessageFactory {
    pub text: String,
    block_start: bool,
    dynamic: Vec<bool>,
}

impl TextMessageFactory {
    /// Creates a new message factory.
    #[must_use]
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
        if self.block_start {
            self.block_start = false;
        } else {
            self.text += "|";
        }
    }
}

impl Default for TextMessageFactory {
    fn default() -> Self {
        Self::new()
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
                Value::ASCIIString(v) | Value::UnicodeString(v) => v.clone(),
                Value::Bytes(b) => bytes_to_string(&b),
            };
            let _ = write!(&mut self.text, "{name}={value}");
        }
    }

    fn start_sequence(&mut self, _id: u32, name: &str, _length: u32) {
        self.delimiter();
        let _ = write!(&mut self.text, "{name}=");
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
        let _ = write!(&mut self.text, "{name}=<");
        self.block_start = true;
    }

    fn stop_group(&mut self) {
        self.text += ">";
        self.block_start = false;
    }

    fn start_template_ref(&mut self, name: &str, dynamic: bool) {
        self.dynamic.push(dynamic);
        if dynamic {
            self.delimiter();

            let _ = write!(&mut self.text, "TemplateReference=<{name}=<");
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
    #[must_use]
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
        if self.block_start {
            self.block_start = false;
        } else {
            self.json += ",";
        }
    }
}

impl Default for JsonMessageFactory {
    fn default() -> Self {
        Self::new()
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
                Value::ASCIIString(v) | Value::UnicodeString(v) => format!("\"{v}\""),
                Value::Bytes(b) => bytes_to_string(&b),
            };
            let _ = write!(&mut self.json, "\"{name}\":{value}");
        }
    }

    fn start_sequence(&mut self, _id: u32, name: &str, _length: u32) {
        self.delimiter();
        let _ = write!(&mut self.json, "\"{name}\":[");
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

        let _ = write!(&mut self.json, "\"{name}\":{{");
        self.block_start = true;
    }

    fn stop_group(&mut self) {
        self.json += "}";
        self.block_start = false;
    }

    fn start_template_ref(&mut self, name: &str, dynamic: bool) {
        self.dynamic.push(dynamic);
        if dynamic {
            self.delimiter();

            let _ = write!(&mut self.json, "\"TemplateReference\":{{\"{name}\":{{");
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

pub struct TextMessageVisitor {
    pub data: TextMessageValue,
    context: Stacked<*const TextMessageValue>,
    group_as_seq: Stacked<bool>,
}

impl TextMessageVisitor {
    /// Creates `TextMessageVisitor` from text
    ///
    /// # Errors
    /// Returns error if text can't be parsed.
    pub fn from_text(text: &str) -> Result<Self> {
        Ok(Self {
            data: TextMessageValue::from_text(text)?,
            context: Stacked::new_empty(),
            group_as_seq: Stacked::new_empty(),
        })
    }
}

impl MessageVisitor for TextMessageVisitor {
    fn get_template_name(&mut self) -> Result<String> {
        match &self.data {
            TextMessageValue::Group(h) => match h.iter().next() {
                None => Err(Error::Runtime("Template name not fount".to_string())),
                Some((name, value)) => {
                    self.context.push(value);
                    Ok(name.clone())
                }
            },
            _ => Err(Error::Runtime(format!(
                "Template value expected to be TextMessageValue::Group, got {:?}",
                self.data
            ))),
        }
    }

    fn get_value(&mut self, name: &str, type_: &ValueType) -> Result<Option<Value>> {
        // SAFETY: the reference to context is always valid because we never modify `self.data`
        let ctx = unsafe { self.context.must_peek().as_ref().unwrap() };
        match ctx {
            TextMessageValue::Group(context) => {
                if let Some(v) = context.get(name) {
                    match v {
                        TextMessageValue::Value(s) => {
                            let mut value = type_.to_default_value()?;
                            value.set_from_string(s)?;
                            Ok(Some(value))
                        }
                        _ => Err(Error::Runtime(format!(
                            "Field {name} expected to be TextMessageValue::Value, got {v:?}"
                        ))),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => unreachable!(),
        }
    }

    fn select_group(&mut self, name: &str) -> Result<bool> {
        // SAFETY: the reference to context is always valid because we never modify `self.data`
        let ctx = unsafe { self.context.must_peek().as_ref().unwrap() };
        match ctx {
            TextMessageValue::Group(context) => {
                if let Some(v) = context.get(name) {
                    match v {
                        TextMessageValue::Group(_) => {
                            self.context.push(v);
                            Ok(true)
                        }
                        _ => Err(Error::Runtime(format!(
                            "Field {name} expected to be TextMessageValue::Group, got {v:?}"
                        ))),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => unreachable!(),
        }
    }

    fn release_group(&mut self) -> Result<()> {
        self.context.pop();
        Ok(())
    }

    fn select_sequence(&mut self, name: &str) -> Result<Option<usize>> {
        // SAFETY: the reference to context is always valid because we never modify `self.data`
        let ctx = unsafe { self.context.must_peek().as_ref().unwrap() };
        match ctx {
            TextMessageValue::Group(context) => {
                if let Some(v) = context.get(name) {
                    match v {
                        TextMessageValue::Group(_) => {
                            self.group_as_seq.push(true);
                            self.context.push(v);
                            Ok(Some(1))
                        }
                        TextMessageValue::Sequence(s) => {
                            let len = s.len();
                            self.group_as_seq.push(false);
                            self.context.push(v);
                            Ok(Some(len))
                        }
                        TextMessageValue::Value(_) => Err(Error::Runtime(format!(
                            "Field {name} expected to be TextMessageValue::Sequence, got {v:?}"
                        ))),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => unreachable!(),
        }
    }

    fn select_sequence_item(&mut self, index: usize) -> Result<()> {
        if *self.group_as_seq.must_peek() {
            // context already set to proper group element
            return if index == 0 {
                Ok(())
            } else {
                Err(Error::Runtime("Index is out of range".to_string()))
            };
        }
        // SAFETY: the reference to context is always valid because we never modify `self.data`
        let ctx = unsafe { self.context.must_peek().as_ref().unwrap() };
        match ctx {
            TextMessageValue::Sequence(sequence) => {
                if let Some(v) = sequence.get(index) {
                    match v {
                        TextMessageValue::Group(_) => {
                            self.context.push(v);
                            Ok(())
                        }
                        _ => Err(Error::Runtime(format!(
                            "Sequence item #{index} expected to be TextMessageValue::Group, got {v:?}"
                        ))),
                    }
                } else {
                    Err(Error::Runtime(format!("Index {index} is out of range")))
                }
            }
            _ => unreachable!(),
        }
    }

    fn release_sequence_item(&mut self) -> Result<()> {
        self.context.pop();
        Ok(())
    }

    fn release_sequence(&mut self) -> Result<()> {
        if !self.group_as_seq.pop().unwrap() {
            self.context.pop();
        }
        Ok(())
    }

    fn select_template_ref(&mut self, _name: &str, dynamic: bool) -> Result<Option<String>> {
        if dynamic {
            todo!()
        } else {
            // do nothing because static template ref is embedded into current context
            Ok(None)
        }
    }

    fn release_template_ref(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum TextMessageValue {
    Value(String),
    Group(HashMap<String, TextMessageValue>),
    Sequence(Vec<TextMessageValue>),
}

impl TextMessageValue {
    fn from_text(text: &str) -> Result<Self> {
        match TextMessageValue::parse_next(text)? {
            (name, TextMessageValue::Group(value), size) if size == text.len() => Ok(
                TextMessageValue::Group(HashMap::from([(name, TextMessageValue::Group(value))])),
            ),
            (_, TextMessageValue::Group(_), _) => Err(Error::Dynamic(
                "Symbols left in buffer after parsing text message".to_string(),
            )),
            _ => Err(Error::Dynamic("Failed to parse message body".to_string())),
        }
    }

    fn parse_next(text: &str) -> Result<(String, Self, usize)> {
        let (name, rest) = text
            .split_once('=')
            .ok_or_else(|| Error::Dynamic("Failed to parse next field".to_string()))?;
        let (value, sz) = TextMessageValue::parse_value(rest)?;
        let size = name.len() + sz + 1;
        Ok((name.to_string(), value, size))
    }

    fn parse_value(mut text: &str) -> Result<(Self, usize)> {
        if text.starts_with('<') {
            // value is Group or Sequence
            let mut size = 0;
            let mut value: Option<TextMessageValue> = None;
            loop {
                let (v, s) = TextMessageValue::parse_group(&text[1..])?;
                size += s + 1;
                text = &text[s + 1..];
                match &mut value {
                    None => {
                        value = Some(v);
                    }
                    Some(TextMessageValue::Group(_)) => {
                        let seq = vec![value.unwrap(), v];
                        value = Some(TextMessageValue::Sequence(seq));
                    }
                    Some(TextMessageValue::Sequence(s)) => {
                        s.push(v);
                    }
                    _ => unreachable!(),
                }
                if text.starts_with('<') {
                    continue;
                }
                break;
            }
            Ok((value.unwrap(), size))
        } else {
            let i = text.find(['|', '>']).ok_or_else(|| {
                Error::Dynamic("Failed to parse next value (no delimiter)".to_string())
            })?;
            Ok((TextMessageValue::Value(text[..i].to_string()), i))
        }
    }

    fn parse_group(mut text: &str) -> Result<(Self, usize)> {
        let mut size = 0;
        let mut value: HashMap<String, TextMessageValue> = HashMap::new();
        loop {
            let (name, v, sz) = TextMessageValue::parse_next(text)?;
            size += sz;
            text = &text[sz..];
            value.insert(name, v);

            if text.starts_with('|') {
                size += 1;
                text = &text[1..];
                continue;
            }
            if text.starts_with('>') {
                size += 1;
                break;
            }
            return Err(Error::Dynamic("Failed to parse group".to_string()));
        }
        Ok((TextMessageValue::Group(value), size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_group() {
        let (value, size) = TextMessageValue::parse_group("MessageType=0|ApplVerID=8>|").unwrap();
        assert_eq!(
            value,
            TextMessageValue::Group(HashMap::from([
                (
                    "MessageType".to_string(),
                    TextMessageValue::Value("0".to_string())
                ),
                (
                    "ApplVerID".to_string(),
                    TextMessageValue::Value("8".to_string())
                ),
            ]))
        );
        assert_eq!(size, 26);
    }

    #[test]
    fn test_parse_text_value() {
        let (value, size) = TextMessageValue::parse_value("CQG|").unwrap();
        assert_eq!(value, TextMessageValue::Value("CQG".to_string()));
        assert_eq!(size, 3);

        let (value, size) = TextMessageValue::parse_value("<MessageType=0>|").unwrap();
        assert_eq!(
            value,
            TextMessageValue::Group(HashMap::from([(
                "MessageType".to_string(),
                TextMessageValue::Value("0".to_string())
            )]))
        );
        assert_eq!(size, 15);

        let (value, size) =
            TextMessageValue::parse_value("<MessageType=0><MessageType=1>|").unwrap();
        assert_eq!(
            value,
            TextMessageValue::Sequence(vec![
                TextMessageValue::Group(HashMap::from([(
                    "MessageType".to_string(),
                    TextMessageValue::Value("0".to_string())
                )])),
                TextMessageValue::Group(HashMap::from([(
                    "MessageType".to_string(),
                    TextMessageValue::Value("1".to_string())
                )])),
            ])
        );
        assert_eq!(size, 30);
    }

    #[test]
    fn test_parse_test_message() {
        let m = TextMessageValue::from_text(
            "MDHeartbeat=<MessageType=0|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=2286|SendingTime=20240712171046052>"
        ).unwrap();
        let r = TextMessageValue::Group(HashMap::from([(
            "MDHeartbeat".to_string(),
            TextMessageValue::Group(HashMap::from([
                (
                    "MessageType".to_string(),
                    TextMessageValue::Value("0".to_string()),
                ),
                (
                    "ApplVerID".to_string(),
                    TextMessageValue::Value("8".to_string()),
                ),
                (
                    "SenderCompID".to_string(),
                    TextMessageValue::Value("CQG".to_string()),
                ),
                (
                    "MsgSeqNum".to_string(),
                    TextMessageValue::Value("2286".to_string()),
                ),
                (
                    "SendingTime".to_string(),
                    TextMessageValue::Value("20240712171046052".to_string()),
                ),
            ])),
        )]));
        assert_eq!(m, r);
    }
}
