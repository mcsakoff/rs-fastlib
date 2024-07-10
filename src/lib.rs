//! **FAST** (**F**IX **A**dapted for **ST**reaming protocol) is a space and processing
//! efficient encoding method for message oriented data streams.
//!
//! The FAST protocol has been developed as part of the FIX Market Data Optimization Working Group.
//! FAST data compression algorithm is designed to optimize electronic exchange of financial data, particularly
//! for high volume, low latency data dissemination. It significantly reduces bandwidth requirements and latency
//! between sender and receiver. FAST works especially well at improving performance during periods of peak message
//! rates.
//!
//! Fot the FAST protocol description see [technical specification](https://www.fixtrading.org/standards/fast-online/).
//!
//! The `fastlib` crate provides a decoder for FAST protocol messages.
//!
//! # Usage
//!
//! ## Using serde
//!
//! Not supported yet.
//!
//! ## Using own message factory
//!
//! Make a new struct that implements [`fastlib::MessageFactory`][crate::MessageFactory] trait:
//!
//! ```rust,ignore
//! use fastlib::{MessageFactory, Value};
//!
//! // Message factory stuct that will build a message during decoding.
//! pub struct MyMessageFactory {
//! }
//!
//! // Callback functions that will be called for each message during decoding process.
//! impl MessageFactory for MyMessageFactory {
//!     // ... your implementation here ...
//! }
//!```
//! Then create a decoder from templates XML file and decode a message:
//!
//! ```rust,ignore
//! use fastlib::Decoder;
//!
//! // Raw data that contains one message.
//! let raw_data: Vec<u8> = vec![ ... ];
//!
//! // Create a decoder from XML templates.
//! let mut decoder = Decoder::new_from_xml(include_str!("templates.xml")).unwrap();
//!
//! // Create a message factory.
//! let mut msg = MyMessageFactory{};
//!
//! // Decode the message.
//! decoder.decode_vec(raw_data, &mut msg).unwrap();
//! ```
//!
//! For message factory implementations see [`fastlib::text::TextMessageFactory`][crate::TextMessageFactory] or
//! [`crate::text::JsonMessageFactory`][crate::JsonMessageFactory] but more likely you will want to construct you own message
//! structs.
//!
mod base;
mod decoder;
mod utils;
mod text;
mod model;

#[cfg(test)]
mod tests;

pub use decoder::{decoder::Decoder, reader::Reader};
pub use base::{value::Value, decimal::Decimal};
pub use base::message::MessageFactory;
pub use text::{TextMessageFactory, JsonMessageFactory};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors happened due to malformed XML.
    #[error("Static Error: {0}")]
    Static(String),

    /// Errors happened due to malformed binary data.
    #[error("Dynamic Error: {0}")]
    Dynamic(String),

    /// Errors happened due to decoding algorithm.
    #[error("Runtime Error: {0}")]
    Runtime(String),

    /// End of file/stream reached.
    #[error("End of file/stream reached")]
    Eof,

    /// Unexpected end of file/stream reached.
    #[error("Unexpected end of file/stream reached")]
    UnexpectedEof,

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse float: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    XMLTreeError(#[from] roxmltree::Error),
}
