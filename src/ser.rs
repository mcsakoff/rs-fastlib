use std::io::Write;

use serde::ser::Serialize;

use crate::{Encoder, Error, Result, Writer};
use crate::model::ModelVisitor;
use crate::model::template::TemplateData;

#[allow(unused)]
pub fn to_vec<T>(encoder: &mut Encoder, value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    // Serialise user data into internal data model
    let mut data = TemplateData::new_empty();
    value.serialize(&mut data)?;

    // Encode FAST message from internal data model
    let mut msg = ModelVisitor::new(data);
    encoder.encode_vec(&mut msg)
}

#[allow(unused)]
pub fn to_bytes<T>(encoder: &mut Encoder, value: &T) -> Result<bytes::BytesMut>
where
    T: ?Sized + Serialize,
{
    // Serialise user data into internal data model
    let mut data = TemplateData::new_empty();
    value.serialize(&mut data)?;

    // Encode FAST message from internal data model
    let mut msg = ModelVisitor::new(data);
    encoder.encode_bytes(&mut msg)
}

#[allow(unused)]
pub fn to_writer<T>(encoder: &mut Encoder, wrt: &mut impl Writer, value: &T) -> Result<()>
where
    T: ?Sized + Serialize,
{
    // Serialise user data into internal data model
    let mut data = TemplateData::new_empty();
    value.serialize(&mut data)?;

    // Encode FAST message from internal data model
    let mut msg = ModelVisitor::new(data);
    encoder.encode_writer(wrt, &mut msg)
}

#[allow(unused)]
pub fn to_stream<T>(encoder: &mut Encoder, wrt: &mut dyn Write, value: &T) -> Result<()>
where
    T: ?Sized + Serialize,
{
    // Serialise user data into internal data model
    let mut data = TemplateData::new_empty();
    value.serialize(&mut data)?;

    // Encode FAST message from internal data model
    let mut msg = ModelVisitor::new(data);
    encoder.encode_stream(wrt, &mut msg)
}

/// Serialize user data into pre-allocated buffer.
/// Returns number of bytes written.
#[allow(unused)]
pub fn to_buffer<T>(encoder: &mut Encoder, buffer: &mut [u8], value: &T) -> Result<usize>
where
    T: ?Sized + Serialize,
{
    // Serialise user data into internal data model
    let mut data = TemplateData::new_empty();
    value.serialize(&mut data)?;

    // Encode FAST message from internal data model
    let mut msg = ModelVisitor::new(data);
    encoder.encode_buffer(buffer, &mut msg)
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display
    {
        Error::Runtime(msg.to_string())
    }
}
