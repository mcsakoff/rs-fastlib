use std::io::Write;
use std::rc::Rc;

use bytes::BytesMut;

use crate::{Error, Result};
use crate::base::instruction::Instruction;
use crate::base::message::MessageVisitor;
use crate::base::pmap::PresenceMap;
use crate::base::types::{Dictionary, Template, TypeRef};
use crate::base::value::{Value, ValueType};
use crate::common::context::{Context, DictionaryType};
use crate::common::definitions::Definitions;
use crate::encoder::writer::{StreamWriter, Writer};
use crate::utils::stacked::Stacked;

/// Encoder for FAST protocol messages.
pub struct Encoder {
    pub(crate) definitions: Definitions,
    pub(crate) context: Context,
}

impl Encoder {
    #[allow(unused)]
    pub(crate) fn new_from_templates(ts: Vec<Template>) -> Result<Self> {
        Ok(Encoder {
            definitions: Definitions::new_from_templates(ts)?,
            context: Context::new(),
        })
    }

    pub fn new_from_xml(text: &str) -> Result<Self> {
        Ok(Encoder {
            definitions: Definitions::new_from_xml(text)?,
            context: Context::new(),
        })
    }

    pub fn reset(&mut self) {
        self.context.reset()
    }

    pub fn encode_vec(&mut self, msg: &mut impl MessageVisitor) -> Result<Vec<u8>> {
        let mut buf = BytesMut::new();
        self.encode_writer(&mut buf, msg)?;
        Ok(buf.to_vec())
    }

    pub fn encode_bytes(&mut self, msg: &mut impl MessageVisitor) -> Result<BytesMut> {
        let mut buf = BytesMut::new();
        self.encode_writer(&mut buf, msg)?;
        Ok(buf)
    }

    pub fn encode_stream(&mut self, wrt: &mut dyn Write, msg: &mut impl MessageVisitor) -> Result<()> {
        let mut wrt = StreamWriter::new(wrt);
        self.encode_writer(&mut wrt, msg)
    }

    pub fn encode_writer(&mut self, wrt: &mut impl Writer, msg: &mut impl MessageVisitor) -> Result<()> {
        EncoderContext::new(self, wrt, msg).encode_template()
    }
}

/// Processing context of the encoder. It represents context state during one message encoding.
/// Created when it starts encoding a new message and destroyed after encoding of a message.
pub(crate) struct EncoderContext<'a> {
    pub(crate) definitions: &'a mut Definitions,
    pub(crate) context: &'a mut Context,
    pub(crate) wrt: Box<&'a mut dyn Writer>,
    pub(crate) msg: Box<&'a mut dyn MessageVisitor>,

    // The current template id.
    // It is updated when a template identifier is encountered in the stream. A static template reference can also change
    // the current template as described in the Template Reference Instruction section.
    pub(crate) template_id: Stacked<u32>,

    // The dictionary set and initial value are described in the Operators section.
    pub(crate) dictionary: Stacked<Dictionary>,

    // The current application type is initially the special type `any`. The current application type changes when the processor
    // encounters an element containing a `typeRef` element. The new type is applicable to the instructions contained within
    // the element. The `typeRef` can appear in the <template>, <group> and <sequence> elements.
    pub(crate) type_ref: Stacked<TypeRef>,

    // The presence map of the current segment.
    pub(crate) presence_map: Stacked<PresenceMap>,
}

impl<'a> EncoderContext<'a> {
    pub(crate) fn new(d: &'a mut Encoder,
                      w: &'a mut impl Writer,
                      m: &'a mut impl MessageVisitor,
    ) -> Self {
        Self {
            definitions: &mut d.definitions,
            context: &mut d.context,
            wrt: Box::new(w),
            msg: Box::new(m),
            template_id: Stacked::new_empty(),
            dictionary: Stacked::new(Dictionary::Global),
            type_ref: Stacked::new(TypeRef::Any),
            presence_map: Stacked::new(PresenceMap::new_empty()),
        }
    }

    // Encode a template to the stream.
    fn encode_template(&mut self) -> Result<()> {
        let template_name = self.msg.get_template_name()?;
        let template = self.definitions.templates_by_name
            .get(&template_name)
            .ok_or_else(|| Error::Dynamic(format!("Unknown template name: {}", template_name)))?
            .clone();

        let mut buf = BytesMut::new();
        self.encode_template_id(&mut buf, template.id)?;

        // Update some context variables
        let has_dictionary = self.switch_dictionary(&template.dictionary);
        let has_type_ref = self.switch_type_ref(&template.type_ref);

        self.encode_instructions(&mut buf, &template.instructions)?;

        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }

        self.drop_template_id();

        let mut buf2 = BytesMut::new();
        self.write_presence_map(&mut buf2)?;
        buf2.write_buf(buf.as_ref())?;

        self.wrt.write_buf(buf2.as_ref()) // presence map + template_id + instructions
    }

    // Write presence map to the stream and remove if from the stack.
    fn write_presence_map(&mut self, buf: &mut dyn Writer) -> Result<()> {
        let presence_map = self.presence_map.pop().unwrap();
        buf.write_presence_map(presence_map.bitmap, presence_map.size)
    }

    // Encode template id to the buffer and change the current processing context accordingly.
    fn encode_template_id(&mut self, buf: &mut dyn Writer, template_id: u32) -> Result<()> {
        self.template_id.push(template_id);
        let instruction = self.definitions.template_id_instruction.clone();
        instruction.inject(self, buf, &Some(Value::UInt32(template_id)))
    }

    // Stop processing the current template id, restore the previous value in the processing context.
    fn drop_template_id(&mut self) {
        self.template_id.pop();
    }

    fn encode_instructions(&mut self, buf: &mut dyn Writer, instructions: &[Instruction]) -> Result<()> {
        for instruction in instructions {
            match instruction.value_type {
                ValueType::Sequence => {
                    self.encode_sequence(buf, instruction)?;
                }
                ValueType::Group => {
                    self.encode_group(buf, instruction)?;
                }
                ValueType::TemplateReference => {
                    self.encode_template_reference(buf, instruction)?;
                }
                _ => {
                    self.encode_field(buf, instruction)?;
                }
            }
        }
        Ok(())
    }

    fn encode_field(&mut self, buf: &mut dyn Writer, instruction: &Instruction) -> Result<()> {
        let value = self.msg.get_value(&instruction.name)?;
        instruction.inject(self, buf, &value)
    }

    fn encode_segment(&mut self, buf: &mut dyn Writer, instructions: &[Instruction]) -> Result<()> {
        self.presence_map.push(PresenceMap::new_empty());
        let mut buf2 = BytesMut::new();
        self.encode_instructions(&mut buf2, instructions)?;
        self.write_presence_map(buf)?;
        buf.write_buf(buf2.as_ref())
    }

    fn encode_group(&mut self, buf: &mut dyn Writer, instruction: &Instruction) -> Result<()> {
        if !self.msg.select_group(&instruction.name)? {
            return if instruction.is_optional() {
                self.pmap_set_next_bit(false);
                Ok(())
            } else {
                Err(Error::Dynamic(format!("Missing mandatory group: {}", instruction.name)))
            };
        }
        if instruction.is_optional() {
            self.pmap_set_next_bit(true);
        }

        let has_dictionary = self.switch_dictionary(&instruction.dictionary);
        let has_type_ref = self.switch_type_ref(&instruction.type_ref);

        if instruction.has_pmap.get() {
            self.encode_segment(buf, &instruction.instructions)?;
        } else {
            self.encode_instructions(buf, &instruction.instructions)?;
        }

        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }

        self.msg.release_group()
    }

    fn encode_sequence(&mut self, buf: &mut dyn Writer, instruction: &Instruction) -> Result<()> {
        let length = self.msg.select_sequence(&instruction.name)?;
        let length_instruction = instruction.instructions.get(0).unwrap();

        let has_dictionary = self.switch_dictionary(&instruction.dictionary);
        let has_type_ref = self.switch_type_ref(&instruction.type_ref);
        match length {
            None => {
                if instruction.is_optional() {
                    length_instruction.inject(self, buf, &None)?;
                } else {
                    return Err(Error::Dynamic(format!("Missing mandatory sequence: {}", instruction.name)));
                }
            }
            Some(length) => {
                length_instruction.inject(self, buf, &Some(Value::UInt32(length as u32)))?;
                for idx in 0..length {
                    self.msg.select_sequence_item(idx)?;
                    if instruction.has_pmap.get() {
                        self.encode_segment(buf, &instruction.instructions[1..])?;
                    } else {
                        self.encode_instructions(buf, &instruction.instructions[1..])?;
                    }
                    self.msg.release_sequence_item()?;
                }
                self.msg.release_sequence()?;
            }
        }
        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }

        Ok(())
    }

    fn encode_template_reference(&mut self, buf: &mut dyn Writer, instruction: &Instruction) -> Result<()> {
        let is_dynamic = instruction.name.is_empty();

        if is_dynamic {
            let template_name = match self.msg.select_template_ref(&instruction.name, true)? {
                Some(name) => name,
                None => {
                    return Err(Error::Dynamic(format!("Missing mandatory template reference: {}", instruction.name)))
                }
            };
            let template = self.definitions.templates_by_name
                .get(&template_name)
                .ok_or_else(|| Error::Dynamic(format!("Unknown template name: {}", template_name)))? // [ErrD09]
                .clone();

            let mut buf2 = BytesMut::new();
            self.presence_map.push(PresenceMap::new_empty());
            self.encode_template_id(&mut buf2, template.id)?;

            let has_dictionary = self.switch_dictionary(&template.dictionary);
            let has_type_ref = self.switch_type_ref(&template.type_ref);

            self.encode_instructions(&mut buf2, &template.instructions)?;

            if has_dictionary { self.restore_dictionary() }
            if has_type_ref { self.restore_type_ref() }

            self.drop_template_id();

            self.write_presence_map(buf)?;
            buf.write_buf(buf2.as_ref())?;
        } else {
            self.msg.select_template_ref(&instruction.name, false)?;
            let template = self.definitions.templates_by_name
                .get(&instruction.name)
                .ok_or_else(|| Error::Dynamic(format!("Unknown template: {}", instruction.name)))? // [ErrD09]
                .clone();

            let has_dictionary = self.switch_dictionary(&template.dictionary);
            let has_type_ref = self.switch_type_ref(&template.type_ref);

            self.encode_instructions(buf, &template.instructions)?;

            if has_dictionary { self.restore_dictionary() }
            if has_type_ref { self.restore_type_ref() }
        }
        self.msg.release_template_ref()
    }

    #[inline]
    fn switch_dictionary(&mut self, dictionary: &Dictionary) -> bool {
        if *dictionary != Dictionary::Inherit {
            self.dictionary.push(dictionary.clone());
            true
        } else {
            false
        }
    }

    #[inline]
    fn restore_dictionary(&mut self) {
        _ = self.dictionary.pop();
    }

    #[inline]
    fn switch_type_ref(&mut self, type_ref: &TypeRef) -> bool {
        if *type_ref != TypeRef::Any {
            self.type_ref.push(type_ref.clone());
            true
        } else {
            false
        }
    }

    #[inline]
    fn restore_type_ref(&mut self) {
        _ = self.type_ref.pop();
    }

    #[inline]
    pub(crate) fn pmap_set_next_bit(&mut self, value: bool) {
        self.presence_map.must_peek_mut().set_next_bit(value)
    }

    #[inline]
    pub(crate) fn ctx_set(&mut self, i: &Instruction, v: &Option<Value>) {
        self.context.set(self.make_dict_type(), i.key.clone(), v);
    }

    #[inline]
    pub(crate) fn ctx_get(&mut self, i: &Instruction) -> Result<Option<Option<Value>>> {
        let v = self.context.get(self.make_dict_type(), &i.key);
        if let Some(Some(ref v)) = v {
            if !i.value_type.matches_type(v) {
                // It is a dynamic error [ERR D4] if the field of an operator accessing an entry does not have
                // the same type as the value of the entry.
                return Err(Error::Runtime(format!("field {} has wrong value type in context", i.name)));  // [ERR D4]
            }
        }
        Ok(v)
    }

    fn make_dict_type(&self) -> DictionaryType {
        let dictionary = self.dictionary.must_peek();
        match dictionary {
            Dictionary::Inherit => unreachable!(),
            Dictionary::Global => {
                DictionaryType::Global
            }
            Dictionary::Template => {
                DictionaryType::Template(*self.template_id.must_peek())
            }
            Dictionary::Type => {
                let name = match self.type_ref.must_peek() {
                    TypeRef::Any => Rc::from("__any__"),
                    TypeRef::ApplicationType(name) => name.clone(),
                };
                DictionaryType::Type(name)
            }
            Dictionary::UserDefined(name) => {
                DictionaryType::UserDefined(name.clone())
            }
        }
    }
}
