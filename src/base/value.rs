use std::cmp::min;
use std::fmt::{Display, Formatter, Write};
use std::num::{IntErrorKind, ParseIntError};
use std::str::FromStr;

use crate::base::decimal::Decimal;
use crate::decoder::reader::invalid_utf8;
use crate::utils::bytes::{
    bytes_delta, bytes_tail, string_delta, string_tail, string_to_bytes, trim_whitespace,
};
use crate::{Error, Result};

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
    /// Creates new value type from tag.
    /// # Errors
    /// Returns error if tag is unknown.
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
            _ => Err(Error::Static(format!("Unknown type: {tag}"))),
        }
    }

    #[must_use]
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
            ValueType::ASCIIString | ValueType::UnicodeString => "string",
            ValueType::Bytes => "byteVector",
            ValueType::Sequence => "sequence",
            ValueType::Group => "group",
            ValueType::TemplateReference => "templateRef",
        }
    }

    /// Returns default value of the self type.
    /// # Errors
    /// Returns error if type does not have default type.
    pub fn to_default_value(&self) -> Result<Value> {
        match self {
            ValueType::Int32 | ValueType::Exponent => Ok(Value::Int32(0)),
            ValueType::Int64 | ValueType::Mantissa => Ok(Value::Int64(0)),
            ValueType::UInt32 | ValueType::Length => Ok(Value::UInt32(0)),
            ValueType::UInt64 => Ok(Value::UInt64(0)),
            ValueType::Decimal => Ok(Value::Decimal(Decimal::default())),
            ValueType::ASCIIString => Ok(Value::ASCIIString(String::new())),
            ValueType::UnicodeString => Ok(Value::UnicodeString(String::new())),
            ValueType::Bytes => Ok(Value::Bytes(Vec::new())),
            _ => Err(Error::Runtime(format!(
                "{} cannot be converted to value",
                self.type_str()
            ))),
        }
    }

    /// Converts string to the self type.
    /// # Errors
    /// Returns error if type is not convertible from string.
    pub fn str_to_value(&self, s: &str) -> Result<Value> {
        let mut value = match self {
            ValueType::Int32 | ValueType::Exponent => Value::Int32(0),
            ValueType::Int64 | ValueType::Mantissa => Value::Int64(0),
            ValueType::UInt32 | ValueType::Length => Value::UInt32(0),
            ValueType::UInt64 => Value::UInt64(0),
            ValueType::Decimal => Value::Decimal(Decimal::default()),
            ValueType::ASCIIString => Value::ASCIIString(String::new()),
            ValueType::UnicodeString => Value::UnicodeString(String::new()),
            ValueType::Bytes => Value::Bytes(Vec::new()),
            _ => {
                return Err(Error::Runtime(format!(
                    "{} cannot be converted to value",
                    self.type_str()
                )));
            }
        };
        value.set_from_string(s)?;
        Ok(value)
    }

    #[allow(clippy::match_like_matches_macro)]
    #[must_use]
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
            _ => false,
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
    /// Sets the value of the same type converted from the string.
    /// Integers and decimals are whitespace trimmed; byte vectors are hexadecimal digits possibly
    /// interleaved with whitespace.
    /// # Errors
    /// Returns [`Error::Dynamic`] if the string does not match the syntax [ERR D11], the number is out of
    /// range [ERR R1, R4] or an ASCII string has non-ASCII characters [ERR R3].
    pub fn set_from_string(&mut self, s: &str) -> Result<()> {
        match self {
            Value::UInt32(_) => *self = Value::UInt32(parse_integer(s)?),
            Value::Int32(_) => *self = Value::Int32(parse_integer(s)?),
            Value::UInt64(_) => *self = Value::UInt64(parse_integer(s)?),
            Value::Int64(_) => *self = Value::Int64(parse_integer(s)?),
            Value::Decimal(_) => *self = Value::Decimal(Decimal::from_string(s)?),
            Value::ASCIIString(_) => {
                if !s.is_ascii() {
                    return Err(Error::Dynamic(format!("'{s}' is not an ASCII string"))); // [ERR R3]
                }
                *self = Value::ASCIIString(s.to_string());
            }
            Value::UnicodeString(_) => *self = Value::UnicodeString(s.to_string()),
            // The string is interpreted as an even number of hexadecimal digits [0-9A-Fa-f] possibly interleaved
            // with whitespace. The literal is turned into a byte vector by first stripping any whitespace.
            // Then each pair of characters is interpreted as a hexadecimal number representing a single byte.
            Value::Bytes(_) => *self = Value::Bytes(string_to_bytes(s)?),
        }
        Ok(())
    }

    /// Applies delta to the current value and returns new value.
    /// # Errors
    /// Returns error if the delta doesn't match the value type, the subtraction length is larger than the
    /// value [ERR D7], or the combined integer is out of range [ERR R4].
    pub fn apply_delta(&self, delta: &Delta) -> Result<Value> {
        self.apply_delta_ref(&delta.borrowed())
    }

    // Same as `apply_delta` with a borrowed delta.
    pub(crate) fn apply_delta_ref(&self, delta: &DeltaRef) -> Result<Value> {
        match (self, delta) {
            (Value::UInt32(v), DeltaRef::Int(d)) => Ok(Value::UInt32(combine(*v, *d)?)),
            (Value::Int32(v), DeltaRef::Int(d)) => Ok(Value::Int32(combine(*v, *d)?)),
            (Value::UInt64(v), DeltaRef::Int(d)) => Ok(Value::UInt64(combine(*v, *d)?)),
            (Value::Int64(v), DeltaRef::Int(d)) => Ok(Value::Int64(combine(*v, *d)?)),
            (Value::ASCIIString(v), DeltaRef::Ascii(d, sub)) => {
                let (front, i) = sub2index(*sub, v.len())?;
                Ok(Value::ASCIIString(splice_str(v, d, front, i)))
            }
            (Value::Bytes(v), DeltaRef::Bytes(d, sub)) => {
                let (front, i) = sub2index(*sub, v.len())?;
                Ok(Value::Bytes(splice_bytes(v, d, front, i)))
            }
            (Value::UnicodeString(v), DeltaRef::Bytes(d, sub)) => {
                let (front, i) = sub2index(*sub, v.len())?;
                bytes_to_unicode(splice_bytes(v.as_bytes(), d, front, i))
            }
            _ => Err(Error::Runtime(format!(
                "Cannot apply delta {delta:?} to {self:?}"
            ))),
        }
    }

    /// Same as [`Value::apply_delta`] but modifies the value in place, reusing the buffer when possible.
    /// # Errors
    /// Same as [`Value::apply_delta`]. If a Unicode string doesn't remain valid UTF-8, it is left empty.
    pub fn apply_delta_in_place(&mut self, delta: &Delta) -> Result<()> {
        self.apply_delta_ref_in_place(&delta.borrowed())
    }

    // Same as `apply_delta_in_place` with a borrowed delta.
    pub(crate) fn apply_delta_ref_in_place(&mut self, delta: &DeltaRef) -> Result<()> {
        match (&mut *self, delta) {
            (Value::UInt32(v), DeltaRef::Int(d)) => {
                *v = combine(*v, *d)?;
                Ok(())
            }
            (Value::Int32(v), DeltaRef::Int(d)) => {
                *v = combine(*v, *d)?;
                Ok(())
            }
            (Value::UInt64(v), DeltaRef::Int(d)) => {
                *v = combine(*v, *d)?;
                Ok(())
            }
            (Value::Int64(v), DeltaRef::Int(d)) => {
                *v = combine(*v, *d)?;
                Ok(())
            }
            (Value::ASCIIString(v), DeltaRef::Ascii(d, sub)) => {
                let (front, i) = sub2index(*sub, v.len())?;
                if front {
                    v.replace_range(..i, d);
                } else {
                    v.truncate(i);
                    v.push_str(d);
                }
                Ok(())
            }
            (Value::Bytes(v), DeltaRef::Bytes(d, sub)) => {
                let (front, i) = sub2index(*sub, v.len())?;
                splice_in_place(v, d, front, i);
                Ok(())
            }
            (Value::UnicodeString(v), DeltaRef::Bytes(d, sub)) => {
                let (front, i) = sub2index(*sub, v.len())?;
                unicode_in_place(v, |b| splice_in_place(b, d, front, i))
            }
            _ => {
                *self = self.apply_delta_ref(delta)?;
                Ok(())
            }
        }
    }

    /// Applies tail to the current value and returns new value.
    /// # Errors
    /// Returns error if types are not bytes/string or previous and current types mismatch.
    pub fn apply_tail(&self, tail: &Value) -> Result<Value> {
        self.apply_tail_ref(&tail_ref(tail)?)
    }

    // Same as `apply_tail` with a borrowed tail.
    pub(crate) fn apply_tail_ref(&self, tail: &TailRef) -> Result<Value> {
        match (self, tail) {
            (Value::ASCIIString(v), TailRef::Ascii(t)) => Ok(Value::ASCIIString(splice_str(
                v,
                t,
                false,
                tail_index(v.len(), t.len()),
            ))),
            (Value::UnicodeString(v), TailRef::Bytes(t)) => bytes_to_unicode(splice_bytes(
                v.as_bytes(),
                t,
                false,
                tail_index(v.len(), t.len()),
            )),
            (Value::Bytes(v), TailRef::Bytes(t)) => Ok(Value::Bytes(splice_bytes(
                v,
                t,
                false,
                tail_index(v.len(), t.len()),
            ))),
            _ => Err(Error::Runtime(format!(
                "Cannot apply tail {tail:?} to {self:?}"
            ))),
        }
    }

    /// Assigns `src` to the value reusing allocated buffer when possible.
    pub fn assign(&mut self, src: &Value) {
        match (self, src) {
            (Value::ASCIIString(d), Value::ASCIIString(s)) |
            (Value::UnicodeString(d), Value::UnicodeString(s)) => d.clone_from(s),
            (Value::Bytes(d), Value::Bytes(s)) => d.clone_from(s),
            (d, s) => *d = s.clone(),
        }
    }

    /// Same as [`Value::apply_tail`] but modifies the value in place.
    /// # Errors
    /// Returns error if types are not bytes/string or previous and current types mismatch.
    /// If a Unicode string doesn't remain valid UTF-8, it is left empty.
    pub fn apply_tail_in_place(&mut self, tail: &Value) -> Result<()> {
        self.apply_tail_ref_in_place(&tail_ref(tail)?)
    }

    // Same as `apply_tail_in_place` with a borrowed tail.
    pub(crate) fn apply_tail_ref_in_place(&mut self, tail: &TailRef) -> Result<()> {
        match (&mut *self, tail) {
            (Value::ASCIIString(v), TailRef::Ascii(t)) => {
                v.truncate(tail_index(v.len(), t.len()));
                v.push_str(t);
                Ok(())
            }
            (Value::Bytes(v), TailRef::Bytes(t)) => {
                v.truncate(tail_index(v.len(), t.len()));
                v.extend_from_slice(t);
                Ok(())
            }
            (Value::UnicodeString(v), TailRef::Bytes(t)) => unicode_in_place(v, |b| {
                b.truncate(tail_index(b.len(), t.len()));
                b.extend_from_slice(t);
            }),
            _ => {
                *self = self.apply_tail_ref(tail)?;
                Ok(())
            }
        }
    }

    /// Increments underlying integer value. The maximum value of the type wraps around to the minimum value.
    /// # Errors
    /// Returns error if self type is not an integer.
    pub fn apply_increment(&self) -> Result<Value> {
        match self {
            Value::UInt32(v) => Ok(Value::UInt32(v.wrapping_add(1))),
            Value::Int32(v) => Ok(Value::Int32(v.wrapping_add(1))),
            Value::UInt64(v) => Ok(Value::UInt64(v.wrapping_add(1))),
            Value::Int64(v) => Ok(Value::Int64(v.wrapping_add(1))),
            _ => Err(Error::Runtime(format!(
                "Cannot apply increment to {self:?}"
            ))),
        }
    }

    /// Searches difference between current and previous value without copying.
    /// # Errors
    /// Returns error if the types do not match.
    pub fn find_delta_ref<'a>(&'a self, prev: &'a Value) -> Result<DeltaRef<'a>> {
        Ok(match (self, prev) {
            (Value::Int32(v), Value::Int32(p)) => DeltaRef::Int(i128::from(*v) - i128::from(*p)),
            (Value::Int64(v), Value::Int64(p)) => DeltaRef::Int(i128::from(*v) - i128::from(*p)),
            (Value::UInt32(v), Value::UInt32(p)) => DeltaRef::Int(i128::from(*v) - i128::from(*p)),
            (Value::UInt64(v), Value::UInt64(p)) => DeltaRef::Int(i128::from(*v) - i128::from(*p)),
            (Value::ASCIIString(v), Value::ASCIIString(p)) => {
                let (delta, sub) = string_delta(p, v);
                DeltaRef::Ascii(delta, sub)
            }
            (Value::UnicodeString(v), Value::UnicodeString(p)) => {
                let (delta, sub) = bytes_delta(p.as_bytes(), v.as_bytes());
                DeltaRef::Bytes(delta, sub)
            }
            (Value::Bytes(v), Value::Bytes(p)) => {
                let (delta, sub) = bytes_delta(p, v);
                DeltaRef::Bytes(delta, sub)
            }
            _ => {
                return Err(Error::Runtime(format!(
                    "Cannot find delta of {self:?} from {prev:?}"
                )));
            }
        })
    }

    /// Searches difference between current and previous value.
    /// # Errors
    /// Returns error if the types do not match.
    pub fn find_delta(&self, prev: &Value) -> Result<Delta> {
        Ok(match self.find_delta_ref(prev)? {
            DeltaRef::Int(d) => Delta::Int(d),
            DeltaRef::Ascii(s, sub) => Delta::Ascii(s.to_string(), sub),
            DeltaRef::Bytes(b, sub) => Delta::Bytes(b.to_vec(), sub),
        })
    }

    /// Searches tail difference between current value and prev string or byte-slices without copying.
    /// # Errors
    /// Returns error if current len is less than previous.
    pub fn find_tail_ref<'a>(&'a self, prev: &'a Value) -> Result<TailRef<'a>> {
        match (self, prev) {
            (Value::ASCIIString(v), Value::ASCIIString(p)) => {
                let tail = string_tail(p, v)?;
                Ok(TailRef::Ascii(tail))
            }
            (Value::UnicodeString(v), Value::UnicodeString(p)) => {
                let tail = bytes_tail(p.as_bytes(), v.as_bytes())?;
                Ok(TailRef::Bytes(tail))
            }
            (Value::Bytes(v), Value::Bytes(p)) => {
                let tail = bytes_tail(p, v)?;
                Ok(TailRef::Bytes(tail))
            }
            _ => unimplemented!(),
        }
    }

    /// Searches tail difference between current value and prev string or byte-slices.
    /// # Errors
    /// Returns error if current len is less than previous.
    pub fn find_tail(&self, prev: &Value) -> Result<Value> {
        Ok(match self.find_tail_ref(prev)? {
            TailRef::Ascii(s) => Value::ASCIIString(s.to_string()),
            TailRef::Bytes(b) => Value::Bytes(b.to_vec()),
        })
    }
}

// The string is interpreted as a sequence of digits; a leading minus is allowed for signed types.
// The literal is whitespace trimmed.
fn parse_integer<T: FromStr<Err = ParseIntError>>(s: &str) -> Result<T> {
    let t = trim_whitespace(s);
    t.parse::<T>().map_err(|e| match e.kind() {
        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
            Error::Dynamic(format!("integer '{t}' is out of range")) // [ERR R4]
        }
        _ => Error::Dynamic(format!("invalid integer '{t}'")), // [ERR D11]
    })
}

// Adds an integer delta. It is a reportable error [ERR R4] if the combined value does not fit into the integer type.
fn combine<T: TryFrom<i128>>(v: impl Into<i128>, d: i128) -> Result<T> {
    v.into()
        .checked_add(d)
        .and_then(|r| T::try_from(r).ok())
        .ok_or_else(|| {
            Error::Dynamic("integer delta result is out of range".to_string()) // [ERR R4]
        })
}

// Converts substraction length into (remove from front, index).
fn sub2index(sub: i32, len: usize) -> Result<(bool, usize)> {
    // A negative subtraction length is used to remove values from the front of the string.
    // Negative zero (excess-1 encoded as -1) is used to append values to the front of the string.
    let (front, count) = if sub < 0 {
        (true, sub.unsigned_abs() as usize - 1)
    } else {
        (false, sub as usize)
    };
    if count > len {
        return Err(Error::Dynamic(format!(
            "subtraction length ({count}) is larger than string length ('{len}')"
        ))); // [ERR D7]
    }
    Ok((front, if front { count } else { len - count }))
}

// Index of the base value where the tail of `tail_len` is appended. If the tail is longer than the base value,
// the combined value becomes the tail value.
fn tail_index(len: usize, tail_len: usize) -> usize {
    len - min(tail_len, len)
}

// Combines the base value `v` with the delta part `d`: if `front`, `d` replaces `v[..i]`, otherwise `v[i..]`.
fn splice_str(v: &str, d: &str, front: bool, i: usize) -> String {
    let mut s = String::with_capacity(v.len() + d.len());
    if front {
        s.push_str(d);
        s.push_str(&v[i..]);
    } else {
        s.push_str(&v[..i]);
        s.push_str(d);
    }
    s
}

// Same as `splice_str` for bytes.
fn splice_bytes(v: &[u8], d: &[u8], front: bool, i: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(v.len() + d.len());
    if front {
        b.extend_from_slice(d);
        b.extend_from_slice(&v[i..]);
    } else {
        b.extend_from_slice(&v[..i]);
        b.extend_from_slice(d);
    }
    b
}

// Same as `splice_bytes` but modifies `v` in place.
fn splice_in_place(v: &mut Vec<u8>, d: &[u8], front: bool, i: usize) {
    if front {
        v.splice(..i, d.iter().copied());
    } else {
        v.truncate(i);
        v.extend_from_slice(d);
    }
}

// Modifies the bytes of a Unicode string in place reusing its buffer.
// It is a reportable error [ERR R2] if the result is not a valid UTF-8 sequence; the string is left empty then.
fn unicode_in_place(v: &mut String, f: impl FnOnce(&mut Vec<u8>)) -> Result<()> {
    let mut b = std::mem::take(v).into_bytes();
    f(&mut b);
    *v = String::from_utf8(b).map_err(|e| invalid_utf8(&e))?;
    Ok(())
}

// It is a reportable error [ERR R2] if the combined value is not a valid UTF-8 sequence.
fn bytes_to_unicode(b: Vec<u8>) -> Result<Value> {
    Ok(Value::UnicodeString(
        String::from_utf8(b).map_err(|e| invalid_utf8(&e))?,
    ))
}

/// Delta of a value, see [`Value::find_delta`] and [`Value::apply_delta`].
#[derive(Debug, PartialEq, Clone)]
pub enum Delta {
    /// Integer delta. A delta of 64-bit integers can need 65 bits.
    Int(i128),
    /// ASCII string delta: the string part and the subtraction length.
    Ascii(String, i32),
    /// Byte vector or Unicode string delta: the bytes part and the subtraction length.
    Bytes(Vec<u8>, i32),
}

impl Delta {
    pub(crate) fn borrowed(&self) -> DeltaRef<'_> {
        match self {
            Delta::Int(d) => DeltaRef::Int(*d),
            Delta::Ascii(s, sub) => DeltaRef::Ascii(s, *sub),
            Delta::Bytes(b, sub) => DeltaRef::Bytes(b, *sub),
        }
    }
}

// Tails are ASCII strings or byte vectors (also for Unicode strings).
fn tail_ref(tail: &Value) -> Result<TailRef<'_>> {
    match tail {
        Value::ASCIIString(t) => Ok(TailRef::Ascii(t)),
        Value::Bytes(t) => Ok(TailRef::Bytes(t)),
        _ => Err(Error::Runtime(format!("{tail:?} cannot be a tail"))),
    }
}

/// Borrowed delta of a value.
#[derive(Debug, PartialEq)]
pub enum DeltaRef<'a> {
    Int(i128),
    Ascii(&'a str, i32),
    Bytes(&'a [u8], i32),
}

/// Borrowed tail of a value.
#[derive(Debug, PartialEq)]
pub enum TailRef<'a> {
    Ascii(&'a str),
    Bytes(&'a [u8]),
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
                    let _ = write!(&mut s, "{v:02x}");
                }
                f.write_fmt(format_args!("{s}"))
            }
        }
    }
}
