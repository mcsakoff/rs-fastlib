use std::io::Read;
use std::rc::Rc;

use crate::{Error, Result};
use crate::base::instruction::Instruction;
use crate::base::message::MessageFactory;
use crate::base::pmap::PresenceMap;
use crate::base::types::{Dictionary, Template, TypeRef};
use crate::base::value::{Value, ValueType};
use crate::common::context::{Context, DictionaryType};
use crate::common::definitions::Definitions;
use crate::decoder::reader::{Reader, StreamReader};
use crate::utils::stacked::Stacked;

/// Decoder for FAST protocol messages.
pub struct Decoder {
    pub(crate) definitions: Definitions,
    pub(crate) context: Context,
}

impl Decoder {
    #[allow(unused)]
    pub(crate) fn new_from_templates(ts: Vec<Template>) -> Result<Self> {
        Ok(Decoder {
            definitions: Definitions::new_from_templates(ts)?,
            context: Context::new(),
        })
    }

    pub fn new_from_xml(text: &str) -> Result<Self> {
        Ok(Decoder {
            definitions: Definitions::new_from_xml(text)?,
            context: Context::new(),
        })
    }

    pub fn reset(&mut self) {
        self.context.reset()
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

    /// Decode single message from object that implements [`std::io::Read`][std::io::Read] trait.
    pub fn decode_stream(&mut self, rdr: &mut dyn Read, msg: &mut impl MessageFactory) -> Result<()> {
        let mut rdr = StreamReader::new(rdr);
        self.decode_reader(&mut rdr, msg)
    }

    /// Decode single message from object that implements [`fastlib::Reader`][crate::decoder::reader::Reader] trait.
    pub fn decode_reader(&mut self, rdr: &mut impl Reader, msg: &mut impl MessageFactory) -> Result<()> {
        DecoderContext::new(self, rdr, msg).decode_template()
    }
}

/// Processing context of the decoder. It represents context state during one message decoding.
/// Created when it starts decoding a new message and destroyed after decoding of a message.
pub(crate) struct DecoderContext<'a> {
    pub(crate) definitions: &'a mut Definitions,
    pub(crate) context: &'a mut Context,
    pub(crate) rdr: Box<&'a mut dyn Reader>,
    pub(crate) msg: Box<&'a mut dyn MessageFactory>,

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

impl<'a> DecoderContext<'a> {
    pub(crate) fn new(d: &'a mut Decoder,
                      r: &'a mut impl Reader,
                      m: &'a mut impl MessageFactory,
    ) -> Self {
        Self {
            definitions: &mut d.definitions,
            context: &mut d.context,
            rdr: Box::new(r),
            msg: Box::new(m),
            template_id: Stacked::new_empty(),
            dictionary: Stacked::new(Dictionary::Global),
            type_ref: Stacked::new(TypeRef::Any),
            presence_map: Stacked::new_empty(),
        }
    }

    // Read template id from the stream.
    fn read_template_id(&mut self) -> Result<u32> {
        let instruction = self.definitions.template_id_instruction.clone();
        match instruction.extract(self)? {
            Some(Value::UInt32(id)) => Ok(id),
            Some(_) => Err(Error::Runtime("Wrong template id type in context storage".to_string())),
            None => Err(Error::Runtime("No template id in context storage".to_string())),
        }
    }

    // Decode template id from the stream and change the current processing context accordingly.
    fn decode_template_id(&mut self) -> Result<()> {
        let template_id = self.read_template_id()?;
        self.template_id.push(template_id);
        Ok(())
    }

    // Stop processing the current template id, restore the previous value in the processing context.
    fn drop_template_id(&mut self) {
        self.template_id.pop();
    }

    // Decode presence map from the stream and change the current processing context accordingly.
    fn decode_presence_map(&mut self) -> Result<()> {
        let (bitmap, size) = self.rdr.read_presence_map()?;
        let presence_map = PresenceMap::new(bitmap, size);
        self.presence_map.push(presence_map);
        Ok(())
    }

    // Restore the previous value for presence map in the processing context.
    fn drop_presence_map(&mut self) {
        _ = self.presence_map.pop();
    }

    // Decode a template from the stream.
    pub(crate) fn decode_template(&mut self) -> Result<()> {
        self.decode_presence_map()?;
        self.decode_template_id()?;
        let template = self.definitions.templates_by_id
            .get(self.template_id.peek().unwrap())
            .ok_or_else(|| Error::Dynamic(format!("Unknown template id: {}", self.template_id.peek().unwrap())))? // [ErrD09]
            .clone(); //
        self.msg.start_template(template.id, &template.name);

        // Update some context variables
        let has_dictionary = self.switch_dictionary(&template.dictionary);
        let has_type_ref = self.switch_type_ref(&template.type_ref);

        self.decode_instructions(&template.instructions)?;

        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }

        self.msg.stop_template();
        self.drop_template_id();
        self.drop_presence_map();
        Ok(())
    }

    fn decode_instructions(&mut self, instructions: &[Instruction]) -> Result<()> {
        for instruction in instructions {
            match instruction.value_type {
                ValueType::Sequence => {
                    self.decode_sequence(instruction)?;
                }
                ValueType::Group => {
                    self.decode_group(instruction)?;
                }
                ValueType::TemplateReference => {
                    self.decode_template_ref(instruction)?;
                }
                _ => {
                    self.decode_field(instruction)?;
                }
            }
        }
        Ok(())
    }

    fn decode_segment(&mut self, instructions: &[Instruction]) -> Result<()> {
        self.decode_presence_map()?;
        self.decode_instructions(instructions)?;
        self.drop_presence_map();
        Ok(())
    }

    fn decode_field(&mut self, instruction: &Instruction) -> Result<()> {
        let value = self.extract_field(instruction)?;
        self.msg.set_value(instruction.id, &instruction.name, value);
        Ok(())
    }

    // A sequence field instruction specifies that the field in the application type is of sequence type and that
    // the contained group of instructions should be used repeatedly to encode each element.
    fn decode_sequence(&mut self, instruction: &Instruction) -> Result<()> {
        let has_dictionary = self.switch_dictionary(&instruction.dictionary);
        let has_type_ref = self.switch_type_ref(&instruction.type_ref);

        // A sequence has an associated length field containing an unsigned integer indicating the number of encoded
        // elements. When a length field is present in the stream, it must appear directly before the encoded elements.
        // The length field has a name, is of type uInt32 and can have a field operator.
        let length_instruction = instruction.instructions.get(0).unwrap();
        match self.extract_field(length_instruction)? {
            None => {}
            Some(Value::UInt32(length)) => {
                self.msg.start_sequence(instruction.id, &instruction.name, length);
                for idx in 0..length {
                    self.msg.start_sequence_item(idx);
                    // If any instruction of the sequence needs to allocate a bit in a presence map, each element is represented
                    // as a segment in the transfer encoding.
                    if instruction.has_pmap.get() {
                        self.decode_segment(&instruction.instructions[1..])?;
                    } else {
                        self.decode_instructions(&instruction.instructions[1..])?;
                    }
                    self.msg.stop_sequence_item();
                }
                self.msg.stop_sequence();
            }
            _ => return Err(Error::Dynamic("Length field must be UInt32".to_string())), // [ErrD10]
        }

        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }
        Ok(())
    }

    // A group field instruction associates a name and presence attribute with a group of instructions.
    // If any instruction of the group needs to allocate a bit in a presence map, the group is represented
    // as a segment in the transfer encoding.
    fn decode_group(&mut self, instruction: &Instruction) -> Result<()> {
        if instruction.is_optional() && !self.pmap_next_bit_set() {
            return Ok(());
        }

        let has_dictionary = self.switch_dictionary(&instruction.dictionary);
        let has_type_ref = self.switch_type_ref(&instruction.type_ref);

        self.msg.start_group(&instruction.name);
        // If any instruction of the group needs to allocate a bit in a presence map, each element is represented
        // as a segment in the transfer encoding.
        if instruction.has_pmap.get() {
            self.decode_segment(&instruction.instructions)?;
        } else {
            self.decode_instructions(&instruction.instructions)?;
        }
        self.msg.stop_group();

        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }
        Ok(())
    }

    // The template reference instruction specifies that a part of the template is specified by another template.
    // A template reference can be either static or dynamic. A reference is static when a name is specified in the
    // instruction. Otherwise, it is dynamic.
    fn decode_template_ref(&mut self, instruction: &Instruction) -> Result<()> {
        let is_dynamic = instruction.name.is_empty();

        let template: Rc<Template>;
        if is_dynamic {
            self.decode_presence_map()?;
            self.decode_template_id()?;
            template = self.definitions.templates_by_id
                .get(self.template_id.peek().unwrap())
                .ok_or_else(|| Error::Dynamic(format!("Unknown template id: {}", self.template_id.peek().unwrap())))? // [ErrD09]
                .clone();
        } else {
            template = self.definitions.templates_by_name
                .get(&instruction.name)
                .ok_or_else(|| Error::Dynamic(format!("Unknown template: {}", instruction.name)))? // [ErrD09]
                .clone();
        }
        self.msg.start_template_ref(&template.name, is_dynamic);

        // Update some context variables
        let has_dictionary = self.switch_dictionary(&template.dictionary);
        let has_type_ref = self.switch_type_ref(&template.type_ref);

        self.decode_instructions(&template.instructions)?;

        if has_dictionary { self.restore_dictionary() }
        if has_type_ref { self.restore_type_ref() }

        self.msg.stop_template_ref();
        if is_dynamic {
            self.drop_template_id();
            self.drop_presence_map();
        }
        Ok(())
    }

    fn extract_field(&mut self, instruction: &Instruction) -> Result<Option<Value>> {
        let has_dict = self.switch_dictionary(&instruction.dictionary);
        let value = instruction.extract(self)?;
        if has_dict {
            self.restore_dictionary();
        }
        Ok(value)
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
    pub(crate) fn pmap_next_bit_set(&mut self) -> bool {
        self.presence_map.must_peek_mut().next_bit_set()
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
