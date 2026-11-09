use bytes::BytesMut;
use std::io::Write;

use crate::base::instruction::Instruction;
use crate::base::message::MessageVisitor;
use crate::base::pmap::PresenceMap;
use crate::base::types::{Template, TypeRef};
use crate::base::value::{Value, ValueType};
use crate::common::context::Context;
use crate::common::definitions::Definitions;
use crate::encoder::writer::{StreamWriter, Writer, presence_map_bytes};
use crate::utils::stacked::Stacked;
use crate::{Error, Result};

/// Encoder for FAST protocol messages.
pub struct Encoder {
    pub(crate) definitions: Definitions,
    pub(crate) context: Context,
    // Reusable buffer for encoding messages into writers.
    scratch: Vec<u8>,
}

impl Encoder {
    #[allow(unused)]
    pub(crate) fn new_from_templates(ts: Vec<Template>) -> Result<Self> {
        Ok(Encoder {
            definitions: Definitions::new_from_templates(ts)?,
            context: Context::new(),
            scratch: Vec::new(),
        })
    }

    /// Creates new encoder from given XML definition.
    /// # Errors
    /// Returns error if definition is ill-formed.
    pub fn new_from_xml(text: &str) -> Result<Self> {
        Ok(Encoder {
            definitions: Definitions::new_from_xml(text)?,
            context: Context::new(),
            scratch: Vec::new(),
        })
    }

    pub fn reset(&mut self) {
        self.context.reset();
    }

    /// Encodes message into a Vec buffer.
    /// Returns encoded buffer.
    /// # Errors
    /// Returns error if encoding failed.
    pub fn encode_vec(&mut self, msg: &mut impl MessageVisitor) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.encode_append(&mut buf, msg)?;
        Ok(buf)
    }

    /// Encodes messages and appends it to the end of `buf`.
    /// This is the most efficient way to encode messages when `buf` is reused.
    /// If encoding fails, `buf` is restored to its original lengths.
    /// # Errors
    /// Returns error if encoding failed.
    pub fn encode_append(
        &mut self,
        buf: &mut Vec<u8>,
        msg: &mut impl MessageVisitor,
    ) -> Result<()> {
        let len = buf.len();
        let res = MessageEncoder::new(self, msg).encode_template(buf);
        if res.is_err() {
            buf.truncate(len);
        }
        res
    }

    /// Encodes message into a bytes buffer.
    /// Returns encoded buffer.
    /// # Errors
    /// Returns error if encoding failed.
    pub fn encode_bytes(&mut self, msg: &mut impl MessageVisitor) -> Result<BytesMut> {
        Ok(BytesMut::from(self.encode_vec(msg)?.as_slice()))
    }

    /// Encodes message into a given stream.
    /// # Errors
    /// Returns error if encoding failed.
    pub fn encode_stream(
        &mut self,
        wrt: &mut dyn Write,
        msg: &mut impl MessageVisitor,
    ) -> Result<()> {
        let mut wrt = StreamWriter::new(wrt);
        self.encode_writer(&mut wrt, msg)
    }

    /// Encodes message into a given buffer.
    /// Returns the number of byres written.
    /// # Errors
    /// Returns error if encoding failed or the buffer if too small.
    pub fn encode_buffer(
        &mut self,
        buffer: &mut [u8],
        msg: &mut impl MessageVisitor,
    ) -> Result<usize> {
        use std::io::{Error, ErrorKind};

        self.with_scratch(msg, |scratch| {
            let dst = buffer.get_mut(..scratch.len()).ok_or_else(|| {
                Error::new(
                    ErrorKind::WriteZero,
                    "buffer is too small for the encoded message",
                )
            })?;
            dst.copy_from_slice(scratch);
            Ok(scratch.len())
        })
    }

    /// Encodes message into a given writer.
    /// # Errors
    /// Returns error if encoding failed.
    pub fn encode_writer(
        &mut self,
        wrt: &mut impl Writer,
        msg: &mut impl MessageVisitor,
    ) -> Result<()> {
        self.with_scratch(msg, |scratch| wrt.write_buf(scratch))
    }

    fn with_scratch<T>(
        &mut self,
        msg: &mut impl MessageVisitor,
        f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        let res = self
            .encode_append(&mut scratch, msg)
            .and_then(|()| f(&scratch));
        self.scratch = scratch;
        res
    }
}

/// Processing context of the encoder. It represents context state during one message encoding.
/// Created when it starts encoding a new message and destroyed after encoding of a message.
pub(crate) struct EncoderContext<'a> {
    pub(crate) definitions: &'a Definitions,
    pub(crate) context: &'a mut Context,

    // The current template id.
    // It is updated when a template identifier is encountered in the stream. A static template reference can also change
    // the current template as described in the Template Reference Instruction section.
    pub(crate) template_id: Stacked<u32>,

    // The current application type is initially the special type `any`. The current application type changes when the processor
    // encounters an element containing a `typeRef` element. The new type is applicable to the instructions contained within
    // the element. The `typeRef` can appear in the <template>, <group> and <sequence> elements.
    // Holds index of the apllication type (`Definitions::type_index`), 0 is `any`.
    pub(crate) type_ref: Stacked<u32>,

    // The presence map of the current segment.
    pub(crate) presence_map: Stacked<PresenceMap>,
}

impl<'a> EncoderContext<'a> {
    pub(crate) fn new(d: &'a mut Encoder) -> Self {
        Self {
            definitions: &d.definitions,
            context: &mut d.context,
            template_id: Stacked::new_empty(),
            type_ref: Stacked::new(0),
            presence_map: Stacked::new_empty(),
        }
    }

    // Encodes a segment: runs `f` with a new presence map and puts the presence map in front of the encoded data.
    // One byte is reserved for the presence mup up front; the data is shifted only if the presence map is longer.
    fn with_segment<T>(
        state: &mut T,
        ctx: impl Fn(&mut T) -> &mut Self,
        buf: &mut Vec<u8>,
        f: impl FnOnce(&mut T, &mut Vec<u8>) -> Result<()>,
    ) -> Result<()> {
        ctx(state).presence_map.push(PresenceMap::new_empty());
        let start = buf.len();
        buf.push(0x80);
        f(state, buf)?;
        let presence_map = ctx(state).presence_map.pop().unwrap();
        let (pmap, len) = presence_map_bytes(presence_map.bitmap, presence_map.size)?;
        if len == 1 {
            buf[start] = pmap[0];
        } else {
            buf.splice(start..=start, pmap[..len].iter().copied());
        }
        Ok(())
    }

    #[inline]
    fn switch_type_ref(&mut self, type_ref: &TypeRef) -> bool {
        match self.definitions.type_index(type_ref) {
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
    pub(crate) fn pmap_set_next_bit(&mut self, value: bool) {
        self.presence_map.must_peek_mut().set_next_bit(value);
    }

    #[inline]
    fn entry(&mut self, i: &Instruction) -> Option<&mut Option<Value>> {
        let template_id = self.template_id.peek().copied().unwrap_or_default();
        let type_index = *self.type_ref.must_peek();
        self.context
            .get_mut(i.dictionary_slot.get(), template_id, type_index)
    }

    #[inline]
    pub(crate) fn ctx_set(&mut self, i: &Instruction, v: Option<&Value>) {
        if let Some(entry) = self.entry(i) {
            match (entry, v) {
                (Some(dst), Some(src)) => dst.assign(src),
                (entry, v) => *entry = v.cloned(),
            }
            return;
        }
        let template_id = self.template_id.peek().copied().unwrap_or_default();
        let type_index = *self.type_ref.must_peek();
        self.context
            .set(i.dictionary_slot.get(), template_id, type_index, v.cloned());
    }

    #[inline]
    pub(crate) fn ctx_get(&mut self, i: &Instruction) -> Result<Option<&Option<Value>>> {
        let template_id = self.template_id.peek().copied().unwrap_or_default();
        let type_index = *self.type_ref.must_peek();

        let v = self
            .context
            .get(i.dictionary_slot.get(), template_id, type_index);
        if let Some(Some(v)) = v
            && !i.value_type.matches_type(v)
        {
            // It is a dynamic error [ERR D4] if the field of an operator accessing an entry does not have
            // the same type as the value of the entry.
            return Err(Error::Runtime(format!(
                "field {} has wrong value type in context",
                i.name
            ))); // [ERR D4]
        }
        Ok(v)
    }
}

struct MessageEncoder<'a, M> {
    ctx: EncoderContext<'a>,
    msg: &'a mut M,
}

impl<'a, M: MessageVisitor> MessageEncoder<'a, M> {
    fn new(e: &'a mut Encoder, msg: &'a mut M) -> Self {
        Self {
            ctx: EncoderContext::new(e),
            msg,
        }
    }

    fn with_segment(
        &mut self,
        buf: &mut Vec<u8>,
        f: impl FnOnce(&mut Self, &mut Vec<u8>) -> Result<()>,
    ) -> Result<()> {
        EncoderContext::with_segment(self, |s| &mut s.ctx, buf, f)
    }

    // Encode a template to the buffer.
    fn encode_template(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        let definitions = self.ctx.definitions;
        let template = definitions.template_by_name(&self.msg.get_template_name()?)?;
        // presence map + template_id + instructions
        self.with_segment(buf, |ctx, buf| {
            ctx.encode_template_body(buf, template, &definitions.template_id_instruction)
        })
    }

    // Encode template id and template instructions.
    // `tid_instruction` is the template id instruction of the current place in the template.
    fn encode_template_body(
        &mut self,
        buf: &mut Vec<u8>,
        template: &Template,
        tid_instruction: &Instruction,
    ) -> Result<()> {
        // The template id is stored in the dictionary of the enclosing template.
        tid_instruction.inject(&mut self.ctx, buf, Some(&Value::UInt32(template.id)))?;
        self.ctx.template_id.push(template.id);

        let has_type_ref = self.ctx.switch_type_ref(&template.type_ref);

        self.encode_instructions(buf, &template.instructions)?;

        if has_type_ref {
            self.ctx.restore_type_ref();
        }

        self.ctx.template_id.pop();
        Ok(())
    }

    fn encode_instructions(
        &mut self,
        buf: &mut Vec<u8>,
        instructions: &[Instruction],
    ) -> Result<()> {
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

    fn encode_field(&mut self, buf: &mut Vec<u8>, instruction: &Instruction) -> Result<()> {
        let value = self
            .msg
            .get_value(&instruction.name, &instruction.value_type)?;
        instruction.inject(&mut self.ctx, buf, value.as_deref())
    }

    fn encode_segment(&mut self, buf: &mut Vec<u8>, instructions: &[Instruction]) -> Result<()> {
        self.with_segment(buf, |ctx, buf| ctx.encode_instructions(buf, instructions))
    }

    fn encode_group(&mut self, buf: &mut Vec<u8>, instruction: &Instruction) -> Result<()> {
        if !self.msg.select_group(&instruction.name)? {
            return if instruction.is_optional() {
                self.ctx.pmap_set_next_bit(false);
                Ok(())
            } else {
                Err(Error::Dynamic(format!(
                    "Missing mandatory group: {}",
                    instruction.name
                )))
            };
        }
        if instruction.is_optional() {
            self.ctx.pmap_set_next_bit(true);
        }

        let has_type_ref = self.ctx.switch_type_ref(&instruction.type_ref);

        if instruction.has_pmap.get() {
            self.encode_segment(buf, &instruction.instructions)?;
        } else {
            self.encode_instructions(buf, &instruction.instructions)?;
        }

        if has_type_ref {
            self.ctx.restore_type_ref();
        }

        self.msg.release_group()
    }

    fn encode_sequence(&mut self, buf: &mut Vec<u8>, instruction: &Instruction) -> Result<()> {
        let length = self.msg.select_sequence(&instruction.name)?;
        let length_instruction = instruction.instructions.first().unwrap();

        let has_type_ref = self.ctx.switch_type_ref(&instruction.type_ref);
        match length {
            None => {
                if instruction.is_optional() {
                    length_instruction.inject(&mut self.ctx, buf, None)?;
                } else {
                    return Err(Error::Dynamic(format!(
                        "Missing mandatory sequence: {}",
                        instruction.name
                    )));
                }
            }
            Some(length) => {
                let length_value = Value::UInt32(u32::try_from(length).map_err(|_| {
                    Error::Dynamic(format!("Sequence {} is too long", instruction.name))
                })?);
                length_instruction.inject(&mut self.ctx, buf, Some(&length_value))?;
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
        if has_type_ref {
            self.ctx.restore_type_ref();
        }

        Ok(())
    }

    fn encode_template_reference(
        &mut self,
        buf: &mut Vec<u8>,
        instruction: &Instruction,
    ) -> Result<()> {
        let definitions = self.ctx.definitions;

        if instruction.name.is_empty() {
            // is dynamic?
            let template_name = self
                .msg
                .select_template_ref(&instruction.name, true)?
                .ok_or_else(|| {
                    Error::Dynamic("Missing mandatory dynamic template reference".to_string())
                })?;
            let template = definitions.template_by_name(&template_name)?;
            self.with_segment(buf, |ctx, buf| {
                ctx.encode_template_body(buf, template, &instruction.instructions[0])
            })?;
        } else {
            self.msg.select_template_ref(&instruction.name, false)?;
            let template = definitions.static_template_ref(instruction);

            let has_type_ref = self.ctx.switch_type_ref(&template.type_ref);

            self.encode_instructions(buf, &template.instructions)?;

            if has_type_ref {
                self.ctx.restore_type_ref();
            }
        }
        self.msg.release_template_ref()
    }
}
