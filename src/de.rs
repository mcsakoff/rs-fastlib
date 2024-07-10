use std::io::Read;

use serde::de::Deserialize;

use crate::{Decoder, Error, Reader, Result};
use crate::model::ModelFactory;

pub fn from_vec<'de, T>(decoder: &mut Decoder, bytes: Vec<u8>) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_vec(bytes, &mut msg)?;

    // Deserialize from internal data model into user data type
    let data = msg.data.unwrap();
    T::deserialize(data)
}

#[allow(unused)]
pub fn from_bytes<'de, T>(decoder: &mut Decoder, bytes: &mut bytes::Bytes) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_bytes(bytes, &mut msg)?;

    // Deserialize from internal data model into user data type
    let data = msg.data.unwrap();
    T::deserialize(data)
}

#[allow(unused)]
pub fn from_reader<'de, T>(decoder: &mut Decoder, rdr: &mut impl Reader) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_reader(rdr, &mut msg)?;

    // Deserialize from internal data model into user data type
    let data = msg.data.unwrap();
    T::deserialize(data)
}

#[allow(unused)]
pub fn from_stream<'de, T>(decoder: &mut Decoder, rdr: &mut dyn Read) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_stream(rdr, &mut msg)?;

    // Deserialize from internal data model into user data type
    let data = msg.data.unwrap();
    T::deserialize(data)
}


impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Runtime(msg.to_string())
    }
}
