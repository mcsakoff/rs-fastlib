use std::string::ToString;

use serde::{forward_to_deserialize_any, Serialize};
use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, value::StringDeserializer, VariantAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use crate::Error;

use super::value::{ValueData, ValueDataSerializer};

#[derive(Debug, PartialEq, Clone)]
pub struct TemplateData {
    pub name: String,
    pub value: ValueData, // Must be Value::Group
}

impl TemplateData {
    pub fn new_empty() -> Self {
        Self {
            name: String::new(),
            value: ValueData::None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for TemplateData {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("message must be enum".to_string()))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        seq tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(EnumDeserializer {
            variant: self.name,
            value: self.value,
        })
    }
}


struct EnumDeserializer {
    variant: String,
    value: ValueData,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant: StringDeserializer<Error> = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        let value = seed.deserialize(variant)?;
        Ok((value, visitor))
    }
}


struct VariantDeserializer {
    value: ValueData,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Error::Static("message body must be struct".to_string()))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            ValueData::Group(_) => {
                seed.deserialize(self.value)
            }
            _ => {
                Err(Error::Runtime("message data model must be ValueData::Group".to_string()))
            }
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("message body must be struct".to_string()))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Static("message body must be struct".to_string()))
    }
}


impl serde::Serializer for &mut TemplateData {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "bool")))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "i8")))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "i16")))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "i32")))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "i64")))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "u8")))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "u16")))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "u32")))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "u64")))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "f32")))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "f64")))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "char")))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "str")))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "bytes")))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "none")))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "some")))
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
        T: ?Sized + Serialize,
    {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "newtype struct")))
    }

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.name = variant.to_string();
        self.value = value.serialize(ValueDataSerializer)?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "seq")))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "tuple")))
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "tuple struct")))
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "tuple variant")))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "map")))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "struct")))
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Runtime(format!("Serialization to {} is not supported", "struct variant")))
    }
}

impl SerializeSeq for &mut TemplateData {
    type Ok = ();
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

impl SerializeTuple for &mut TemplateData {
    type Ok = ();
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

impl SerializeTupleStruct for &mut TemplateData {
    type Ok = ();
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

impl SerializeTupleVariant for &mut TemplateData {
    type Ok = ();
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

impl SerializeMap for &mut TemplateData {
    type Ok = ();
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

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl SerializeStruct for &mut TemplateData {
    type Ok = ();
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

impl SerializeStructVariant for &mut TemplateData {
    type Ok = ();
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
