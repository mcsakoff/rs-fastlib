use std::cell::Cell;
use std::rc::Rc;

use roxmltree::Node;

use crate::{Decimal, Error, Result};
use crate::base::types::{Dictionary, Operator, Presence, TypeRef};
use crate::base::value::{Value, ValueType};
use crate::decoder::state::DecoderState;

const MAX_INT32: i64 = 2147483647;
const MIN_INT32: i64 = -2147483648;
const MAX_UINT32: u64 = 4294967295;
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
                nm = "".to_string();
                ky = "".to_string();
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

    pub fn from_node(node: Node) -> Result<Self> {
        let id = node
            .attribute("id")
            .unwrap_or("0")
            .parse::<u32>()?;
        let name = node
            .attribute("name")
            .unwrap_or("");
        let unicode = match node.attribute("charset") {
            Some("unicode") => true,
            Some(charset) => {
                return Err(Error::Static(format!("unknown charset: {charset}")));
            }
            _ => false
        };
        let type_ = ValueType::new_from_tag(node.tag_name().name(), unicode)?;
        match type_ {
            ValueType::Mantissa | ValueType::Exponent | ValueType::Sequence | ValueType::Group | ValueType::TemplateReference => {}
            _ => {
                if id == 0 {
                    return Err(Error::Runtime("instruction must have non-zero 'id' attribute".to_string()));
                }
            }
        }
        match type_ {
            ValueType::Mantissa | ValueType::Exponent | ValueType::Length | ValueType::TemplateReference => {}
            _ => {
                if name.is_empty() {
                    return Err(Error::Runtime("instruction must have 'name' attribute".to_string()));
                }
            }
        }

        let mut instruction = Instruction::new(id, name, type_);
        if let Some(p) = node.attribute("presence") {
            instruction.presence = Presence::from_str(p)?;
        };
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
                for n in node.children() {
                    if !n.is_element() {
                        continue;
                    }
                    let i = Instruction::from_node(n)?;
                    instruction.add_instruction(i);
                }
            }

            ValueType::Sequence => {
                let mut i: usize = 0;
                for n in node.children() {
                    if !n.is_element() {
                        continue;
                    }
                    let mut instr = Instruction::from_node(n)?;
                    if i == 0 {
                        match instr.value_type {
                            ValueType::Length => {
                                if instr.name.is_empty() {
                                    // The name is generated and is unique to the name of the sequence field.
                                    // The name is guaranteed to never collide with a field name explicitly specified
                                    // in a template.
                                    instr.name = format!("{}:length", instruction.name);
                                }
                                // An optional sequence means that the length field is optional.
                                instr.presence = instruction.presence.clone();
                            }
                            // If no <length> element is specified, the length field has an implicit name and no field operator.
                            _ => {
                                let mut length = Instruction::new(
                                    0,
                                    &format!("{}:length", instruction.name),
                                    ValueType::Length,
                                );
                                // An optional sequence means that the length field is optional.
                                length.presence = instruction.presence.clone();
                                instruction.add_instruction(length);
                            }
                        }
                    }
                    instruction.add_instruction(instr);
                    i += 1;
                }
            }

            ValueType::Decimal => {
                // find out what kind of sub-elements we have
                let mut operator: Option<Operator> = None;
                let mut exponent: Option<Instruction> = None;
                let mut mantissa: Option<Instruction> = None;
                let mut initial_value: Option<String> = None;

                for op_node in node.children() {
                    if !op_node.is_element() {
                        continue;
                    }
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
                for operator in node.children() {
                    if !operator.is_element() {
                        continue;
                    }
                    instruction.operator = Operator::new_from_tag(operator.tag_name().name())?;
                    if let Some(s) = operator.attribute("value") {
                        instruction.set_initial_value(s)?; // [ERR S3]
                    }
                    break;
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
                    return Err(Error::Static("constant operator has no initial value".to_string())); // [ERR S4]
                }
            }
            Operator::Default => {
                // The default operator is applicable to all field types.
                // Unless the field has optional presence, it is a static error [ERR S5] if the instruction context has no initial value.
                if !self.is_optional() && self.initial_value.is_none() {
                    return Err(Error::Static("default operator has no initial value".to_string())); // [ERR S5]
                }
            }
            Operator::Increment => {
                // The increment operator is applicable to integer field types.
                match self.value_type {
                    ValueType::UInt32 | ValueType::Int32 | ValueType::UInt64 | ValueType::Int64 |
                    ValueType::Length | ValueType::Exponent | ValueType::Mantissa => {}
                    _ => {
                        return Err(Error::Static(format!("increment operator is not applicable to {} field type", self.value_type.type_str()))); // [ERR S2]
                    }
                }
            }
            Operator::Tail => {
                // The tail operator is applicable to vector field types.
                match self.value_type {
                    ValueType::ASCIIString | ValueType::UnicodeString | ValueType::Bytes => {}
                    _ => {
                        return Err(Error::Static(format!("tail operator is not applicable to {} field type", self.value_type.type_str()))); // [ERR S2]
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
            ValueType::UInt32 | ValueType::Int32 | ValueType::UInt64 | ValueType::Int64 |
            ValueType::Length | ValueType::Exponent | ValueType::Mantissa |
            ValueType::ASCIIString | ValueType::UnicodeString | ValueType::Bytes => {
                self.initial_value = Some(self.value_type.str_to_value(value)?);
                Ok(())
            }
            // If the field is of type decimal, the value resulting from the conversion is normalized.
            ValueType::Decimal => unreachable!(),
            _ => {
                Err(Error::Static(format!("cannot set initial value to {}", self.value_type.type_str())))
            }
        }
    }

    pub(crate) fn extract(&self, s: &mut DecoderState) -> Result<Option<Value>> {
        match self.operator {
            Operator::None => {
                let v = self.read(s)?;
                s.ctx_set(&self, &v);
                Ok(v)
            }

            // The constant operator specifies that the value of a field will always be the same.
            // The value of the field is the initial value. It is a static error [ERR S4] if the instruction context
            // has no initial value. The value of a constant field is never transferred.
            Operator::Constant => {
                let mut v = None;
                if !self.is_optional() || s.pmap_next_bit_set() {
                    v = match &self.initial_value {
                        Some(v) => Some(v.clone()),
                        None => unreachable!(),
                    };
                }
                Ok(v)
            }

            // The default operator specifies that the value of a field is either present in the stream
            // or it will be the initial value. Unless the field has optional presence, it is a static error [ERR S5]
            // if the instruction context has no initial value. If the field has optional presence and no initial value,
            // the field is considered absent when there is no value in the stream.
            Operator::Default => {
                let v: Option<Value>;
                if s.pmap_next_bit_set() {
                    v = self.read(s)?;
                } else {
                    v = match &self.initial_value {
                        Some(v) => Some(v.clone()),
                        None => {
                            if self.is_optional() {
                                None
                            } else {
                                unreachable!()
                            }
                        }
                    }
                }
                Ok(v)
            }

            // The copy operator specifies that the value of a field is optionally present in the stream.
            Operator::Copy => {
                let v: Option<Value>;
                if s.pmap_next_bit_set() {
                    // If the value is present in the stream it becomes the new previous value.
                    v = self.read(s)?;
                    s.ctx_set(&self, &v);
                } else {
                    // When the value is not present in the stream there are three cases depending
                    // on the state of the previous value:
                    v = match s.ctx_get(&self) {
                        Ok(v) => match v {
                            // Assigned: The value of the field is the previous value.
                            Some(prev) => Some(prev.clone()),
                            // Empty: If the field is optional the value is considered absent.
                            // It is a dynamic error [ERR D6] if the field is mandatory.
                            None => {
                                if self.is_optional() {
                                    None
                                } else {
                                    return Err(Error::Runtime("copy operator has no previous value".to_string())); // [ERR D6]
                                }
                            }
                        }
                        // Undefined: The value of the field is the initial value that also becomes the new previous value.
                        // Unless the field has optional presence, it is a dynamic error [ERR D5] if the instruction context has no initial value.
                        // If the field has optional presence and no initial value, the field is considered absent and the state of the previous
                        // value is changed to empty.
                        Err(_) => {
                            let v = match &self.initial_value {
                                Some(i) => Some(i.clone()),
                                None => {
                                    if self.is_optional() {
                                        None
                                    } else {
                                        return Err(Error::Runtime("copy operator has no initial value".to_string())); // [ERR D5]
                                    }
                                }
                            };
                            s.ctx_set(&self, &v);
                            v
                        }
                    }
                }
                Ok(v)
            }

            // The increment operator specifies that the value of a field is optionally present in the stream.
            Operator::Increment => {
                let v: Option<Value>;
                if s.pmap_next_bit_set() {
                    //If the value is present in the stream it becomes the new previous value.
                    v = self.read(s)?;
                    s.ctx_set(&self, &v);
                } else {
                    // When the value is not present in the stream there are three cases depending on the state of the previous value:
                    v = match s.ctx_get(&self) {
                        Ok(v) => match v {
                            // Assigned: the value of the field is the previous value incremented by one.
                            // The incremented value also becomes the new previous value.
                            Some(prev) => {
                                let v = Some(prev.apply_increment()?);
                                s.ctx_set(&self, &v);
                                v
                            }
                            // Empty: the value of the field is empty.
                            // If the field is optional, the value is considered absent.
                            // It is a dynamic error [ERR D6] if the field is mandatory.
                            None => {
                                if self.is_optional() {
                                    None
                                } else {
                                    return Err(Error::Runtime("increment operator has no previous value".to_string())); // [ERR D6]
                                }
                            }
                        }
                        // Undefined: the value of the field is the initial value that also becomes the new previous value.
                        // Unless the field has optional presence, it is a dynamic error [ERR D5] if the instruction context
                        // has no initial value. If the field has optional presence and no initial value, the field is considered
                        // absent and the state of the previous value is changed to empty.
                        Err(_) => {
                            let v = match &self.initial_value {
                                Some(i) => Some(i.clone()),
                                None => {
                                    if self.is_optional() {
                                        None
                                    } else {
                                        return Err(Error::Runtime("increment operator has no initial value".to_string())); // [ERR D5]
                                    }
                                }
                            };
                            s.ctx_set(&self, &v);
                            v
                        }
                    };
                }
                Ok(v)
            }

            // The delta operator specifies that a delta value is present in the stream.
            Operator::Delta => {
                // If the field has optional presence, the delta value can be NULL.
                // In that case the value of the field is considered absent.
                let (delta, aux) = match self.read_delta(s)? {
                    Some(d) => d,
                    None => {
                        return Ok(None)
                    }
                };
                // Otherwise, the field is obtained by combining the delta value with a base value.
                // The base value depends on the state of the previous value in the following way:
                let base = match s.ctx_get(&self) {
                    Ok(v) => match v {
                        // Assigned: the base value is the previous value.
                        Some(prev) => prev.clone(),
                        // Empty: It is a dynamic error [ERR D6] if the previous value is empty.
                        None => {
                            return Err(Error::Runtime("delta operator has no previous value".to_string())); // [ERR D6]
                        }
                    }
                    // Undefined: The base value is the initial value if present in the instruction context.
                    // Otherwise, a type dependant default base value is used.
                    Err(_) => {
                        match &self.initial_value {
                            Some(v) => v.clone(),
                            None => self.value_type.to_default_value()?,
                        }
                    }
                };
                let value = Some(base.apply_delta(delta, aux)?);
                s.ctx_set(&self, &value);
                Ok(value)
            }

            // The tail operator specifies that a tail value is optionally present in the stream.
            Operator::Tail => {
                let value: Option<Value>;
                if s.pmap_next_bit_set() {
                    let tail = match self.read_tail(s)? {
                        // If the field has optional presence, the tail value can be NULL.
                        // In that case the value of the field is considered absent.
                        None => {
                            return Ok(None)
                        }
                        Some(t) => t,
                    };
                    // Otherwise, if the tail value is present, the value of the field is obtained by combining
                    // the tail value with a base value. The base value depends on the state of the previous value:
                    let base = match s.ctx_get(&self) {
                        Ok(v) => match v {
                            // Assigned: the base value is the previous value.
                            Some(prev) => prev.clone(),
                            // Empty: the base value is the initial value if present in the instruction context.
                            // Otherwise, a type dependant default base value is used.
                            None => {
                                match &self.initial_value {
                                    Some(v) => v.clone(),
                                    None => self.value_type.to_default_value()?,
                                }
                            }
                        }
                        // Undefined: the base value is the initial value if present in the instruction context.
                        // Otherwise, a type dependant default base value is used.
                        Err(_) => {
                            match &self.initial_value {
                                Some(v) => v.clone(),
                                None => self.value_type.to_default_value()?,
                            }
                        }
                    };
                    value = Some(base.apply_tail(tail)?);
                    // The combined value becomes the new previous value.
                    s.ctx_set(&self, &value);
                } else {
                    // If the tail value is not present in the stream, the value of the field depends
                    // on the state of the previous value.
                    value = match s.ctx_get(&self) {
                        Ok(v) => match v {
                            // Assigned: the value of the field is the previous value.
                            Some(prev) => Some(prev.clone()),
                            // Empty: the value of the field is empty. If the field is optional the value is considered absent.
                            // It is a dynamic error [ERR D7] if the field is mandatory.
                            None => {
                                if self.is_optional() {
                                    None
                                } else {
                                    return Err(Error::Runtime("tail operator has no previous value".to_string())); // [ERR D7]
                                }
                            }
                        }
                        // Undefined: the value of the field is the initial value that also becomes the new previous value.
                        // Unless the field has optional presence, it is a dynamic error [ERR D6] if the instruction context
                        // has no initial value. If the field has optional presence and no initial value, the field is considered
                        // absent and the state of the previous value is changed to empty.
                        Err(_) => {
                            let value = match &self.initial_value {
                                Some(v) => Some(v.clone()),
                                None => {
                                    if self.is_optional() {
                                        None
                                    } else {
                                        return Err(Error::Runtime("tail operator has no initial value".to_string())); // [ERR D6]
                                    }
                                }
                            };
                            s.ctx_set(&self, &value);
                            value
                        }
                    };
                }
                Ok(value)
            }
        }
    }

    fn read(&self, s: &mut DecoderState) -> Result<Option<Value>> {
        match self.value_type {
            ValueType::UInt32 | ValueType::Length => {
                match self.read_uint32(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::UInt32(v))),
                }
            }
            ValueType::UInt64 => {
                match self.read_uint64(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::UInt64(v))),
                }
            }
            ValueType::Int32 => {
                match self.read_int32(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::Int32(v))),
                }
            }
            ValueType::Int64 => {
                match self.read_int64(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::Int64(v))),
                }
            }
            ValueType::ASCIIString => {
                match self.read_ascii_string(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::ASCIIString(v))),
                }
            }
            ValueType::UnicodeString => {
                match self.read_unicode_string(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::UnicodeString(v))),
                }
            }
            ValueType::Bytes => {
                match self.read_bytes(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::Bytes(v))),
                }
            }
            // A scaled number is represented as a Signed Integer exponent followed by a Signed Integer mantissa.
            ValueType::Decimal => {
                let (exponent, mantissa) = match self.read_decimal_components(s)? {
                    Some((e, m)) => (e, m),
                    None => {
                        return Ok(None)
                    }
                };
                Ok(Some(Value::Decimal(Decimal::new(exponent, mantissa))))
            }
            ValueType::Exponent => {
                match self.read_exponent(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::Int32(v))),
                }
            }
            ValueType::Mantissa => {
                match self.read_int64(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some(Value::Int64(v))),
                }
            }
            _ => unreachable!()
        }
    }

    fn read_uint32(&self, s: &mut DecoderState) -> Result<Option<u32>> {
        if self.is_nullable() {
            match s.rdr.read_uint_nullable()? {
                None => Ok(None),
                Some(v) => {
                    if v > MAX_UINT32 {
                        return Err(Error::Runtime(format!("uInt32 value is out of range: {}", v)));
                    }
                    Ok(Some(v as u32))
                }
            }
        } else {
            let v = s.rdr.read_uint()?;
            if v > MAX_UINT32 {
                return Err(Error::Runtime(format!("uInt32 value is out of range: {}", v)));
            }
            Ok(Some(v as u32))
        }
    }

    fn read_uint64(&self, s: &mut DecoderState) -> Result<Option<u64>> {
        if self.is_nullable() {
            Ok(s.rdr.read_uint_nullable()?)
        } else {
            Ok(Some(s.rdr.read_uint()?))
        }
    }

    fn read_int32(&self, s: &mut DecoderState) -> Result<Option<i32>> {
        if self.is_nullable() {
            match s.rdr.read_int_nullable()? {
                None => Ok(None),
                Some(v) => {
                    if v < MIN_INT32 || v > MAX_INT32 {
                        return Err(Error::Runtime(format!("uInt32 value is out of range: {}", v))); // [ERR D2]
                    }
                    Ok(Some(v as i32))
                }
            }
        } else {
            let v = s.rdr.read_int()?;
            if v < MIN_INT32 || v > MAX_INT32 {
                return Err(Error::Runtime(format!("uInt32 value is out of range: {}", v))); // [ERR D2]
            }
            Ok(Some(v as i32))
        }
    }

    fn read_int64(&self, s: &mut DecoderState) -> Result<Option<i64>> {
        if self.is_nullable() {
            Ok(s.rdr.read_int_nullable()?)
        } else {
            Ok(Some(s.rdr.read_int()?))
        }
    }

    fn read_ascii_string(&self, s: &mut DecoderState) -> Result<Option<String>> {
        if self.is_nullable() {
            Ok(s.rdr.read_ascii_string_nullable()?)
        } else {
            Ok(Some(s.rdr.read_ascii_string()?))
        }
    }

    fn read_unicode_string(&self, s: &mut DecoderState) -> Result<Option<String>> {
        if self.is_nullable() {
            Ok(s.rdr.read_unicode_string_nullable()?)
        } else {
            Ok(Some(s.rdr.read_unicode_string()?))
        }
    }

    fn read_bytes(&self, s: &mut DecoderState) -> Result<Option<Vec<u8>>> {
        if self.is_nullable() {
            Ok(s.rdr.read_bytes_nullable()?)
        } else {
            Ok(Some(s.rdr.read_bytes()?))
        }
    }

    // The delta operator specifies that a delta value is present in the stream.
    // If the field has optional presence, the delta value can be NULL. In that case the value of the field
    // is considered absent. Otherwise, the field is obtained by combining the delta value with a base value.
    fn read_delta(&self, s: &mut DecoderState) -> Result<Option<(Value, i32)>> {
        match self.value_type {
            ValueType::UInt32 | ValueType::Int32 | ValueType::UInt64 | ValueType::Int64 |
            ValueType::Length | ValueType::Exponent | ValueType::Mantissa => {
                match self.read_int64(s)? {
                    None => Ok(None),
                    Some(v) => Ok(Some((Value::Int64(v), 0))),
                }
            }
            ValueType::ASCIIString | ValueType::UnicodeString | ValueType::Bytes => {
                match self.read_int32(s)? {
                    None => Ok(None),
                    Some(sub) => {
                        match self.value_type {
                            ValueType::ASCIIString => {
                                let diff = self.read_ascii_string(s)?.unwrap();
                                Ok(Some((Value::ASCIIString(diff), sub)))
                            }
                            ValueType::UnicodeString | ValueType::Bytes => {
                                let diff = self.read_bytes(s)?.unwrap();
                                Ok(Some((Value::Bytes(diff), sub)))
                            }
                            _ => unreachable!()
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn read_tail(&self, s: &mut DecoderState) -> Result<Option<Value>> {
        match self.value_type {
            ValueType::ASCIIString => {
                let tail = self.read_ascii_string(s)?.unwrap();
                Ok(Some(Value::ASCIIString(tail)))
            }
            ValueType::UnicodeString | ValueType::Bytes => {
                let tail = self.read_bytes(s)?.unwrap();
                Ok(Some(Value::Bytes(tail)))
            }
            _ => unreachable!()
        }
    }

    fn read_decimal_components(&self, s: &mut DecoderState) -> Result<Option<(i32, i64)>> {
        let exponent = self.instructions
            .get(0)
            .ok_or_else(|| Error::Runtime("exponent field not found".to_string()))?
            .extract(s)?;
        if let None = exponent {
            return Ok(None);
        }
        let mantissa = self.instructions
            .get(1)
            .ok_or_else(|| Error::Runtime("mantissa field not found".to_string()))?
            .extract(s)?;

        if let (Some(Value::Int32(e)), Some(Value::Int64(m))) = (exponent, mantissa) {
            Ok(Some((e, m)))
        } else {
            return Err(Error::Runtime("exponent or mantissa not found".to_string()));
        }
    }

    fn read_exponent(&self, s: &mut DecoderState) -> Result<Option<i32>> {
        let e = match self.read_int32(s)? {
            None => return Ok(None),
            Some(e) => e,
        };
        if e > MAX_EXPONENT || e < MIN_EXPONENT {
            return Err(Error::Dynamic(format!("exponent value is out of range: {}", e))); // [ERR R1]
        }
        Ok(Some(e))
    }
}
