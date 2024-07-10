use std::fmt::Formatter;
use serde::{Deserialize, Deserializer, de, forward_to_deserialize_any};
use crate::{Decimal, Error};

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct DecimalVisitor;
        impl<'de> de::Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("expecting String or tuple")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                match Decimal::from_string(&v) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(E::custom(e))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>
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
        T: de::DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self).map(Some)
    }
}

impl<'a, 'de> Deserializer<'de> for &'a mut Decimal {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
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
        V: de::Visitor<'de>
    {
        visitor.visit_i32(self.exponent)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
    {
        visitor.visit_i64(self.mantissa)
    }
}
