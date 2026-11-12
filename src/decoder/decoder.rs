use std::io::Read;

use crate::base::instruction::Instruction;
use crate::base::message::MessageFactory;
use crate::base::pmap::PresenceMap;
use crate::base::types::{Template, TypeRef};
use crate::base::value::{Value, ValueType};
use crate::common::context::{Context, Scopes};
use crate::common::definitions::Definitions;
use crate::decoder::reader::{Reader, SliceReader, StreamReader};
use crate::utils::stacked::Stacked;
use crate::{Error, Result};

/// Decoder for FAST protocol messages.
pub struct Decoder {
    pub(crate) definitions: Definitions,
    pub(crate) context: Context,
    pub(crate) scopes: Scopes,
}

impl Decoder {
    #[allow(unused)]
    /// Creates decoder from template lists.
    /// # Errors
    /// Returns error if invalid templates given.
    pub(crate) fn new_from_templates(ts: Vec<Template>) -> Result<Self> {
        Ok(Decoder {
            definitions: Definitions::new_from_templates(ts)?,
            context: Context::new(),
            scopes: Scopes::new(),
        })
    }

    /// Creates Decoder from XML definitions.
    /// # Errors
    /// Returns error if invalid definitions given.
    pub fn new_from_xml(text: &str) -> Result<Self> {
        Ok(Decoder {
            definitions: Definitions::new_from_xml(text)?,
            context: Context::new(),
            scopes: Scopes::new(),
        })
    }

    pub fn reset(&mut self) {
        self.context.reset();
    }

    /// Decode single message from buffer.
    /// Returns number of bytes consumed from the buffer.
    /// # Errors
    /// Returns error if message decode failed.
    pub fn decode_buffer(&mut self, buffer: &[u8], msg: &mut impl MessageFactory) -> Result<u64> {
        let mut rdr = SliceReader::new(buffer);
        self.decode_reader(&mut rdr, msg)?;
        Ok(rdr.position() as u64)
    }

    /// Decode single message from slice.
    /// The `bytes` slice must be consumed completely. It is an error if any bytes left after the message is decoded.
    /// # Errors
    /// Returns error if message decode failed.
    pub fn decode_slice(&mut self, bytes: &[u8], msg: &mut impl MessageFactory) -> Result<()> {
        let mut rdr = SliceReader::new(bytes);
        self.decode_reader(&mut rdr, msg)?;
        if !rdr.remaining().is_empty() {
            return Err(Error::Dynamic(format!(
                "Bytes left in the buffer after decoding: {}",
                rdr.remaining().len()
            )));
        }
        Ok(())
    }

    /// Decode single message from bytes vector.
    /// The `bytes` vector must be the whole message. It is an error if any bytes left after the message is decoded.
    /// # Errors
    /// Returns error if message decode failed.
    pub fn decode_vec(&mut self, bytes: Vec<u8>, msg: &mut impl MessageFactory) -> Result<()> {
        let mut raw = bytes::Bytes::from(bytes);
        self.decode_reader(&mut raw, msg)?;
        if !raw.is_empty() {
            return Err(Error::Dynamic(format!(
                "Bytes left in the buffer after decoding: {}",
                raw.len()
            )));
        }
        Ok(())
    }

    /// Decode single message from `bytes::Bytes`.
    /// # Errors
    /// Returns error if message decode failed.
    pub fn decode_bytes(
        &mut self,
        bytes: &mut bytes::Bytes,
        msg: &mut impl MessageFactory,
    ) -> Result<()> {
        self.decode_reader(bytes, msg)
    }

    /// Decode single message from object that implements [`std::io::Read`][std::io::Read] trait.
    /// # Errors
    /// Returns error if message decode failed.
    pub fn decode_stream(
        &mut self,
        rdr: &mut dyn Read,
        msg: &mut impl MessageFactory,
    ) -> Result<()> {
        let mut rdr = StreamReader::new(rdr);
        self.decode_reader(&mut rdr, msg)
    }

    /// Decode single message from object that implements [`fastlib::Reader`][crate::decoder::reader::Reader] trait.
    /// # Errors
    /// Returns error if message decode failed.
    pub fn decode_reader(
        &mut self,
        rdr: &mut impl Reader,
        msg: &mut impl MessageFactory,
    ) -> Result<()> {
        DecoderContext::new(self, rdr, msg).decode_template()
    }
}

/// Processing context of the decoder. It represents context state during one message decoding.
/// Created when it starts decoding a new message and destroyed after decoding of a message.
pub(crate) struct DecoderContext<'a, R, M> {
    pub(crate) definitions: &'a Definitions,
    pub(crate) context: &'a mut Context,
    pub(crate) rdr: &'a mut R,
    pub(crate) msg: &'a mut M,

    // The index of the current template (`Template::index`).
    // It is updated when a template identifier is encountered in the stream. A static template reference can also change
    // the current template as described in the Template Reference Instruction section.
    pub(crate) template: &'a mut Stacked<u32>,

    // The current application type is initially the special type `any`. The current application type changes when the processor
    // encounters an element containing a `typeRef` element. The new type is applicable to the instructions contained within
    // the element. The `typeRef` can appear in the <template>, <group> and <sequence> elements.
    // Holds index of the apllication type (`Definitions::type_index`), 0 is `any`.
    pub(crate) type_ref: &'a mut Stacked<u32>,

    // The presence map of the current segment.
    pub(crate) presence_map: &'a mut Stacked<PresenceMap>,
}

impl<'a, R: Reader, M: MessageFactory> DecoderContext<'a, R, M> {
    pub(crate) fn new(d: &'a mut Decoder, r: &'a mut R, m: &'a mut M) -> Self {
        d.scopes.reset();
        Self {
            definitions: &d.definitions,
            context: &mut d.context,
            rdr: r,
            msg: m,
            template: &mut d.scopes.template,
            type_ref: &mut d.scopes.type_ref,
            presence_map: &mut d.scopes.presence_map,
        }
    }

    // Read template id from the stream.
    fn read_template_id(&mut self) -> Result<u32> {
        let definitions = self.definitions;
        match definitions.template_id_instruction.extract(self)? {
            Some(Value::UInt32(id)) => Ok(id),
            Some(_) => Err(Error::Dynamic(
                "Wrong template id type in context storage".to_string(),
            )),
            None => Err(Error::Dynamic(
                "No template id in context storage".to_string(),
            )),
        }
    }

    // Decode template id from the stream and make the template current.
    fn decode_template_id(&mut self) -> Result<&'a Template> {
        let template_id = self.read_template_id()?;
        let template = self.definitions.template_by_id(template_id)?;
        self.template.push(template.index);
        Ok(template)
    }

    // Stop processing the current template, restore the previous one in the processing context.
    fn drop_template(&mut self) {
        self.template.pop();
    }

    // Decode presence map from the stream and change the current processing context accordingly.
    // `long` is `true` if the segment can have a presence map longer than `MAX_PMAP_BITS`.
    fn decode_presence_map(&mut self, long: bool) -> Result<()> {
        let presence_map = if long {
            self.read_long_presence_map()?
        } else {
            let (bitmap, size) = self.rdr.read_presence_map()?;
            // A presence map is overlong if it has more than seven bits and ends in seven or more bits that are all zero.
            if size > 7 && bitmap.trailing_zeros() >= 7 {
                return Err(overlong_presence_map()); // [ERR R7]
            }
            PresenceMap::new(bitmap, size)
        };
        self.presence_map.push(presence_map);
        Ok(())
    }

    #[cold]
    fn read_long_presence_map(&mut self) -> Result<PresenceMap> {
        let mut groups = Vec::new();
        match self.rdr.read_stop_bit_run(&mut groups) {
            Ok(()) => {}
            // The stream ends cleanly at the first byte of the presence map.
            Err(Error::UnexpectedEof) if groups.is_empty() => return Err(Error::Eof),
            Err(e) => return Err(e),
        }
        if let Some(last) = groups.last_mut() {
            *last &= 0x7f;
        }
        if groups.len() > 1 && groups.last() == Some(&0) {
            return Err(overlong_presence_map()); // [ERR R7]
        }
        Ok(PresenceMap::from_groups(groups))
    }

    // Restore the previous value for presence map in the processing context.
    fn drop_presence_map(&mut self) -> Result<()> {
        let presence_map = self.presence_map.pop();
        // It is a reportable error [ERR R8] if a presence map contains more bits than required by the instructions.
        if presence_map.is_some_and(|p| p.has_unread_bits_set()) {
            return Err(Error::Dynamic(
                "presence map has more bits than required".to_string(),
            ));
        }
        Ok(())
    }

    // Decode a template from the stream.
    pub(crate) fn decode_template(&mut self) -> Result<()> {
        self.decode_presence_map(self.definitions.long_template_pmap)?;
        let template = self.decode_template_id()?;
        // Templates selected by the identifier always have one.
        self.msg
            .start_template(template.id.unwrap_or_default(), &template.name);

        // Update some context variables
        let has_type_ref = self.switch_type_ref(&template.type_ref);

        self.decode_instructions(&template.instructions)?;

        if has_type_ref {
            self.restore_type_ref();
        }

        self.msg.stop_template();
        self.drop_template();
        self.drop_presence_map()
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

    // `long` is `true` if the segment can have a presence map longer than `MAX_PMAP_BITS`.
    fn decode_segment(&mut self, instructions: &[Instruction], long: bool) -> Result<()> {
        self.decode_presence_map(long)?;
        self.decode_instructions(instructions)?;
        self.drop_presence_map()
    }

    fn decode_field(&mut self, instruction: &Instruction) -> Result<()> {
        let value = instruction.extract(self)?;
        if let Some(length) = instruction.length_field() {
            // The named length preamble of a byte vector is reported before the value.
            let len = match &value {
                Some(Value::Bytes(b)) => Some(b.len()),
                Some(Value::UnicodeString(s) | Value::ASCIIString(s)) => Some(s.len()),
                _ => None,
            };
            let len = len
                .map(|l| u32::try_from(l).map(Value::UInt32))
                .transpose()
                .map_err(|_| Error::Dynamic(format!("{} is too long", instruction.name)))?;
            self.msg.set_value(length.id, &length.name, len);
        }
        self.msg.set_value(instruction.id, &instruction.name, value);
        Ok(())
    }

    // A sequence field instruction specifies that the field in the application type is of sequence type and that
    // the contained group of instructions should be used repeatedly to encode each element.
    fn decode_sequence(&mut self, instruction: &Instruction) -> Result<()> {
        let has_type_ref = self.switch_type_ref(&instruction.type_ref);

        // A sequence has an associated length field containing an unsigned integer indicating the number of encoded
        // elements. When a length field is present in the stream, it must appear directly before the encoded elements.
        // The length field has a name, is of type uInt32 and can have a field operator.
        let length_instruction = instruction.instructions.first().unwrap();
        match length_instruction.extract(self)? {
            None => {}
            Some(Value::UInt32(length)) => {
                self.msg
                    .start_sequence(instruction.id, &instruction.name, length);
                for idx in 0..length {
                    self.msg.start_sequence_item(idx);
                    // If any instruction of the sequence needs to allocate a bit in a presence map, each element is represented
                    // as a segment in the transfer encoding.
                    if instruction.has_pmap.get() {
                        self.decode_segment(
                            &instruction.instructions[1..],
                            instruction.long_pmap.get(),
                        )?;
                    } else {
                        self.decode_instructions(&instruction.instructions[1..])?;
                    }
                    self.msg.stop_sequence_item();
                }
                self.msg.stop_sequence();
            }
            _ => return Err(Error::Dynamic("Length field must be UInt32".to_string())), // [ErrD10]
        }

        if has_type_ref {
            self.restore_type_ref();
        }
        Ok(())
    }

    // A group field instruction associates a name and presence attribute with a group of instructions.
    // If any instruction of the group needs to allocate a bit in a presence map, the group is represented
    // as a segment in the transfer encoding.
    fn decode_group(&mut self, instruction: &Instruction) -> Result<()> {
        if instruction.is_optional() && !self.pmap_next_bit_set() {
            return Ok(());
        }

        let has_type_ref = self.switch_type_ref(&instruction.type_ref);

        self.msg.start_group(&instruction.name);
        // If any instruction of the group needs to allocate a bit in a presence map, each element is represented
        // as a segment in the transfer encoding.
        if instruction.has_pmap.get() {
            self.decode_segment(&instruction.instructions, instruction.long_pmap.get())?;
        } else {
            self.decode_instructions(&instruction.instructions)?;
        }
        self.msg.stop_group();

        if has_type_ref {
            self.restore_type_ref();
        }
        Ok(())
    }

    // The template reference instruction specifies that a part of the template is specified by another template.
    // A template reference can be either static or dynamic. A reference is static when a name is specified in the
    // instruction. Otherwise, it is dynamic.
    fn decode_template_ref(&mut self, instruction: &Instruction) -> Result<()> {
        let is_dynamic = instruction.name.is_empty();

        let template = if is_dynamic {
            self.decode_presence_map(self.definitions.long_template_pmap)?;
            self.decode_template_id()?
        } else {
            // A static reference continues with the referred template as the current template.
            let template = self.definitions.static_template_ref(instruction);
            self.template.push(template.index);
            template
        };
        self.msg.start_template_ref(&template.name, is_dynamic);

        // Update some context variables
        let has_type_ref = self.switch_type_ref(&template.type_ref);

        self.decode_instructions(&template.instructions)?;

        if has_type_ref {
            self.restore_type_ref();
        }

        self.msg.stop_template_ref();
        self.drop_template();
        if is_dynamic {
            self.drop_presence_map()?;
        }
        Ok(())
    }

    #[inline]
    fn switch_type_ref(&mut self, type_ref: &TypeRef) -> bool {
        match Definitions::type_index(type_ref) {
            None => false,
            Some(index) => {
                self.type_ref.push(index);
                true
            }
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
    pub(crate) fn ctx_set(&mut self, i: &Instruction, v: Option<Value>) {
        let template = self.template.peek().copied().unwrap_or_default();
        let type_index = *self.type_ref.must_peek();
        self.context
            .set(i.dictionary_slot.get(), template, type_index, v);
    }

    #[inline]
    pub(crate) fn ctx_get(&mut self, i: &Instruction) -> Result<Option<&Option<Value>>> {
        let template = self.template.peek().copied().unwrap_or_default();
        let type_index = *self.type_ref.must_peek();
        let v = self
            .context
            .get(i.dictionary_slot.get(), template, type_index);
        if let Some(Some(v)) = v
            && !i.value_type.matches_type(v)
        {
            // It is a dynamic error [ERR D4] if the field of an operator accessing an entry does not have
            // the same type as the value of the entry.
            return Err(Error::Dynamic(format!(
                "field {} has wrong value type in context",
                i.name
            ))); // [ERR D4]
        }
        Ok(v)
    }

    #[inline]
    pub(crate) fn ctx_get_mut(&mut self, i: &Instruction) -> Result<Option<&mut Option<Value>>> {
        let template = self.template.peek().copied().unwrap_or_default();
        let type_index = *self.type_ref.must_peek();
        let v = self
            .context
            .get_mut(i.dictionary_slot.get(), template, type_index);
        if let Some(Some(v)) = v
            && !i.value_type.matches_type(v)
        {
            // It is a dynamic error [ERR D4] if the field of an operator accessing an entry does not have
            // the same type as the value of the entry.
            return Err(Error::Dynamic(format!(
                "field {} has wrong value type in context",
                i.name
            ))); // [ERR D4]
        }
        Ok(v)
    }
}

#[cold]
fn overlong_presence_map() -> Error {
    Error::Dynamic("overlong presence map".to_string()) // [ERR R7]
}
