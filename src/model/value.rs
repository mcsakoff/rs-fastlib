use hashbrown::HashMap;
use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};
use serde::Serialize;

use crate::{Decimal, Error, Value};
use crate::model::template::TemplateData;

#[derive(Debug, PartialEq, Clone)]
pub enum ValueData {
    None,                                        // For optional groups and sequences
    Value(Option<Value>),
    Group(HashMap<String, ValueData>),
    Sequence(Vec<ValueData>),                    // Always Vec of Value::Group
    StaticTemplateRef(String, Box<ValueData>),   // Always Value::Group
    DynamicTemplateRef(Box<TemplateData>),
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
                    Value::ASCIIString(s) => visitor.visit_string(s),
                    Value::UnicodeString(s) => visitor.visit_string(s),
                    Value::Bytes(b) => visitor.visit_byte_buf(b),
                }
            }
            _ => Err(Error::Runtime("deserialize_any: data model must be ValueData::Value".to_string())),
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
                    Value::Int32(n) => {
                        visitor.visit_i32(n)
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_i32: data model must be Value::Int32".to_string()))
                    }
                }
            }
            _ => {
                Err(Error::Runtime("deserialize_i32: data model must be ValueData::Value".to_string()))
            }
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
                    Value::Int64(n) => {
                        visitor.visit_i64(n)
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_i64: data model must be Value::Int64".to_string()))
                    }
                }
            }
            _ => {
                Err(Error::Runtime("deserialize_i64: data model must be ValueData::Value".to_string()))
            }
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
                    Value::UInt32(n) => {
                        visitor.visit_u32(n)
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_u64: data model must be Value::UInt32".to_string()))
                    }
                }
            }
            _ => {
                Err(Error::Runtime("deserialize_u64: data model must be ValueData::Value".to_string()))
            }
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
                    Value::UInt64(n) => {
                        visitor.visit_u64(n)
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_u64: data model must be Value::UInt64".to_string()))
                    }
                }
            }
            _ => {
                Err(Error::Runtime("deserialize_u64: data model must be ValueData::Value".to_string()))
            }
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
                    Value::Decimal(n) => {
                        visitor.visit_f64(n.to_float())
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_f64: data model must be Value::Decimal".to_string()))
                    }
                }
            }
            _ => {
                Err(Error::Runtime("deserialize_f64: data model must be ValueData::Value".to_string()))
            }
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
                    _ => {
                        Err(Error::Runtime("deserialize_char: data model must be Value::ASCIIString or Value::UnicodeString".to_string()))
                    }
                }
            },
            _ => {
                Err(Error::Runtime("deserialize_char: data model must be ValueData::Value".to_string()))
            }
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
                    Value::ASCIIString(s) | Value::UnicodeString(s) => {
                        visitor.visit_string(s)
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_string: data model must be Value::ASCIIString or Value::UnicodeString".to_string()))
                    }
                }
            },
            _ => {
                Err(Error::Runtime("deserialize_string: data model must be ValueData::Value".to_string()))
            }
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
                    Value::Bytes(b) => {
                        visitor.visit_byte_buf(b)
                    }
                    _ => {
                        Err(Error::Runtime("deserialize_byte_buf: data model must be Value::Bytes".to_string()))
                    }
                }
            },
            _ => {
                Err(Error::Runtime("deserialize_string: data model must be ValueData::Value".to_string()))
            }
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
            }
            ValueData::Group(_) => {
                visitor.visit_some(self)
            }
            ValueData::Sequence(_) => {
                visitor.visit_some(self)
            }
            _ => {
                Err(Error::Runtime("deserialize_option: cannot be optional".to_string()))
            }
        }
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("unit is not supported".to_string()))
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("unit_struct is not supported".to_string()))
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error>
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
            ValueData::Sequence(q) => {
                visitor.visit_seq(SequenceDeserializer::new(q))
            }
            _ => {
                Err(Error::Runtime("deserialize_seq: data model must be ValueData::Sequence".to_string()))
            }
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("tuple is not supported".to_string()))
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if name == "Decimal" && len == 2 {
            return if let ValueData::Value(Some(Value::Decimal(d))) = self {
                visitor.visit_seq(d)
            } else {
                Err(Error::Runtime("deserialize_tuple_struct: expected Value::Decimal".to_string()))
            }
        }
        Err(Error::Runtime("deserialize_seq: unsupported data model".to_string()))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::Group(group) => {
                visitor.visit_map(GroupDeserializer::new(group))
            }
            _ => {
                Err(Error::Runtime("deserialize_map: data model must be ValueData::Group".to_string()))
            }
        }
    }

    fn deserialize_struct<V>(self, _name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueData::DynamicTemplateRef(t) => {
                t.deserialize_enum(name, variants, visitor)
            }
            _ => {
                Err(Error::Runtime("deserialize_enum: data model must be ValueData::DynamicTemplateRef".to_string()))
            }
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
    items: <HashMap<String, ValueData> as IntoIterator>::IntoIter,
    value: Option<ValueData>,
}

impl GroupDeserializer {
    fn new(values: HashMap<String, ValueData>) -> Self {
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
            Some(value) => {
                seed.deserialize(value)
            }
            None => {
                Err(Error::Runtime("visit_value called before visit_key".to_string()))
            }
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
            Some(value) => {
                seed.deserialize(value).map(Some)
            }
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
        Err(Error::Runtime(format!("Serialization to {} is not supported", "bool")))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int32(v as i32))))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int32(v as i32))))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int32(v))))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Int64(v))))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt32(v as u32))))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt32(v as u32))))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt32(v))))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::UInt64(v))))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Decimal(Decimal::from_float(v as f64)?))))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Decimal(Decimal::from_float(v)?))))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let s =v.to_string();
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
        T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "unit")))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "unit struct")))
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "unit variant")))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "newtype struct")))
    }

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
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
        Err(Error::Runtime(format!("Serialization to {} is not supported", "tuple")))

    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        if name == "Decimal" && len == 2 {
            Ok(ValueDataDecimalSerializer::new())
        } else {
            Err(Error::Runtime(format!("Serialization to {} is not supported", "tuple struct")))
        }
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "tuple variant")))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ValueDataMapSerializer::new(len))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ValueDataGroupSerializer::new(len))
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "struct variant")))
    }
}

impl SerializeTuple for ValueDataSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
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
        T: ?Sized + Serialize
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
        T: ?Sized + Serialize
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}


pub(crate) struct ValueDataMapSerializer {
    data: HashMap<String, ValueData>
}


impl ValueDataMapSerializer {
    fn new(len: Option<usize>) -> Self {
        Self {
            data: match len {
                Some(len) => HashMap::with_capacity(len),
                None => HashMap::new()
            },
        }
    }
}

impl SerializeMap for ValueDataMapSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        unreachable!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        unreachable!()
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize
    {
        let key = match key.serialize(ValueDataSerializer)? {
            ValueData::Value(Some(Value::ASCIIString(s))) => s,
            ValueData::Value(Some(Value::UnicodeString(s))) => s,
            _ => return Err(Error::Runtime("serialize_entry: key must be a string".to_string()))
        };
        let value = value.serialize(ValueDataSerializer)?;
        self.data.insert(key, value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Group(self.data))
    }
}


pub(crate) struct ValueDataGroupSerializer {
    data: HashMap<String, ValueData>
}

impl ValueDataGroupSerializer {
    fn new(len: usize) -> Self {
        Self {
            data: HashMap::with_capacity(len)
        }
    }
}

impl SerializeStruct for ValueDataGroupSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.data.insert(key.to_string(), value.serialize(ValueDataSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Group(self.data))
    }
}


pub(crate) struct ValueDataSequenceSerializer {
    data: Vec<ValueData>
}

impl ValueDataSequenceSerializer {
    fn new(len: Option<usize>) -> Self {
        Self {
            data: match len {
                Some(len) => Vec::with_capacity(len),
                None => Vec::new()
            },
        }
    }
}

impl SerializeSeq for ValueDataSequenceSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.data.push(value.serialize(ValueDataSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Sequence(self.data))
    }
}


pub(crate) struct ValueDataDecimalSerializer {
    data: Decimal
}

impl ValueDataDecimalSerializer {
    pub(crate) fn new() -> Self {
        Self {
            data: Decimal::new(0, 0)
        }
    }
}

impl SerializeTupleStruct for ValueDataDecimalSerializer {
    type Ok = ValueData;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        value.serialize(&mut self.data)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueData::Value(Some(Value::Decimal(self.data))))
    }
}
