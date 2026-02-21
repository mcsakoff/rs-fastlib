use std::io::Read;

use serde::de::Deserialize;

use crate::model::ModelFactory;
use crate::{Decoder, Error, Reader, Result};

/// Decode single message from `Vec<u8>`.
/// # Errors
/// Returns error if message decode failed or if T deserialize failed.
pub fn from_vec<'de, T>(decoder: &mut Decoder, bytes: Vec<u8>) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_vec(bytes, &mut msg)?;

    // Deserialize from internal data model into user data type
    deserialize(msg)
}

/// Decode single message from buffer.
/// Returns the decoded message and number of bytes consumed.
/// # Errors
/// Returns error if message decode failed or if T deserialize failed.
pub fn from_buffer<'de, T>(decoder: &mut Decoder, buffer: &[u8]) -> Result<(T, u64)>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    let n = decoder.decode_buffer(buffer, &mut msg)?;

    // Deserialize from internal data model into user data type
    deserialize(msg).map(|r| (r, n))
}

/// Decode single message from buffer.
/// The `bytes` slice must be consumed completely. It is an error if any bytes left after the message is decoded.
/// # Errors
/// Returns error if message decode failed or if T deserialize failed.
pub fn from_slice<'de, T>(decoder: &mut Decoder, bytes: &[u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_slice(bytes, &mut msg)?;

    // Deserialize from internal data model into user data type
    deserialize(msg)
}

/// Decode single message from `bytes::Bytes`.
/// # Errors
/// Returns error if message decode failed or if T deserialize failed.
#[allow(unused)]
pub fn from_bytes<'de, T>(decoder: &mut Decoder, bytes: &mut bytes::Bytes) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_bytes(bytes, &mut msg)?;

    // Deserialize from internal data model into user data type
    deserialize(msg)
}

/// Decode single message from object that implements `fastlib::Reader` trait.
/// # Errors
/// Returns error if message decode failed or if T deserialize failed.
#[allow(unused)]
pub fn from_reader<'de, T>(decoder: &mut Decoder, rdr: &mut impl Reader) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_reader(rdr, &mut msg)?;

    // Deserialize from internal data model into user data type
    deserialize(msg)
}

/// Decode single message from object that implements `std::io::Read` trait.
/// # Errors
/// Returns error if message decode failed or if T deserialize failed.
#[allow(unused)]
pub fn from_stream<'de, T>(decoder: &mut Decoder, rdr: &mut dyn Read) -> Result<T>
where
    T: Deserialize<'de>,
{
    // Decode FAST message into internal data model
    let mut msg = ModelFactory::new();
    decoder.decode_stream(rdr, &mut msg)?;

    // Deserialize from internal data model into user data type
    deserialize(msg)
}

// Deserializes message to the given type.
// # Errors
// Returns error if deserialization failed.
fn deserialize<'de, T>(msg: ModelFactory) -> Result<T>
where
    T: Deserialize<'de>,
{
    let data = msg
        .data
        .ok_or_else(|| Error::Runtime("No data in message".to_string()))?;
    T::deserialize(data)
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Runtime(msg.to_string())
    }
}
