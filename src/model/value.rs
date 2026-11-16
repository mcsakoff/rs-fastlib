use crate::model::template::TemplateData;
use crate::{Decimal, Error, Value};
use serde::Serialize;
use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use std::cell::Cell;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Clone)]
pub enum ValueData {
    None, // For optional groups and sequences
    Value(Option<Value>),
    Group(Fields),
    Sequence(Vec<ValueData>),                  // Always Vec of Value::Group
    StaticTemplateRef(String, Box<ValueData>), // Always Value::Group
    DynamicTemplateRef(Box<TemplateData>),
}

/// Name of a field in a group.
///
/// Short names are stored inline, so the decoder doesn't allocate a string for every field of a message.
/// The length is stored at the same place for all kinds of names: looking a field up skips names
/// of other lengths without checking their kind.
#[repr(C)]
pub struct Key {
    // The length of the name (truncated to 32 bits; used to skip names of other lengths).
    len: u32,
    kind: KeyKind,
    // `Inline`: the UTF-8 bytes of the name.
    // `Static`, `Owned`: the pointer to the bytes followed by their length (both unaligned).
    data: [u8; Key::INLINE_LEN],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum KeyKind {
    Inline,
    Static,
    // The bytes are a leaked `Box<str>` freed when the key is dropped.
    Owned,
}

const PTR_SIZE: usize = std::mem::size_of::<*const u8>();

impl Key {
    // Keeps the size of `Key` at three words.
    const INLINE_LEN: usize = 19;

    /// Creates the key that refers to a static name.
    #[must_use]
    pub fn borrowed(name: &'static str) -> Self {
        Self::out_of_line(KeyKind::Static, name.as_ptr(), name.len())
    }

    fn out_of_line(kind: KeyKind, ptr: *const u8, len: usize) -> Self {
        let mut data = [0; Key::INLINE_LEN];
        // SAFETY: `data` has room for a pointer and a length. The pointer is stored as a pointer
        //         to keep its provenance.
        unsafe {
            let dst = data.as_mut_ptr();
            dst.cast::<*const u8>().write_unaligned(ptr);
            dst.add(PTR_SIZE).cast::<usize>().write_unaligned(len);
        }
        Self {
            len: len as u32,
            kind,
            data,
        }
    }

    // Returns the pointer and the length stored by `out_of_line`.
    #[inline]
    fn raw_parts(&self) -> (*const u8, usize) {
        debug_assert_ne!(self.kind, KeyKind::Inline);
        let src = self.data.as_ptr();
        // SAFETY: `out_of_line` stores a pointer followed by a length in `data`.
        unsafe {
            (
                src.cast::<*const u8>().read_unaligned(),
                src.add(PTR_SIZE).cast::<usize>().read_unaligned(),
            )
        }
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        if self.kind == KeyKind::Inline {
            // SAFETY: inline names are not longer than `data`, see `From<&str>`.
            unsafe { self.data.get_unchecked(..self.len as usize) }
        } else {
            let (ptr, len) = self.raw_parts();
            // SAFETY: the string is either static or owned by the key.
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        // SAFETY: keys are created from `&str` only.
        unsafe { std::str::from_utf8_unchecked(self.bytes()) }
    }

    #[must_use]
    pub fn into_owned(self) -> String {
        self.as_str().to_string()
    }
}

impl From<&str> for Key {
    #[inline]
    fn from(s: &str) -> Self {
        if s.len() <= Key::INLINE_LEN {
            let mut data = [0; Key::INLINE_LEN];
            data[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                len: s.len() as u32,
                kind: KeyKind::Inline,
                data,
            }
        } else {
            Self::from(Box::<str>::from(s))
        }
    }
}

impl From<Box<str>> for Key {
    fn from(s: Box<str>) -> Self {
        let len = s.len();
        // Freed in `Drop`.
        Self::out_of_line(KeyKind::Owned, Box::into_raw(s).cast::<u8>(), len)
    }
}

impl From<String> for Key {
    // The string is already allocated, so it is kept as is.
    fn from(s: String) -> Self {
        Key::from(s.into_boxed_str())
    }
}

impl Drop for Key {
    #[inline]
    fn drop(&mut self) {
        if self.kind == KeyKind::Owned {
            let (ptr, len) = self.raw_parts();
            let bytes = std::ptr::slice_from_raw_parts_mut(ptr.cast_mut(), len);
            // SAFETY: the bytes of an owned key are a `Box<str>` leaked in `From<Box<str>>`.
            drop(unsafe { Box::from_raw(bytes as *mut str) });
        }
    }
}

impl Clone for Key {
    fn clone(&self) -> Self {
        match self.kind {
            // Long names are copied.
            KeyKind::Owned => Key::from(self.as_str()),
            kind => Self {
                len: self.len,
                kind,
                data: self.data,
            },
        }
    }
}

impl std::ops::Deref for Key {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for Key {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.len == other.len() as u32 && self.as_str() == other
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

pub(crate) fn template_ref_key(n: u32) -> Key {
    const NAMES: [&str; 16] = [
        "templateRef:0",
        "templateRef:1",
        "templateRef:2",
        "templateRef:3",
        "templateRef:4",
        "templateRef:5",
        "templateRef:6",
        "templateRef:7",
        "templateRef:8",
        "templateRef:9",
        "templateRef:10",
        "templateRef:11",
        "templateRef:12",
        "templateRef:13",
        "templateRef:14",
        "templateRef:15",
    ];
    match NAMES.get(n as usize) {
        Some(name) => Key::borrowed(name),
        None => Key::from(format!("templateRef:{n}")),
    }
}

/// Fields of a group in insertion order.
///
/// Vector with a search hit is faster than a map?
///
#[derive(Debug, Clone, Default)]
pub struct Fields {
    items: Vec<(Key, ValueData)>,
    hint: Cell<usize>,
}

impl Fields {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            hint: Cell::new(0),
        }
    }

    pub fn push(&mut self, key: impl Into<Key>, value: ValueData) {
        self.items.push((key.into(), value));
    }

    pub fn insert(&mut self, key: impl Into<Key>, value: ValueData) {
        let key = key.into();
        match self.position(&key) {
            Some(i) => self.items[i].1 = value,
            None => self.items.push((key, value)),
        }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ValueData> {
        self.position(name).map(|i| &self.items[i].1)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn position(&self, name: &str) -> Option<usize> {
        let start = self.hint.get().min(self.items.len());

        let (head, tail) = self.items.split_at(start);
        let i = match tail.iter().position(|(k, _)| k == name) {
            Some(i) => start + i,
            None => head.iter().position(|(k, _)| k == name)?,
        };
        self.hint.set(i + 1);
        Some(i)
    }
}

impl PartialEq for Fields {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.items.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}

impl<K: Into<Key>> FromIterator<(K, ValueData)> for Fields {
    fn from_iter<T: IntoIterator<Item = (K, ValueData)>>(iter: T) -> Self {
        let mut fields = Fields::default();
        fields.extend(iter);
        fields
    }
}

impl<K: Into<Key>> Extend<(K, ValueData)> for Fields {
    fn extend<T: IntoIterator<Item = (K, ValueData)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl IntoIterator for Fields {
    type Item = (Key, ValueData);
    type IntoIter = IntoIter<(Key, ValueData)>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'de> serde::Deserializer<'de> for ValueData {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::None => visitor.visit_none(),
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::UInt32(n) => visitor.visit_u32(n),
                    Value::Int32(n) => visitor.visit_i32(n),
                    Value::UInt64(n) => visitor.visit_u64(n),
                    Value::Int64(n) => visitor.visit_i64(n),
                    Value::Decimal(f) => visitor.visit_f64(f.to_float()),
                    Value::ASCIIString(s) | Value::UnicodeString(s) => visitor.visit_string(s),
                    Value::Bytes(b) => visitor.visit_byte_buf(b),
                },
            },
            ValueData::Group(_) => self.deserialize_map(visitor),
            ValueData::Sequence(_) => self.deserialize_seq(visitor),
            _ => Err(Error::Runtime(format!(
                "deserialize_any: data model unsupported type: {self:?}"
            ))),
        }
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("boolean is not supported".to_string()))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("i8 is not supported".to_string()))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("i16 is not supported".to_string()))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::Int32(n) => visitor.visit_i32(n),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_i32: data model must be Value::Int32, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_i32: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::Int64(n) => visitor.visit_i64(n),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_i64: data model must be Value::Int64, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_i64: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("u8 is not supported".to_string()))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("u16 is not supported".to_string()))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::UInt32(n) => visitor.visit_u32(n),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_u64: data model must be Value::UInt32, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_u64: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::UInt64(n) => visitor.visit_u64(n),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_u64: data model must be Value::UInt64, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_u64: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("f32 is not supported".to_string()))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::Decimal(n) => visitor.visit_f64(n.to_float()),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_f64: data model must be Value::Decimal, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_f64: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::ASCIIString(s) | Value::UnicodeString(s) => {
                        if s.len() == 1 {
                            let c = s.chars().next().unwrap();
                            visitor.visit_char(c)
                        } else {
                            Err(Error::Runtime("deserialize_char: data model must be Value::ASCIIString or Value::UnicodeString of length 1".to_string()))
                        }
                    }
                    _ => Err(Error::Runtime(format!(
                        "deserialize_char: data model must be Value::ASCIIString or Value::UnicodeString, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_char: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::ASCIIString(s) | Value::UnicodeString(s) => visitor.visit_string(s),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_string: data model must be Value::ASCIIString or Value::UnicodeString, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_string: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(v) => match v {
                    Value::Bytes(b) => visitor.visit_byte_buf(b),
                    _ => Err(Error::Runtime(format!(
                        "deserialize_byte_buf: data model must be Value::Bytes, got: {v:?}"
                    ))),
                },
            },
            _ => Err(Error::Runtime(format!(
                "deserialize_string: data model must be ValueData::Value, got: {self:?}"
            ))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self {
            ValueData::None => visitor.visit_none(),
            ValueData::Value(v) => match v {
                None => visitor.visit_none(),
                Some(_) => visitor.visit_some(self),
            },
            ValueData::Group(_) | ValueData::Sequence(_) => visitor.visit_some(self),
            _ => Err(Error::Runtime(format!(
                "deserialize_option: cannot be optional, got: {self:?}"
            ))),
        }
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("unit is not supported".to_string()))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("unit_struct is not supported".to_string()))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("newtype_struct is not supported".to_string()))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Sequence(q) => visitor.visit_seq(SequenceDeserializer::new(q)),
            _ => Err(Error::Runtime(format!(
                "deserialize_seq: data model must be ValueData::Sequence, got {self:?}"
            ))),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("tuple is not supported".to_string()))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if name == "Decimal" && len == 2 {
            return if let ValueData::Value(Some(Value::Decimal(d))) = self {
                visitor.visit_seq(d)
            } else {
                Err(Error::Runtime(format!(
                    "deserialize_tuple_struct: expected Value::Decimal, got {self:?}"
                )))
            };
        }
        Err(Error::Runtime(format!(
            "deserialize_seq: unsupported data model {self:?}"
        )))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Group(group) => visitor.visit_map(GroupDeserializer::new(group)),
            _ => Err(Error::Runtime(format!(
                "deserialize_map: data model must be ValueData::Group, got: {self:?}"
            ))),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::DynamicTemplateRef(t) => t.deserialize_enum(name, variants, visitor),
            _ => Err(Error::Runtime(format!(
                "deserialize_enum: data model must be ValueData::DynamicTemplateRef, got: {self:?}"
            ))),
        }
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("identifier is not supported".to_string()))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

struct GroupDeserializer {
    items: <Fields as IntoIterator>::IntoIter,
    value: Option<ValueData>,
}

impl GroupDeserializer {
    fn new(values: Fields) -> Self {
        Self {
            items: values.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for GroupDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.items.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(Error::Runtime(
                "visit_value called before visit_key".to_string(),
            )),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.items.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct SequenceDeserializer {
    items: <Vec<ValueData> as IntoIterator>::IntoIter,
}

impl SequenceDeserializer {
    fn new(values: Vec<ValueData>) -> Self {
        Self {
            items: values.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for SequenceDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.items.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

pub(crate) struct ValueDataSerializer;

impl serde::Serializer for ValueDataSerializer {
    type Ok = ValueData;
    type Error = Error;
    type SerializeSeq = ValueDataSequenceSerializer;
    type SerializeTuple = Self;
    type SerializeTupleStruct = ValueDataDecimalSerializer;
    type SerializeTupleVariant = Self;
    type SerializeMap = ValueDataMapSerializer;
    type SerializeStruct = ValueDataGroupSerializer;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "bool"
        )))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int32(i32::from(v)))))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int32(i32::from(v)))))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int32(v))))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int64(v))))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt32(u32::from(v)))))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt32(u32::from(v)))))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt32(v))))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt64(v))))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Decimal(Decimal::from_float(
            f64::from(v),
        )?))))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Decimal(Decimal::from_float(
            v,
        )?))))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let s = v.to_string();
        self.serialize_str(&s)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if v.is_ascii() {
            Ok(ValueData::Value(Some(Value::ASCIIString(v.to_string()))))
        } else {
            Ok(ValueData::Value(Some(Value::UnicodeString(v.to_string()))))
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Bytes(v.to_vec()))))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(None))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "unit"
        )))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "unit struct"
        )))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "unit variant"
        )))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "newtype struct"
        )))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(ValueData::DynamicTemplateRef(Box::new(TemplateData {
            name: variant.to_string(),
            value: value.serialize(ValueDataSerializer)?,
        })))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ValueDataSequenceSerializer::new(len))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "tuple"
        )))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        if name == "Decimal" && len == 2 {
            Ok(ValueDataDecimalSerializer::new())
        } else {
            Err(Error::Runtime(format!(
                "Serialization to {} is not supported",
                "tuple struct"
            )))
        }
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "tuple variant"
        )))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ValueDataMapSerializer::new(len))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ValueDataGroupSerializer::new(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Runtime(format!(
            "Serialization to {} is not supported",
            "struct variant"
        )))
    }
}

impl SerializeTuple for ValueDataSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl SerializeTupleVariant for ValueDataSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl SerializeStructVariant for ValueDataSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

pub(crate) struct ValueDataMapSerializer {
    data: Fields,
}

impl ValueDataMapSerializer {
    fn new(len: Option<usize>) -> Self {
        Self {
            data: Fields::with_capacity(len.unwrap_or_default()),
        }
    }
}

impl SerializeMap for ValueDataMapSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let ValueData::Value(Some(Value::ASCIIString(s) | Value::UnicodeString(s))) =
            key.serialize(ValueDataSerializer)?
        else {
            return Err(Error::Runtime(
                "serialize_entry: key must be a string".to_string(),
            ));
        };
        let value = value.serialize(ValueDataSerializer)?;
        self.data.push(s, value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Group(self.data))
    }
}

pub(crate) struct ValueDataGroupSerializer {
    data: Fields,
}

impl ValueDataGroupSerializer {
    fn new(len: usize) -> Self {
        Self {
            data: Fields::with_capacity(len),
        }
    }
}

impl SerializeStruct for ValueDataGroupSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Static names are not copied.
        self.data
            .push(Key::borrowed(key), value.serialize(ValueDataSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Group(self.data))
    }
}

pub(crate) struct ValueDataSequenceSerializer {
    data: Vec<ValueData>,
}

impl ValueDataSequenceSerializer {
    fn new(len: Option<usize>) -> Self {
        Self {
            data: match len {
                Some(len) => Vec::with_capacity(len),
                None => Vec::new(),
            },
        }
    }
}

impl SerializeSeq for ValueDataSequenceSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.data.push(value.serialize(ValueDataSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Sequence(self.data))
    }
}

pub(crate) struct ValueDataDecimalSerializer {
    data: Decimal,
}

impl ValueDataDecimalSerializer {
    pub(crate) fn new() -> Self {
        Self {
            data: Decimal::new(0, 0),
        }
    }
}

impl SerializeTupleStruct for ValueDataDecimalSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut self.data)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Decimal(self.data))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(n: u32) -> ValueData {
        ValueData::Value(Some(Value::UInt32(n)))
    }

    #[test]
    fn key() {
        assert_eq!(std::mem::size_of::<Key>(), 3 * std::mem::size_of::<usize>());
        let long = "x".repeat(Key::INLINE_LEN + 1);
        for name in [
            "",
            "a",
            "NoMDEntries",
            "é€😀",
            &"y".repeat(Key::INLINE_LEN),
            &long,
            &"z".repeat(1000),
        ] {
            assert_eq!(
                Key::from(name).kind == KeyKind::Inline,
                name.len() <= Key::INLINE_LEN,
                "{name}"
            );
            assert_eq!(Key::from(name.to_string()).kind, KeyKind::Owned);
            for key in [Key::from(name), Key::from(name.to_string())] {
                assert_eq!(key.as_str(), name);
                assert!(key == *name);
                assert!(key != *"other");
                assert_eq!(format!("{key}"), name);
                assert_eq!(format!("{key:?}"), format!("{name:?}"));
                let copy = key.clone();
                assert_eq!(copy, key);
                drop(key);
                assert_eq!(copy.into_owned(), name);
            }
        }
        let key = Key::borrowed("static name that is longer than inline");
        assert_eq!(key.kind, KeyKind::Static);
        assert_eq!(
            key.clone().as_str(),
            "static name that is longer than inline"
        );
        assert_eq!(key, Key::from("static name that is longer than inline"));
        assert_ne!(key, Key::from("static name that is longer than inlinE"));
    }

    #[test]
    fn fields() {
        let mut f = Fields::default();
        f.push("a", v(1));
        f.push("b", v(2));
        f.push("c".to_string(), v(3));
        for name in ["c", "a", "b", "b", "a", "c"] {
            assert!(f.get(name).is_some(), "{name}");
        }
        assert_eq!(f.get("d"), None);

        f.insert("b", v(20));
        f.insert("d", v(4));
        assert_eq!(f.len(), 4);
        assert_eq!(f.get("b"), Some(&v(20)));
        assert_eq!(f.get("d"), Some(&v(4)));

        let other: Fields = [("d", v(4)), ("c", v(3)), ("b", v(20)), ("a", v(1))]
            .into_iter()
            .collect();
        assert_eq!(f, other);

        let other: Fields = [("d", v(4)), ("c", v(3)), ("b", v(2)), ("a", v(1))]
            .into_iter()
            .collect();
        assert_ne!(f, other);
    }

    #[test]
    fn template_ref_keys() {
        for n in [0, 15, 16, 123] {
            assert_eq!(template_ref_key(n).as_str(), format!("templateRef:{n}"))
        }
    }
}
