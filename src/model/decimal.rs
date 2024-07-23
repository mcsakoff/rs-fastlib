use std::fmt::Formatter;

use serde::{de, Deserialize, Deserializer, forward_to_deserialize_any, ser, Serialize, Serializer};
use serde::ser::SerializeTupleStruct;

use crate::{Decimal, Error};

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DecimalVisitor;
        impl<'de> de::Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("expecting String or tuple")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Decimal::from_string(&v) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(E::custom(e))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let e = match seq.next_element()? {
                    Some(e) => e,
                    None => return Err(de::Error::invalid_length(0, &self))
                };
                let m = match seq.next_element()? {
                    Some(m) => m,
                    None => return Err(de::Error::invalid_length(1, &self))
                };
                Ok(Decimal::new(e, m))
            }
        }

        deserializer.deserialize_tuple_struct("Decimal", 2, DecimalVisitor)
    }
}

impl<'de> de::SeqAccess<'de> for Decimal {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self).map(Some)
    }
}

impl<'a, 'de> Deserializer<'de> for &'a mut Decimal {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Static("Decimal: unsupported deserialize_*".to_string()))
    }

    forward_to_deserialize_any! {
        bool i8 i16 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        seq tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.exponent)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.mantissa)
    }
}

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple_struct("Decimal", 2)?;
        seq.serialize_field(&self.exponent)?;
        seq.serialize_field(&self.mantissa)?;
        seq.end()
    }
}

impl Serializer for &mut Decimal {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.exponent = v;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.mantissa = v;
        Ok(())
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<Self::Ok, Self::Error> { unreachable!() }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> { unreachable!() }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> { unreachable!() }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { unreachable!() }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { unreachable!() }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> { unreachable!() }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> { unreachable!() }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> { unreachable!() }
}

impl ser::SerializeSeq for &mut Decimal {
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

impl ser::SerializeTuple for &mut Decimal {
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

impl ser::SerializeTupleStruct for &mut Decimal {
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

impl ser::SerializeTupleVariant for &mut Decimal {
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

impl ser::SerializeMap for &mut Decimal {
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
        unreachable!()
    }
}

impl ser::SerializeStruct for &mut Decimal {
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

impl ser::SerializeStructVariant for &mut Decimal {
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
