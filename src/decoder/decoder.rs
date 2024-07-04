use std::cell::Cell;
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;

use crate::{Error, Result};
use crate::base::instruction::Instruction;
use crate::base::message::MessageFactory;
use crate::base::types::{Dictionary, Operator, Presence, Template, TypeRef};
use crate::base::value::ValueType;
use crate::decoder::{context::DecoderContext, reader::Reader, state::DecoderState};
use crate::decoder::reader::StreamReader;

/// Decoder for FAST protocol messages.
pub struct Decoder {
    pub(crate) context: DecoderContext,
    pub(crate) templates: Vec<Rc<Template>>,
    pub(crate) templates_by_id: HashMap<u32, Rc<Template>>,
    pub(crate) templates_by_name: HashMap<String, Rc<Template>>,
    pub(crate) template_id_instruction: Rc<Instruction>,
}

impl Decoder {
    pub(crate) fn new_from_templates(ts: Vec<Template>) -> Result<Self> {
        let mut templates = Vec::with_capacity(ts.len());
        let mut templates_by_id = HashMap::with_capacity(ts.len());
        let mut templates_by_name = HashMap::with_capacity(ts.len());
        for t in ts {
            let t = Rc::new(t);
            if t.id != 0 {
                templates_by_id.insert(t.id, t.clone());
            }
            if !t.name.is_empty() {
                templates_by_name.insert(t.name.clone(), t.clone());
            }
            templates.push(t);
        }

        let template_id_instruction = Rc::new(Instruction {
            id: 0,
            name: "__template_id__".to_string(),
            value_type: ValueType::UInt32,
            presence: Presence::Mandatory,
            operator: Operator::Copy,
            initial_value: None,
            instructions: Vec::new(),
            dictionary: Dictionary::Global,
            key: Rc::new("__template_id__".to_string()),
            type_ref: TypeRef::Any,
            has_pmap: Cell::new(false),
        });

        let decoder = Self {
            context: DecoderContext::new(),
            templates,
            templates_by_id,
            templates_by_name,
            template_id_instruction,
        };
        decoder.finalize()?;
        Ok(decoder)
    }

    pub fn new_from_xml(text: &str) -> Result<Self> {
        let doc = roxmltree::Document::parse(text)?;
        let root = doc
            .root()
            .first_child()
            .ok_or_else(|| Error::Static("no root element found".to_string()))?;
        if root.tag_name().name() != "templates" {
            return Err(Error::Static("<templates/> node not found".to_string()));
        }
        let mut templates = Vec::new();
        for child in root.children() {
            if child.is_element() {
                templates.push(Template::from_node(child)?);
            }
        }
        Self::new_from_templates(templates)
    }

    pub fn reset(&mut self) {
        self.context.reset();
    }

    /// Decode single message from bytes vector.
    /// The `bytes` vector must be the whole message. It is an error if any bytes left after the message is decoded.
    pub fn decode_vec(&mut self, bytes: Vec<u8>, msg: &mut impl MessageFactory) -> Result<()> {
        let mut raw = bytes::Bytes::from(bytes);
        self.decode_reader(&mut raw, msg)?;
        if !raw.is_empty() {
            return Err(Error::Runtime(format!("Bytes left in the buffer after decoding: {}", raw.len())));
        }
        Ok(())
    }

    /// Decode single message from `bytes::Bytes`.
    pub fn decode_bytes(&mut self, bytes: &mut bytes::Bytes, msg: &mut impl MessageFactory) -> Result<()> {
        self.decode_reader(bytes, msg)
    }

    /// Decode single message from object that implements [`fastlib::Reader`][crate::decoder::reader::Reader] trait.
    pub fn decode_reader(&mut self, rdr: &mut impl Reader, msg: &mut impl MessageFactory) -> Result<()> {
        DecoderState::new(self, rdr, msg).decode_template()
    }

    /// Decode single message from object that implements [`std::io::Read`][std::io::Read] trait.
    pub fn decode_stream(&mut self, rdr: &mut dyn Read, msg: &mut impl MessageFactory) -> Result<()> {
        let mut rdr = StreamReader::new(rdr);
        self.decode_reader(&mut rdr, msg)
    }

    // After generating the templates we have to go through all the instructions and set flags
    // for structures that must have a presence map. That can only be done when whole
    // templates structure is generated.
    fn finalize(&self) -> Result<()> {
        for tpl in &self.templates {
            let need_pmap = self.require_presence_map_bit(&tpl.instructions)?;
            tpl.require_pmap.set(Some(need_pmap));
        }
        Ok(())
    }

    // Go through sequence of instructions and check if any of them require presence map bit.
    // No early exit! Must iterate over all items because has_presence_map_bit() also initializes has_pmap bit.
    fn require_presence_map_bit(&self, instructions: &[Instruction]) -> Result<bool> {
        let mut has_pmap_bit = false;
        for i in instructions {
            if self.has_presence_map_bit(i)? {
                has_pmap_bit = true;
            }
        }
        Ok(has_pmap_bit)
    }

    fn set_has_pmap(&self, instr: &Instruction) -> Result<()> {
        let instructions: &[Instruction];
        match instr.value_type {
            ValueType::Group | ValueType::TemplateReference | ValueType::Decimal => {
                instructions = &instr.instructions;
            }
            ValueType::Sequence => {
                instructions = &instr.instructions[1..];
            }
            _ => {
                return Ok(());
            }
        }
        let need_pmap = self.require_presence_map_bit(instructions)?;
        instr.has_pmap.set(need_pmap);
        Ok(())
    }

    fn has_presence_map_bit(&self, instr: &Instruction) -> Result<bool> {
        // first, initialize internals of the instruction
        self.set_has_pmap(instr)?;

        // then, check if it has a presence map bit
        match instr.value_type {
            ValueType::Group => {
                // If a ::Group field is optional, it will occupy a single bit in the presence map.
                return Ok(instr.is_optional())
            }
            ValueType::Sequence => {
                // For ::Sequence its length field show if the sequence has a bit in the presence map.
                return self.has_presence_map_bit(
                    instr.instructions.get(0)
                        .ok_or_else(|| Error::Static(format!("sequence '{}' has no length field", instr.name)))?
                );
            }
            ValueType::TemplateReference => {
                if !instr.name.is_empty() {
                    // Static template ref checks corresponding template it is needs any presence bit.
                    let template = match self.templates_by_name.get(&instr.name) {
                        None => return Err(Error::Static(format!("template '{}' not found", instr.name))),
                        Some(t) => t,
                    };
                    match template.require_pmap.get() {
                        None => return Err(Error::Static(
                                format!("template '{}' not initialized yet; consider reordering templates", instr.name)
                            )),
                        Some(b) => return Ok(b),
                    }
                } else {
                    // Dynamic template ref doesn't need a presence map bit.
                    return Ok(false);
                }
            }
            ValueType::Decimal => {
                if instr.has_pmap.get() {
                    // We already know that this field require a presence bit due to its subcomponents.
                    return Ok(true);
                }
                // Otherwise, fall-though and check the field's operator (like for normal field).
            }
            _ => {}
        }
        match instr.operator {
            // If a field (is mandatory and) has no field operator, it will not occupy any bit in the presence map
            // and its value must always appear in the stream.
            // TODO: Check! According to Presence Map table from the spec, field with no operator is always present in the stream!
            Operator::None => Ok(false),
            // Delta is always present in the stream, so doesn't need a presence bit.
            Operator::Delta => Ok(false),
            // Always require a presence bit.
            Operator::Default | Operator::Copy | Operator::Increment | Operator::Tail => Ok(true),
            // An optional field with the constant operator will occupy a single bit.
            Operator::Constant => Ok(instr.is_optional()),
        }
    }
}
