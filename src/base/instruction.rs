use std::cell::Cell;
use std::ops::RangeInclusive;
use std::rc::Rc;

use roxmltree::Node;

use crate::base::types::{Dictionary, Operator, Presence, TypeRef};
use crate::base::value::{Value, ValueType};
use crate::decoder::decoder::DecoderContext;
use crate::encoder::encoder::EncoderContext;
use crate::encoder::writer::Writer;
use crate::{Decimal, Error, MessageFactory, MessageVisitor, Reader, Result};

const MAX_EXPONENT: i32 = 63;
const MIN_EXPONENT: i32 = -63;

/// # Field Instruction
///
/// Each field instruction has a name and a type. The name identifies the corresponding field in the current application type.
/// The type specifies the basic encoding of the field. The optional presence attribute indicates whether the field is mandatory
/// or optional. If the attribute is not specified, the field is mandatory.
///
/// A primitive field, i.e. a field that is not a group or sequence, can have a field operator. The operator specifies an optimization operation for the field.
///
#[derive(Debug)]
pub(crate) struct Instruction {
    pub(crate) id: u32,

    // The name identifies the corresponding field in the current application type.
    pub(crate) name: String,

    // Specifies the basic encoding of the field.
    pub(crate) value_type: ValueType,

    //The optional presence attribute indicates whether the field is mandatory or optional.
    // If the attribute is not specified, the field is mandatory.
    pub(crate) presence: Presence,

    // A primitive field, i.e. a field that is not a group or sequence, can have a field operator.
    // The operator specifies an optimization operation for the field.
    pub(crate) operator: Operator,

    // Initial value specified by the value attribute on the operator element.
    pub(crate) initial_value: Option<Value>,

    // Group, Sequence and Decimal have a list of child instructions.
    pub(crate) instructions: Vec<Instruction>,

    // The dictionary to use for previous values.
    pub(crate) dictionary: Dictionary,

    // The dictionary to use for previous values.
    pub(crate) type_ref: TypeRef,

    // Internal key name for lookup in storage
    pub(crate) key: Rc<str>,

    // For ::Sequence it shows if the instruction needs a pmap.
    // For ::Decimal it shows if any of its subcomponent needs a pmap.
    pub(crate) has_pmap: Cell<bool>,
}

impl Instruction {
    fn new(id: u32, name: &str, type_: ValueType) -> Self {
        let nm: String;
        let ky: String;
        match type_ {
            ValueType::Mantissa | ValueType::Exponent => {
                nm = String::new();
                ky = String::new();
            }
            _ => {
                nm = name.to_string();
                ky = name.to_string();
            }
        }
        Self {
            id,
            name: nm,
            value_type: type_,
            presence: Presence::Mandatory,
            operator: Operator::None,
            initial_value: None,
            instructions: Vec::new(),
            dictionary: Dictionary::Inherit,
            type_ref: TypeRef::Any,
            key: Rc::from(ky),
            has_pmap: Cell::new(false),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn from_node(node: Node) -> Result<Self> {
        let id = node.attribute("id").unwrap_or("0").parse::<u32>()?;
        let name = node.attribute("name").unwrap_or("");
        let unicode = match node.attribute("charset") {
            Some("unicode") => true,
            Some(charset) => {
                return Err(Error::Static(format!("unknown charset: {charset}")));
            }
            _ => false,
        };
        let type_ = ValueType::new_from_tag(node.tag_name().name(), unicode)?;
        match type_ {
            ValueType::Mantissa
            | ValueType::Exponent
            | ValueType::Sequence
            | ValueType::Group
            | ValueType::TemplateReference => {}
            _ => {
                if id == 0 {
                    return Err(Error::Runtime(
                        "instruction must have non-zero 'id' attribute".to_string(),
                    ));
                }
            }
        }
        match type_ {
            ValueType::Mantissa
            | ValueType::Exponent
            | ValueType::Length
            | ValueType::TemplateReference => {}
            _ => {
                if name.is_empty() {
                    return Err(Error::Runtime(
                        "instruction must have 'name' attribute".to_string(),
                    ));
                }
            }
        }

        let mut instruction = Instruction::new(id, name, type_);
        if let Some(p) = node.attribute("presence") {
            instruction.presence = Presence::from_str(p)?;
        }
        if let Some(d) = node.attribute("dictionary") {
            instruction.dictionary = Dictionary::from_str(d);
        }
        if let Some(k) = node.attribute("key") {
            instruction.key = Rc::from(k);
        }
        if let Some(k) = node.attribute("typeRef") {
            instruction.type_ref = TypeRef::from_str(k);
        }

        match instruction.value_type {
            ValueType::TemplateReference => {}

            ValueType::Group => {
                for n in node.children().filter(Node::is_element) {
                    let i = Instruction::from_node(n)?;
                    instruction.add_instruction(i);
                }
            }

            ValueType::Sequence => {
                for (i, c) in node.children().filter(Node::is_element).enumerate() {
                    let mut instr = Instruction::from_node(c)?;
                    if i == 0 {
                        if let ValueType::Length = instr.value_type {
                            if instr.name.is_empty() {
                                // The name is generated and is unique to the name of the sequence field.
                                // The name is guaranteed to never collide with a field name explicitly specified
                                // in a template.
                                instr.name = format!("{}:length", instruction.name);
                            }
                            // An optional sequence means that the length field is optional.
                            instr.presence = instruction.presence;
                        } else {
                            // If no <length> element is specified, the length field has an implicit name and no field operator.

                            let mut length = Instruction::new(
                                0,
                                &format!("{}:length", instruction.name),
                                ValueType::Length,
                            );
                            // An optional sequence means that the length field is optional.
                            length.presence = instruction.presence;
                            instruction.add_instruction(length);
                        }
                    }
                    instruction.add_instruction(instr);
                }
            }

            ValueType::Decimal => {
                // find out what kind of sub-elements we have
                let mut operator: Option<Operator> = None;
                let mut exponent: Option<Instruction> = None;
                let mut mantissa: Option<Instruction> = None;
                let mut initial_value: Option<String> = None;

                for op_node in node.children().filter(Node::is_element) {
                    let op_name = op_node.tag_name().name();
                    match op_name {
                        "exponent" => {
                            exponent = Some(Instruction::from_node(op_node)?);
                        }
                        "mantissa" => {
                            mantissa = Some(Instruction::from_node(op_node)?);
                        }
                        _ => {
                            operator = Some(Operator::new_from_tag(op_name)?);
                            if let Some(v) = op_node.attribute("value") {
                                initial_value = Some(v.to_string());
                            }
                        }
                    }
                }

                let mut op: Operator;
                let mut ex: Instruction;
                let mut mn: Instruction;
                match (operator, exponent, mantissa) {
                    // No elements.
                    (None, None, None) => {
                        op = Operator::None;
                        ex = Instruction::new(0, "exponent", ValueType::Exponent);
                        mn = Instruction::new(0, "mantissa", ValueType::Mantissa);
                    }
                    // Only one element and it is an operation.
                    (Some(o), None, None) => {
                        op = o;
                        ex = Instruction::new(0, "exponent", ValueType::Exponent);
                        mn = Instruction::new(0, "mantissa", ValueType::Mantissa);
                        match o {
                            Operator::Delta | Operator::Increment => {
                                op = Operator::None;
                                ex.operator = o;
                                mn.operator = o;
                            }
                            _ => {}
                        }
                        if let Some(v) = initial_value {
                            let d = Decimal::from_string(&v)?; // [ERR S3]
                            ex.initial_value = Some(Value::Int32(d.exponent));
                            mn.initial_value = Some(Value::Int64(d.mantissa));
                        }
                    }
                    // Elements are decimal subcomponents.
                    (None, Some(e), Some(m)) => {
                        op = Operator::None;
                        ex = e;
                        mn = m;
                    }
                    _ => {
                        return Err(Error::Static("invalid decimal elements".to_string()));
                    }
                }
                // Set proper presence flag.
                ex.presence = instruction.presence;
                mn.presence = Presence::Mandatory;
                // Set proper storage keys if it is not set explicitly with 'key' attribute.
                if ex.key.is_empty() {
                    ex.key = Rc::from(format!("{}:exponent", &instruction.key));
                }
                if mn.key.is_empty() {
                    mn.key = Rc::from(format!("{}:mantissa", &instruction.key));
                }
                instruction.operator = op;
                // Put subcomponents into instruction.
                instruction.add_instruction(ex);
                instruction.add_instruction(mn);
            }

            _ => {
                if let Some(operator) = node.children().find(Node::is_element) {
                    instruction.operator = Operator::new_from_tag(operator.tag_name().name())?;
                    if let Some(s) = operator.attribute("value") {
                        instruction.set_initial_value(s)?; // [ERR S3]
                    }
                }
            }
        }
        instruction.check_is_valid()?;
        Ok(instruction)
    }

    pub(crate) fn check_is_valid(&self) -> Result<()> {
        // Not all operators are applicable to all field types.
        // It is a static error [ERR S2] if an operator is specified for a field type for which it is not applicable.
        match self.operator {
            Operator::None | Operator::Copy | Operator::Delta => {
                // The copy and delta operators are applicable to all field types.
            }
            Operator::Constant => {
                // The constant operator is applicable to all field types.
                // It is a static error [ERR S4] if the instruction context has no initial value.
                if self.initial_value.is_none() {
                    return Err(Error::Static(
                        "constant operator has no initial value".to_string(),
                    )); // [ERR S4]
                }
            }
            Operator::Default => {
                // The default operator is applicable to all field types.
                // Unless the field has optional presence, it is a static error [ERR S5] if the instruction context has no initial value.
                if !self.is_optional() && self.initial_value.is_none() {
                    return Err(Error::Static(
                        "default operator has no initial value".to_string(),
                    )); // [ERR S5]
                }
            }
            Operator::Increment => {
                // The increment operator is applicable to integer field types.
                match self.value_type {
                    ValueType::UInt32
                    | ValueType::Int32
                    | ValueType::UInt64
                    | ValueType::Int64
                    | ValueType::Length
                    | ValueType::Exponent
                    | ValueType::Mantissa => {}
                    _ => {
                        return Err(Error::Static(format!(
                            "increment operator is not applicable to {} field type",
                            self.value_type.type_str()
                        ))); // [ERR S2]
                    }
                }
            }
            Operator::Tail => {
                // The tail operator is applicable to vector field types.
                match self.value_type {
                    ValueType::ASCIIString | ValueType::UnicodeString | ValueType::Bytes => {}
                    _ => {
                        return Err(Error::Static(format!(
                            "tail operator is not applicable to {} field type",
                            self.value_type.type_str()
                        ))); // [ERR S2]
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn is_optional(&self) -> bool {
        match self.presence {
            Presence::Mandatory => false,
            Presence::Optional => true,
        }
    }

    // Each field has a type that has a nullability property.
    // If a type is nullable, there is a special representation of a NULL value.
    // When a type is non-nullable, no representation for NULL is reserved.
    pub(crate) fn is_nullable(&self) -> bool {
        match self.operator {
            Operator::Constant => false,
            _ => self.is_optional(),
        }
    }

    // The value is a string of Unicode characters. This value is converted to the type of the field as defined
    // in the Converting from String section. The possible dynamic and reportable errors that may occur during
    // conversion are treated as static errors [ERR S3] when interpreting the initial value.
    fn set_initial_value(&mut self, value: &str) -> Result<()> {
        match self.value_type {
            ValueType::UInt32
            | ValueType::Int32
            | ValueType::UInt64
            | ValueType::Int64
            | ValueType::Length
            | ValueType::Exponent
            | ValueType::Mantissa
            | ValueType::ASCIIString
            | ValueType::UnicodeString
            | ValueType::Bytes => {
                self.initial_value = Some(self.value_type.str_to_value(value)?);
                Ok(())
            }
            // If the field is of type decimal, the value resulting from the conversion is normalized.
            ValueType::Decimal => unreachable!(),
            _ => Err(Error::Static(format!(
                "cannot set initial value to {}",
                self.value_type.type_str()
            ))),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn extract<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<Value>>
    where
        R: Reader,
        M: MessageFactory,
    {
        match self.operator {
            Operator::None => Ok(self.read(s)?),

            // The constant operator specifies that the value of a field will always be the same.
            // The value of the field is the initial value. It is a static error [ERR S4] if the instruction context
            // has no initial value. The value of a constant field is never transferred.
            Operator::Constant => {
                let v = if !self.is_optional() || s.pmap_next_bit_set() {
                    match &self.initial_value {
                        Some(v) => Some(v.clone()),
                        None => unreachable!(),
                    }
                } else {
                    None
                };
                Ok(v)
            }

            // The default operator specifies that the value of a field is either present in the stream
            // or it will be the initial value. Unless the field has optional presence, it is a static error [ERR S5]
            // if the instruction context has no initial value. If the field has optional presence and no initial value,
            // the field is considered absent when there is no value in the stream.
            Operator::Default => {
                if s.pmap_next_bit_set() {
                    Ok(self.read(s)?)
                } else {
                    if self.is_nullable() && !self.is_optional() {
                        return Err(Error::Runtime(
                            "default operator has no default value".to_string(),
                        )); // [ERR D6])
                    }
                    Ok(self.initial_value.clone())
                }
            }

            // The copy operator specifies that the value of a field is optionally present in the stream.
            Operator::Copy => {
                if s.pmap_next_bit_set() {
                    // If the value is present in the stream it becomes the new previous value.
                    let v = self.read(s)?;
                    s.ctx_set(self, v.clone());
                    return Ok(v);
                }

                // When the value is not present in the stream there are three cases depending
                // on the state of the previous value:
                let Some(v) = s.ctx_get(self)? else {
                    // Undefined: The value of the field is the initial value that also becomes the new previous value.
                    // Unless the field has optional presence, it is a dynamic error [ERR D5] if the instruction context has no initial value.
                    // If the field has optional presence and no initial value, the field is considered absent and the state of the previous
                    // value is changed to empty.
                    if self.initial_value.is_none() && !self.is_optional() {
                        return Err(Error::Runtime(
                            "copy operator has no initial value".to_string(),
                        )); // [ERR D5]
                    }

                    s.ctx_set(self, self.initial_value.clone());
                    return Ok(self.initial_value.clone());
                };

                // Empty: If the field is optional the value is considered absent.
                // It is a dynamic error [ERR D6] if the field is mandatory.
                if v.is_none() && !self.is_optional() {
                    return Err(Error::Runtime(
                        "copy operator has no previous value".to_string(),
                    )); // [ERR D6]
                }

                // Assigned: The value of the field is the previous value.
                Ok(v)
            }

            // The increment operator specifies that the value of a field is optionally present in the stream.
            Operator::Increment => {
                if s.pmap_next_bit_set() {
                    //If the value is present in the stream it becomes the new previous value.
                    let v = self.read(s)?;
                    s.ctx_set(self, v.clone());
                    return Ok(v);
                }
                // When the value is not present in the stream there are three cases depending on the state of the previous value:
                let Some(v) = s.ctx_get(self)? else {
                    // Undefined: the value of the field is the initial value that also becomes the new previous value.
                    // Unless the field has optional presence, it is a dynamic error [ERR D5] if the instruction context
                    // has no initial value. If the field has optional presence and no initial value, the field is considered
                    // absent and the state of the previous value is changed to empty.
                    if !self.is_optional() && self.initial_value.is_none() {
                        return Err(Error::Runtime(
                            "increment operator has no initial value".to_string(),
                        )); // [ERR D5]
                    }
                    s.ctx_set(self, self.initial_value.clone());
                    return Ok(self.initial_value.clone());
                };

                // Assigned: the value of the field is the previous value incremented by one.
                // The incremented value also becomes the new previous value.
                let Some(prev) = v else {
                    // Empty: the value of the field is empty.
                    // If the field is optional, the value is considered absent.
                    // It is a dynamic error [ERR D6] if the field is mandatory.
                    if !self.is_optional() {
                        return Err(Error::Runtime(
                            "increment operator has no previous value".to_string(),
                        ));
                    }
                    return Ok(None);
                };

                let v = Some(prev.apply_increment()?);
                s.ctx_set(self, v.clone());
                Ok(v)
            }

            // The delta operator specifies that a delta value is present in the stream.
            Operator::Delta => {
                // If the field has optional presence, the delta value can be NULL.
                // In that case the value of the field is considered absent.
                let Some((delta, aux)) = self.read_delta(s)? else {
                    return Ok(None);
                };
                // Otherwise, the field is obtained by combining the delta value with a base value.
                // The base value depends on the state of the previous value in the following way:
                let base = match s.ctx_get(self)? {
                    Some(v) => match v {
                        // Assigned: the base value is the previous value.
                        Some(prev) => prev.clone(),
                        // Empty: It is a dynamic error [ERR D6] if the previous value is empty.
                        None => {
                            return Err(Error::Runtime(
                                "delta operator has no previous value".to_string(),
                            )); // [ERR D6]
                        }
                    },
                    // Undefined: The base value is the initial value if present in the instruction context.
                    // Otherwise, a type dependant default base value is used.
                    None => match &self.initial_value {
                        Some(v) => v.clone(),
                        None => self.value_type.to_default_value()?,
                    },
                };
                let value = Some(base.apply_delta(&delta, aux)?);
                s.ctx_set(self, value.clone());
                Ok(value)
            }

            // The tail operator specifies that a tail value is optionally present in the stream.
            Operator::Tail => {
                if s.pmap_next_bit_set() {
                    let Some(tail) = self.read_tail(s)? else {
                        // If the field has optional presence, the tail value can be NULL.
                        // In that case the value of the field is considered absent.
                        return if self.is_optional() {
                            Ok(None)
                        } else {
                            Err(Error::Runtime(
                                "tail operator has no previous value".to_string(),
                            )) // [ERR D7]
                        };
                    };
                    // Otherwise, if the tail value is present, the value of the field is obtained by combining
                    // the tail value with a base value. The base value depends on the state of the previous value:
                    #[allow(clippy::collapsible_match)]
                    let base = match s.ctx_get(self)? {
                        Some(v) => match v {
                            // Assigned: the base value is the previous value.
                            Some(prev) => prev.clone(),
                            // Empty: the base value is the initial value if present in the instruction context.
                            // Otherwise, a type dependant default base value is used.
                            None => match &self.initial_value {
                                Some(v) => v.clone(),
                                None => self.value_type.to_default_value()?,
                            },
                        },
                        // Undefined: the base value is the initial value if present in the instruction context.
                        // Otherwise, a type dependant default base value is used.
                        None => match &self.initial_value {
                            Some(v) => v.clone(),
                            None => self.value_type.to_default_value()?,
                        },
                    };
                    let value = Some(base.apply_tail(&tail)?);
                    // The combined value becomes the new previous value.
                    s.ctx_set(self, value.clone());
                    return Ok(value);
                }

                // If the tail value is not present in the stream, the value of the field depends
                // on the state of the previous value.
                let Some(v) = s.ctx_get(self)? else {
                    // Undefined: the value of the field is the initial value that also becomes the new previous value.
                    // Unless the field has optional presence, it is a dynamic error [ERR D6] if the instruction context
                    // has no initial value. If the field has optional presence and no initial value, the field is considered
                    // absent and the state of the previous value is changed to empty.
                    if self.initial_value.is_none() && !self.is_optional() {
                        return Err(Error::Runtime(
                            "tail operator has no initial value".to_string(),
                        )); // [ERR D6]
                    }
                    s.ctx_set(self, self.initial_value.clone());
                    return Ok(self.initial_value.clone());
                };

                // Empty: the value of the field is empty. If the field is optional the value is considered absent.
                // It is a dynamic error [ERR D7] if the field is mandatory.
                if v.is_none() && !self.is_optional() {
                    return Err(Error::Runtime(
                        "tail operator has no previous value".to_string(),
                    )); // [ERR D7]
                }
                Ok(v)
            }
        }
    }

    fn read<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<Value>>
    where
        R: Reader,
        M: MessageFactory,
    {
        match self.value_type {
            ValueType::UInt32 | ValueType::Length => match self.read_uint32(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::UInt32(v))),
            },
            ValueType::UInt64 => match self.read_uint64(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::UInt64(v))),
            },
            ValueType::Int32 => match self.read_int32(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::Int32(v))),
            },
            ValueType::Int64 | ValueType::Mantissa => match self.read_int64(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::Int64(v))),
            },
            ValueType::ASCIIString => match self.read_ascii_string(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::ASCIIString(v))),
            },
            ValueType::UnicodeString => match self.read_unicode_string(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::UnicodeString(v))),
            },
            ValueType::Bytes => match self.read_bytes(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::Bytes(v))),
            },
            // A scaled number is represented as a Signed Integer exponent followed by a Signed Integer mantissa.
            ValueType::Decimal => {
                let Some((exponent, mantissa)) = self.read_decimal_components(s)? else {
                    return Ok(None);
                };
                Ok(Some(Value::Decimal(Decimal::new(exponent, mantissa))))
            }
            ValueType::Exponent => match self.read_exponent(s)? {
                None => Ok(None),
                Some(v) => Ok(Some(Value::Int32(v))),
            },
            _ => unreachable!(),
        }
    }

    fn read_uint32<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<u32>>
    where
        R: Reader,
        M: MessageFactory,
    {
        if self.is_nullable() {
            match s.rdr.read_uint_nullable()? {
                None => Ok(None),
                Some(v) => {
                    if v > u64::from(u32::MAX) {
                        return Err(Error::Runtime(format!("uInt32 value is out of range: {v}")));
                    }
                    Ok(Some(v as u32))
                }
            }
        } else {
            let v = s.rdr.read_uint()?;
            if v > u64::from(u32::MAX) {
                return Err(Error::Runtime(format!("uInt32 value is out of range: {v}")));
            }
            Ok(Some(v as u32))
        }
    }

    fn read_uint64<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<u64>>
    where
        R: Reader,
        M: MessageFactory,
    {
        if self.is_nullable() {
            Ok(s.rdr.read_uint_nullable()?)
        } else {
            Ok(Some(s.rdr.read_uint()?))
        }
    }

    fn read_int32<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<i32>>
    where
        R: Reader,
        M: MessageFactory,
    {
        const INT32_RANGE: RangeInclusive<i64> = (i32::MIN as i64)..=(i32::MAX as i64);
        if self.is_nullable() {
            match s.rdr.read_int_nullable()? {
                None => Ok(None),
                Some(v) => {
                    if !INT32_RANGE.contains(&v) {
                        return Err(Error::Runtime(format!("Int32 value is out of range: {v}"))); // [ERR D2]
                    }
                    Ok(Some(v as i32))
                }
            }
        } else {
            let v = s.rdr.read_int()?;
            if !INT32_RANGE.contains(&v) {
                return Err(Error::Runtime(format!("Int32 value is out of range: {v}"))); // [ERR D2]
            }
            Ok(Some(v as i32))
        }
    }

    fn read_int64<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<i64>>
    where
        R: Reader,
        M: MessageFactory,
    {
        if self.is_nullable() {
            Ok(s.rdr.read_int_nullable()?)
        } else {
            Ok(Some(s.rdr.read_int()?))
        }
    }

    fn read_ascii_string<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<String>>
    where
        R: Reader,
        M: MessageFactory,
    {
        if self.is_nullable() {
            Ok(s.rdr.read_ascii_string_nullable()?)
        } else {
            Ok(Some(s.rdr.read_ascii_string()?))
        }
    }

    fn read_unicode_string<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<String>>
    where
        R: Reader,
        M: MessageFactory,
    {
        if self.is_nullable() {
            Ok(s.rdr.read_unicode_string_nullable()?)
        } else {
            Ok(Some(s.rdr.read_unicode_string()?))
        }
    }

    fn read_bytes<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<Vec<u8>>>
    where
        R: Reader,
        M: MessageFactory,
    {
        if self.is_nullable() {
            Ok(s.rdr.read_bytes_nullable()?)
        } else {
            Ok(Some(s.rdr.read_bytes()?))
        }
    }

    // The delta operator specifies that a delta value is present in the stream.
    // If the field has optional presence, the delta value can be NULL. In that case the value of the field
    // is considered absent. Otherwise, the field is obtained by combining the delta value with a base value.
    fn read_delta<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<(Value, i32)>>
    where
        R: Reader,
        M: MessageFactory,
    {
        match self.value_type {
            ValueType::UInt32
            | ValueType::Int32
            | ValueType::UInt64
            | ValueType::Int64
            | ValueType::Length
            | ValueType::Exponent
            | ValueType::Mantissa => match self.read_int64(s)? {
                None => Ok(None),
                Some(v) => Ok(Some((Value::Int64(v), 0))),
            },
            ValueType::ASCIIString | ValueType::UnicodeString | ValueType::Bytes => {
                match self.read_int32(s)? {
                    None => Ok(None),
                    Some(sub) => match self.value_type {
                        ValueType::ASCIIString => {
                            let diff = self.read_ascii_string(s)?.unwrap();
                            Ok(Some((Value::ASCIIString(diff), sub)))
                        }
                        ValueType::UnicodeString | ValueType::Bytes => {
                            let diff = self.read_bytes(s)?.unwrap();
                            Ok(Some((Value::Bytes(diff), sub)))
                        }
                        _ => unreachable!(),
                    },
                }
            }
            _ => unreachable!(),
        }
    }

    fn read_tail<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<Value>>
    where
        R: Reader,
        M: MessageFactory,
    {
        match self.value_type {
            ValueType::ASCIIString => Ok(self.read_ascii_string(s)?.map(Value::ASCIIString)),
            ValueType::UnicodeString | ValueType::Bytes => {
                Ok(self.read_bytes(s)?.map(Value::Bytes))
            }
            _ => unreachable!(),
        }
    }

    fn read_decimal_components<R, M>(
        &self,
        s: &mut DecoderContext<R, M>,
    ) -> Result<Option<(i32, i64)>>
    where
        R: Reader,
        M: MessageFactory,
    {
        let exponent = self
            .instructions
            .first()
            .ok_or_else(|| Error::Runtime("exponent field not found".to_string()))?
            .extract(s)?;
        if exponent.is_none() {
            return Ok(None);
        }
        let mantissa = self
            .instructions
            .get(1)
            .ok_or_else(|| Error::Runtime("mantissa field not found".to_string()))?
            .extract(s)?;

        if let (Some(Value::Int32(e)), Some(Value::Int64(m))) = (exponent, mantissa) {
            Ok(Some((e, m)))
        } else {
            Err(Error::Runtime("exponent or mantissa not found".to_string()))
        }
    }

    fn read_exponent<R, M>(&self, s: &mut DecoderContext<R, M>) -> Result<Option<i32>>
    where
        R: Reader,
        M: MessageFactory,
    {
        let Some(e) = self.read_int32(s)? else {
            return Ok(None);
        };
        if !(MIN_EXPONENT..=MAX_EXPONENT).contains(&e) {
            return Err(Error::Dynamic(format!(
                "exponent value is out of range: {e}"
            ))); // [ERR R1]
        }
        Ok(Some(e))
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn inject<W, M>(
        &self,
        s: &mut EncoderContext<W, M>,
        buf: &mut impl Writer,
        value: Option<Value>,
    ) -> Result<()>
    where
        W: Writer,
        M: MessageVisitor,
    {
        if value.is_none() && !self.is_optional() {
            return Err(Error::Runtime(format!(
                "mandatory field {} has no value",
                self.name
            )));
        }
        match self.operator {
            Operator::None => self.write(buf, s, value),
            Operator::Constant => {
                if value.is_some() && self.initial_value != value {
                    return Err(Error::Runtime(format!(
                        "constant field {} has wrong value",
                        self.name
                    )));
                }
                if self.is_optional() {
                    s.pmap_set_next_bit(value.is_some());
                }
                Ok(())
            }
            Operator::Default => {
                if self.initial_value == value {
                    s.pmap_set_next_bit(false);
                    Ok(())
                } else {
                    s.pmap_set_next_bit(true);
                    self.write(buf, s, value)
                }
            }
            Operator::Copy => {
                let prev_value = if let Some(v) = s.ctx_get(self)? {
                    v
                } else {
                    s.ctx_set(self, self.initial_value.clone());
                    self.initial_value.clone()
                };
                if prev_value == value {
                    s.pmap_set_next_bit(false);
                    Ok(())
                } else {
                    s.pmap_set_next_bit(true);
                    s.ctx_set(self, value.clone());
                    self.write(buf, s, value)
                }
            }
            Operator::Increment => {
                let prev_value = s
                    .ctx_get(self)?
                    .unwrap_or_else(|| self.initial_value.clone());
                let next_value = match prev_value {
                    None => None,
                    Some(v) => Some(v.apply_increment()?),
                };
                s.ctx_set(self, value.clone());
                if next_value == value {
                    s.pmap_set_next_bit(false);
                    Ok(())
                } else {
                    s.pmap_set_next_bit(true);
                    self.write(buf, s, value)
                }
            }
            Operator::Delta => {
                let Some(value) = value else {
                    return self.write_delta(buf, None);
                };

                let base = match s.ctx_get(self)? {
                    Some(v) => match v {
                        Some(v) => v,
                        None => {
                            return Err(Error::Runtime(
                                "delta operator has empty previous value".to_string(),
                            )); // [ERR D6]
                        }
                    },
                    None => match &self.initial_value {
                        Some(v) => v.clone(),
                        None => self.value_type.to_default_value()?,
                    },
                };

                let delta = value.find_delta(&base);
                s.ctx_set(self, Some(value));
                self.write_delta(buf, Some(delta))
            }
            Operator::Tail => {
                let prev_value = s
                    .ctx_get(self)?
                    .unwrap_or_else(|| self.initial_value.clone());
                if prev_value == value {
                    s.pmap_set_next_bit(false);
                    s.ctx_set(self, value);
                    Ok(())
                } else {
                    let tail = match &value {
                        None => None,
                        Some(v) => {
                            s.ctx_set(self, value.clone());
                            let prev = match prev_value {
                                Some(p) => p,
                                None => self.value_type.to_default_value()?,
                            };
                            Some(v.find_tail(&prev)?)
                        }
                    };
                    s.pmap_set_next_bit(true);
                    self.write_tail(buf, tail)
                }
            }
        }
    }

    fn write<W, M>(
        &self,
        buf: &mut impl Writer,
        s: &mut EncoderContext<W, M>,
        value: Option<Value>,
    ) -> Result<()>
    where
        W: Writer,
        M: MessageVisitor,
    {
        match self.value_type {
            ValueType::UInt32 | ValueType::Length => match value {
                None => self.write_uint::<u32>(buf, None),
                Some(Value::UInt32(v)) => self.write_uint(buf, Some(v)),
                _ => Err(Error::Runtime(format!(
                    "Field {} must have UInt32 value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::Int32 => match value {
                None => self.write_int::<i32>(buf, None),
                Some(Value::Int32(v)) => self.write_int(buf, Some(v)),
                _ => Err(Error::Runtime(format!(
                    "Field {} must have Int32 value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::UInt64 => match value {
                None => self.write_uint::<u64>(buf, None),
                Some(Value::UInt64(v)) => self.write_uint(buf, Some(v)),
                _ => Err(Error::Runtime(format!(
                    "Field {} must have UInt64 value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::Int64 | ValueType::Mantissa => match value {
                None => self.write_int::<i64>(buf, None),
                Some(Value::Int64(v)) => self.write_int(buf, Some(v)),
                _ => Err(Error::Runtime(format!(
                    "Field {}:mantissa must have Int64 value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::Exponent => match value {
                None => self.write_exponent(buf, None),
                Some(Value::Int32(v)) => self.write_exponent(buf, Some(v)),
                _ => Err(Error::Runtime(format!(
                    "Field {}:exponent must have Int32 value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::Decimal => match value {
                None => self.write_decimal(buf, s, None),
                Some(Value::Decimal(d)) => self.write_decimal(buf, s, Some(d)),
                _ => Err(Error::Runtime(format!(
                    "Field {} must have Decimal value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::ASCIIString => match value {
                None => self.write_ascii_string(buf, None),
                Some(Value::ASCIIString(v)) => self.write_ascii_string(buf, Some(&v)),
                Some(Value::UnicodeString(v)) => {
                    if v.is_ascii() {
                        self.write_ascii_string(buf, Some(&v))
                    } else {
                        Err(Error::Runtime(format!(
                            "Field {} must be valid ASCII string",
                            self.name
                        )))
                    }
                }
                _ => Err(Error::Runtime(format!(
                    "Field {} must have ASCIIString value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::UnicodeString => match value {
                None => self.write_unicode_string(buf, None),
                Some(Value::UnicodeString(v) | Value::ASCIIString(v)) => {
                    self.write_unicode_string(buf, Some(&v))
                }
                _ => Err(Error::Runtime(format!(
                    "Field {} must have UnicodeString value, got: {:?} instead",
                    self.name, value
                ))),
            },
            ValueType::Bytes => match value {
                None => self.write_bytes(buf, None),
                Some(Value::Bytes(v)) => self.write_bytes(buf, Some(&v)),
                _ => Err(Error::Runtime(format!(
                    "Field {} must have Bytes value, got: {:?} instead",
                    self.name, value
                ))),
            },
            _ => unreachable!(),
        }
    }

    fn write_uint<T>(&self, buf: &mut impl Writer, value: Option<T>) -> Result<()>
    where
        T: Into<u64>,
    {
        let value = value.map(std::convert::Into::into);
        if self.is_nullable() {
            buf.write_uint_nullable(value)
        } else {
            buf.write_uint(value.ok_or_else(|| {
                Error::Runtime(format!("mandatory field {} has no value", self.name))
            })?)
        }
    }

    fn write_int<T>(&self, buf: &mut impl Writer, value: Option<T>) -> Result<()>
    where
        T: Into<i64>,
    {
        let value = value.map(std::convert::Into::into);
        if self.is_nullable() {
            buf.write_int_nullable(value)
        } else {
            buf.write_int(value.ok_or_else(|| {
                Error::Runtime(format!("mandatory field {} has no value", self.name))
            })?)
        }
    }

    fn write_ascii_string(&self, buf: &mut impl Writer, value: Option<&str>) -> Result<()> {
        if self.is_nullable() {
            buf.write_ascii_string_nullable(value)
        } else {
            buf.write_ascii_string(value.ok_or_else(|| {
                Error::Runtime(format!("mandatory field {} has no value", self.name))
            })?)
        }
    }

    fn write_unicode_string(&self, buf: &mut impl Writer, value: Option<&str>) -> Result<()> {
        if self.is_nullable() {
            buf.write_unicode_string_nullable(value)
        } else {
            buf.write_unicode_string(value.ok_or_else(|| {
                Error::Runtime(format!("mandatory field {} has no value", self.name))
            })?)
        }
    }

    fn write_bytes(&self, buf: &mut impl Writer, value: Option<&[u8]>) -> Result<()> {
        if self.is_nullable() {
            buf.write_bytes_nullable(value)
        } else {
            buf.write_bytes(value.ok_or_else(|| {
                Error::Runtime(format!("mandatory field {} has no value", self.name))
            })?)
        }
    }

    fn write_delta(&self, buf: &mut impl Writer, value: Option<(Value, i32)>) -> Result<()> {
        match self.value_type {
            ValueType::UInt32
            | ValueType::Int32
            | ValueType::UInt64
            | ValueType::Int64
            | ValueType::Length
            | ValueType::Exponent
            | ValueType::Mantissa => match value {
                None => self.write_int::<i64>(buf, None),
                Some((Value::Int64(v), _)) => self.write_int(buf, Some(v)),
                Some((v, _)) => Err(Error::Runtime(format!(
                    "{} field's delta must be Int64, got: {:?} instead",
                    self.name, v
                ))),
            },
            ValueType::ASCIIString | ValueType::UnicodeString | ValueType::Bytes => match value {
                None => self.write_int::<i32>(buf, None),
                Some((delta, sub)) => {
                    self.write_int(buf, Some(sub))?;
                    match (&self.value_type, delta) {
                        (ValueType::ASCIIString, Value::ASCIIString(s)) => {
                            self.write_ascii_string(buf, Some(&s))
                        }
                        (ValueType::UnicodeString | ValueType::Bytes, Value::Bytes(b)) => {
                            self.write_bytes(buf, Some(&b))
                        }
                        _ => unreachable!(),
                    }
                }
            },
            _ => unreachable!(),
        }
    }

    fn write_tail(&self, buf: &mut impl Writer, tail: Option<Value>) -> Result<()> {
        match self.value_type {
            ValueType::ASCIIString => match tail {
                None => self.write_ascii_string(buf, None),
                Some(Value::ASCIIString(s)) => self.write_ascii_string(buf, Some(&s)),
                Some(v) => Err(Error::Runtime(format!(
                    "{} field's tail must be ASCIIString, got: {:?} instead",
                    self.name, v
                ))),
            },
            ValueType::UnicodeString | ValueType::Bytes => match tail {
                None => self.write_bytes(buf, None),
                Some(Value::Bytes(b)) => self.write_bytes(buf, Some(&b)),
                Some(v) => Err(Error::Runtime(format!(
                    "{} field's tail must be Bytes, got: {:?} instead",
                    self.name, v
                ))),
            },
            _ => unreachable!(),
        }
    }

    fn write_decimal<W, M>(
        &self,
        buf: &mut impl Writer,
        s: &mut EncoderContext<W, M>,
        value: Option<Decimal>,
    ) -> Result<()>
    where
        W: Writer,
        M: MessageVisitor,
    {
        let (e, m) = match value {
            None => (None, Value::Int64(0)),
            Some(d) => (Some(Value::Int32(d.exponent)), Value::Int64(d.mantissa)),
        };

        let without_exponent = e.is_none();
        // write exponent
        self.instructions
            .first()
            .ok_or_else(|| Error::Runtime("exponent field not found".to_string()))?
            .inject(s, buf, e)?;

        if without_exponent {
            return Ok(());
        }
        // write mantissa
        self.instructions
            .get(1)
            .ok_or_else(|| Error::Runtime("mantissa field not found".to_string()))?
            .inject(s, buf, Some(m))
    }

    fn write_exponent(&self, buf: &mut impl Writer, value: Option<i32>) -> Result<()> {
        if let Some(e) = value
            && !(MIN_EXPONENT..=MAX_EXPONENT).contains(&e)
        {
            return Err(Error::Dynamic(format!(
                "exponent value is out of range: {e}"
            ))); // [ERR R1]
        }
        self.write_int(buf, value)
    }
}
