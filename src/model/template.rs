use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, value::StringDeserializer, VariantAccess, Visitor};
use serde::forward_to_deserialize_any;

use crate::Error;

use super::value::ValueData;

#[derive(Debug, PartialEq)]
pub struct TemplateData {
    pub name: String,
    pub value: ValueData, // Must be Value::Group
}

impl<'de> serde::Deserializer<'de> for TemplateData {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
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
        V: Visitor<'de>
    {
        visitor.visit_enum(EnumDeserializer{
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
        T: DeserializeSeed<'de>
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
        V: Visitor<'de>
    {
        Err(Error::Static("message body must be struct".to_string()))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        Err(Error::Static("message body must be struct".to_string()))
    }
}
