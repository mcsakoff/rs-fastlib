use std::cmp::min;
use std::fmt::{Display, Formatter};

use crate::{Error, Result};
use crate::utils::bytes::{bytes_delta, bytes_tail, string_delta, string_tail, string_to_bytes};
use crate::base::decimal::Decimal;

/// Represents type of field instruction.
///
/// It can be field instruction (including sequence and group) or template reference instruction.
#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    UInt32,
    Int32,
    UInt64,
    Int64,
    Length,
    Exponent,
    Mantissa,
    Decimal,
    ASCIIString,
    UnicodeString,
    Bytes,
    Sequence,
    Group,
    TemplateReference,
}

impl ValueType {
    pub fn new_from_tag(tag: &str, unicode: bool) -> Result<Self> {
        match tag {
            "uInt32" => Ok(Self::UInt32),
            "int32" => Ok(Self::Int32),
            "uInt64" => Ok(Self::UInt64),
            "int64" => Ok(Self::Int64),
            "length" => Ok(Self::Length),
            "exponent" => Ok(Self::Exponent),
            "mantissa" => Ok(Self::Mantissa),
            "decimal" => Ok(Self::Decimal),
            "string" => {
                if unicode {
                    Ok(Self::UnicodeString)
                } else {
                    Ok(Self::ASCIIString)
                }
            }
            "byteVector" => Ok(Self::Bytes),
            "sequence" => Ok(Self::Sequence),
            "group" => Ok(Self::Group),
            "templateRef" => Ok(Self::TemplateReference),
            _ => Err(Error::Static(format!("Unknown type: {}", tag))),
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            ValueType::UInt32 => "uInt32",
            ValueType::Int32 => "int32",
            ValueType::UInt64 => "uInt64",
            ValueType::Int64 => "int64",
            ValueType::Length => "length",
            ValueType::Exponent => "exponent",
            ValueType::Mantissa => "mantissa",
            ValueType::Decimal => "decimal",
            ValueType::ASCIIString => "string",
            ValueType::UnicodeString => "string",
            ValueType::Bytes => "byteVector",
            ValueType::Sequence => "sequence",
            ValueType::Group => "group",
            ValueType::TemplateReference => "templateRef",
        }
    }

    pub fn to_default_value(&self) -> Result<Value> {
        match self {
            ValueType::UInt32 => Ok(Value::UInt32(0)),
            ValueType::Int32 => Ok(Value::Int32(0)),
            ValueType::UInt64 => Ok(Value::UInt64(0)),
            ValueType::Int64 => Ok(Value::Int64(0)),
            ValueType::Length => Ok(Value::UInt32(0)),
            ValueType::Exponent => Ok(Value::Int32(0)),
            ValueType::Mantissa => Ok(Value::Int64(0)),
            ValueType::Decimal => Ok(Value::Decimal(Decimal::default())),
            ValueType::ASCIIString => Ok(Value::ASCIIString(String::new())),
            ValueType::UnicodeString => Ok(Value::UnicodeString(String::new())),
            ValueType::Bytes => Ok(Value::Bytes(Vec::new())),
            _ => Err(Error::Runtime(format!("{} cannot be converted to value", self.type_str()))),
        }
    }

    pub fn str_to_value(&self, s: &str) -> Result<Value> {
        let mut value = match self {
            ValueType::UInt32 => Value::UInt32(0),
            ValueType::Int32 => Value::Int32(0),
            ValueType::UInt64 => Value::UInt64(0),
            ValueType::Int64 => Value::Int64(0),
            ValueType::Length => Value::UInt32(0),
            ValueType::Exponent => Value::Int32(0),
            ValueType::Mantissa => Value::Int64(0),
            ValueType::Decimal => Value::Decimal(Decimal::default()),
            ValueType::ASCIIString => Value::ASCIIString(String::new()),
            ValueType::UnicodeString => Value::UnicodeString(String::new()),
            ValueType::Bytes => Value::Bytes(Vec::new()),
            _ => return Err(Error::Runtime(format!("{} cannot be converted to value", self.type_str()))),
        };
        value.set_from_string(s)?;
        Ok(value)
    }

    pub fn matches_type(&self, v: &Value) -> bool {
        match (self, v) {
            (ValueType::UInt32, Value::UInt32(_)) => true,
            (ValueType::Int32, Value::Int32(_)) => true,
            (ValueType::UInt64, Value::UInt64(_)) => true,
            (ValueType::Int64, Value::Int64(_)) => true,
            (ValueType::Length, Value::UInt32(_)) => true,
            (ValueType::Exponent, Value::Int32(_)) => true,
            (ValueType::Mantissa, Value::Int64(_)) => true,
            (ValueType::Decimal, Value::Decimal(_)) => true,
            (ValueType::ASCIIString, Value::ASCIIString(_)) => true,
            (ValueType::UnicodeString, Value::UnicodeString(_)) => true,
            (ValueType::Bytes, Value::Bytes(_)) => true,
            _ => false
        }
    }
}


/// Represents current value of a field.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    UInt32(u32),
    Int32(i32),
    UInt64(u64),
    Int64(i64),
    Decimal(Decimal),
    ASCIIString(String),
    UnicodeString(String),
    Bytes(Vec<u8>),
}

impl Value {
    // It is a dynamic error [ERR D11] if a string does not match the syntax.
    pub fn set_from_string(&mut self, s: &str) -> Result<()> {
        match self {
            Value::UInt32(_) => {
                *self = Value::UInt32(s.parse()?);
            }
            Value::Int32(_) => {
                *self = Value::Int32(s.parse()?);
            }
            Value::UInt64(_) => {
                *self = Value::UInt64(s.parse()?);
            }
            Value::Int64(_) => {
                *self = Value::Int64(s.parse()?);
            }
            Value::Decimal(_) => {
                *self = Value::Decimal(Decimal::from_string(s)?);
            }
            Value::ASCIIString(_) => {
                *self = Value::ASCIIString(s.to_string());
            }
            Value::UnicodeString(_) => {
                *self = Value::UnicodeString(s.to_string());
            }
            // The string is interpreted as an even number of hexadecimal digits [0-9A-Fa-f] possibly interleaved
            // with whitespace. The literal is turned into a byte vector by first stripping any whitespace.
            // Then each pair of characters is interpreted as a hexadecimal number representing a single byte.
            Value::Bytes(_) => {
                *self = Value::Bytes(string_to_bytes(s)?);
            }
        }
        Ok(())
    }

    pub fn apply_delta(&self, delta: Value, sub: i32) -> Result<Value> {

        fn sub2index(sub: i32, len: usize) -> Result<(bool, usize)> {
            // A negative subtraction length is used to remove values from the front of the string.
            // Negative zero is used to append values to the front of the string.
            let front: bool;
            let mut i: usize;
            if sub < 0 {
                front = true;
                i = (-sub - 1) as usize;
            } else {
                front = false;
                i = sub as usize;
            }
            if i > len {
                return Err(Error::Dynamic(format!("subtraction length ({i}) is larger than string length ('{len}')")));  // [ERR D7]
            }
            if !front {
                i = len - i;
            }
            Ok((front, i))
        }

        fn bytes_delta(v: &[u8], d: &[u8], sub: i32) -> Result<Vec<u8>> {
            let (front, i) = sub2index(sub, v.len())?;
            let mut b = Vec::with_capacity(v.len() + d.len());
            if front {
                b.extend_from_slice(d);
                b.extend_from_slice(&v[i..]);
            } else {
                b.extend_from_slice(&v[..i]);
                b.extend_from_slice(d);
            }
            Ok(b)
        }

        match (self, &delta) {
            (Value::UInt32(v), Value::Int64(d)) => {
                if *d < 0 {
                    Ok(Value::UInt32(*v - (-*d) as u32))
                } else {
                    Ok(Value::UInt32(*v + *d as u32))
                }
            }
            (Value::Int32(v), Value::Int64(d)) => {
                Ok(Value::Int32(*v + *d as i32))
            }
            (Value::UInt64(v), Value::Int64(d)) => {
                if *d < 0 {
                    Ok(Value::UInt64(*v - (-*d) as u64))
                } else {
                    Ok(Value::UInt64(*v + *d as u64))
                }
            }
            (Value::Int64(v), Value::Int64(d)) => {
                Ok(Value::Int64(*v + *d))
            }
            (Value::ASCIIString(v), Value::ASCIIString(d)) => {
                let (front, i) = sub2index(sub, v.len())?;
                let s;
                if front {
                    s = format!("{}{}", d, v[i..].to_string());
                } else {
                    s = format!("{}{}", v[..i].to_string(), d);
                }
                Ok(Value::ASCIIString(s))
            }
            (Value::Bytes(v), Value::Bytes(d)) => {
                Ok(Value::Bytes(bytes_delta(v, d, sub)?))
            }
            (Value::UnicodeString(v), Value::Bytes(d)) => {
                let b = bytes_delta(v.as_bytes(), d, sub)?;
                let s = String::from_utf8(b)?; // [ERR R2]
                Ok(Value::UnicodeString(s))
            }
            _ => Err(Error::Runtime(format!("Cannot apply delta {:?} to {:?}", delta, self))),
        }
    }

    pub fn apply_tail(&self, tail: Value) -> Result<Value> {
        let len: usize;
        match (self, &tail) {
            (Value::ASCIIString(v), Value::ASCIIString(t)) => {
                len = min(t.len(), v.len());
            }
            (Value::UnicodeString(v), Value::Bytes(t)) => {
                len = min(t.len(), v.len());
            }
            (Value::Bytes(v), Value::Bytes(t)) => {
                len = min(t.len(), v.len());
            }
            _ => return Err(Error::Runtime(format!("Cannot apply tail {:?} to {:?}", tail, self))),
        }
        self.apply_delta(tail, len as i32)
    }

    pub fn apply_increment(&self) -> Result<Value> {
        match self {
            Value::UInt32(v) => {
                Ok(Value::UInt32(v + 1))
            }
            Value::Int32(v) => {
                Ok(Value::Int32(v + 1))
            }
            Value::UInt64(v) => {
                Ok(Value::UInt64(v + 1))
            }
            Value::Int64(v) => {
                Ok(Value::Int64(v + 1))
            }
            _ => Err(Error::Runtime(format!("Cannot apply increment to {:?}", self)))
        }
    }

    pub fn find_delta(&self, prev: &Value) -> Result<(Value, i32)> {
        match (self, prev) {
            (Value::Int32(v), Value::Int32(p)) => {
                Ok((Value::Int64((v - p) as i64), 0))
            }
            (Value::Int64(v), Value::Int64(p)) => {
                Ok((Value::Int64(v - p), 0))
            }
            (Value::UInt32(v), Value::UInt32(p)) => {
                Ok((Value::Int64(*v as i64 - *p as i64), 0))
            }
            (Value::UInt64(v), Value::UInt64(p)) => {
                if *v < *p {
                    Ok((Value::Int64(-((*p - *v) as i64)), 0))
                } else {
                    Ok((Value::Int64((*v - *p) as i64), 0))
                }
            }
            (Value::ASCIIString(v), Value::ASCIIString(p)) => {
                let (delta, sub) = string_delta(p, v)?;
                Ok((Value::ASCIIString(delta.to_string()), sub))
            }
            (Value::UnicodeString(v), Value::UnicodeString(p)) => {
                let (delta, sub) = bytes_delta(p.as_bytes(), v.as_bytes())?;
                Ok((Value::Bytes(delta.to_vec()), sub))
            }
            (Value::Bytes(v), Value::Bytes(p)) => {
                let (delta, sub) = bytes_delta(p, v)?;
                Ok((Value::Bytes(delta.to_vec()), sub))
            }
            _ => unimplemented!()
        }
    }

    pub fn find_tail(&self, prev: &Value) -> Result<Value> {
        match (self, prev) {
            (Value::ASCIIString(v), Value::ASCIIString(p)) => {
                let tail = string_tail(p, v)?;
                Ok(Value::ASCIIString(tail.to_string()))
            }
            (Value::UnicodeString(v), Value::UnicodeString(p)) => {
                let tail = bytes_tail(p.as_bytes(), v.as_bytes())?;
                Ok(Value::Bytes(tail.to_vec()))
            }
            (Value::Bytes(v), Value::Bytes(p)) => {
                let tail = bytes_tail(p, v)?;
                Ok(Value::Bytes(tail.to_vec()))
            }
            _ => unimplemented!()
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::UInt32(v) => f.write_fmt(format_args!("{v}")),
            Value::Int32(v) => f.write_fmt(format_args!("{v}")),
            Value::UInt64(v) => f.write_fmt(format_args!("{v}")),
            Value::Int64(v) => f.write_fmt(format_args!("{v}")),
            Value::Decimal(v) => f.write_fmt(format_args!("{v}")),
            Value::ASCIIString(s) => f.write_str(s),
            Value::UnicodeString(s) => f.write_fmt(format_args!("{s}")),
            Value::Bytes(b) => {
                let mut s = String::with_capacity(2 * b.len());
                for v in b {
                    s += &format!("{:02x}", v);
                }
                f.write_fmt(format_args!("{s}"))
            }
        }
    }
}
