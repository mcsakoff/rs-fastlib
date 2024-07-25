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
//! ## Serialize/Deserialize using serde
//!
//! For templates defined in XML, e.g.:
//!
//! ```xml
//! <?xml version="1.0" encoding="UTF-8" ?>
//! <templates xmlns="http://www.fixprotocol.org/ns/fast/td/1.1">
//!     <template name="MsgHeader">
//!         <uInt32 id="34" name="MsgSeqNum"/>
//!         <uInt64 id="52" name="SendingTime"/>
//!     </template>
//!     <template id="1" name="MDHeartbeat">
//!         <templateRef name="MsgHeader"/>
//!     </template>
//!     <template id="2" name="MDLogout">
//!         <templateRef name="MsgHeader"/>
//!         <string id="58" name="Text" presence="optional"/>
//!     </template>
//! </templates>
//! ```
//!
//! Define the message types in Rust:
//!
//! ```rust,ignore
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! enum Message {
//!     MDHeartbeat(Heartbeat),
//!     MDLogout(Logout),
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct MsgHeader {
//!     #[serde(rename = "MsgSeqNum")]
//!     msg_seq_num: u32,
//!     #[serde(rename = "SendingTime")]
//!     sending_time: u64,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! #[serde(rename_all = "PascalCase")]
//! struct Heartbeat {
//!     #[serde(flatten)]
//!     msg_header: MsgHeader,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! #[serde(rename_all = "PascalCase")]
//! struct Logout {
//!     #[serde(flatten)]
//!     msg_header: MsgHeader,
//!     text: Option<String>,
//! }
//! ```
//!
//! Some guidelines:
//!
//! * `<templates>` must be implemented as `enum`;
//! * `<decimal>` can be deserialized to `f64` or `fastlib::Decimal` (if you need to preserve original scale);
//! * `<byteVector>` is a `Vec<u8>` and must be prefixed with `#[serde(with = "serde_bytes")]`;
//! * `<sequence>` is a `Vec<SequenceItem>`, where `SequenceItem` is a `struct`;
//! * `<group>` is a nested `struct`;
//! * fields with optional presence are `Option<...>`;
//! * static template reference can be plain fields from the template or flattened `struct`,
//! * dynamic template references must be `Box<Message>` with `#[serde(rename = "templateRef:N")]`, where `N`
//!   is a 0-based index of the `<templateRef>` in its group.
//!
//! To deserialize a message call `fastlib::from_vec`, `fastlib::from_bytes` or `from_stream`:
//!
//! ```rust,ignore
//! use fastlib::Decoder;
//!
//! // Create a decoder from XML templates.
//! let mut decoder = Decoder::new_from_xml(include_str!("templates.xml"))?;
//!
//! // Raw data that contains one message.
//! let raw_data: Vec<u8> = vec![ ... ];
//!
//! // Deserialize a message.
//! let msg: Message = fastlib::from_vec(&mut decoder, raw_data)?;
//! ```
//!
//! To serialize a message call `fastlib::to_vec`, `fastlib::to_bytes` or `to_stream`:
//!
//! ```rust,ignore
//! use fastlib::Encoder;
//!
//! // Create an encoder from XML templates.
//! let mut encoder = Encoder::new_from_xml(include_str!("templates.xml"))?;
//!
//! // Message to serialize.
//! let msg = Message::MDHeartbeat{
//!     Heartbeat {
//!         ...
//!     }
//! };
//!
//! // Serialize a message.
//! let raw: Vec<u8> = fastlib::to_vec(&mut encoder, &msg)?;
//! ```
//!
//!
//! ## Decode to JSON
//!
//! ```rust,ignore
//! use fastlib::Decoder;
//! use fastlib::JsonMessageFactory;
//!
//! // Raw data that contains one message.
//! let raw_data: Vec<u8> = vec![ ... ];
//!
//! // Create a decoder from XML templates.
//! let mut decoder = Decoder::new_from_xml(include_str!("templates.xml"))?;
//!
//! // Create a JSON message factory.
//! let mut msg = JsonMessageFactory::new();
//!
//! // Decode the message.
//! decoder.decode_vec(raw_data, &mut msg)?;
//!
//! println!("{}", msg.json);
//! ```
//!
//! ## Decode using own message factory
//!
//! Make a new struct that implements [`fastlib::MessageFactory`][crate::MessageFactory] trait:
//!
//! ```rust,ignore
//! use fastlib::{MessageFactory, Value};
//!
//! // Message factory struct that will build a message during decoding.
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
//! let mut decoder = Decoder::new_from_xml(include_str!("templates.xml"))?;
//!
//! // Create a message factory.
//! let mut msg = MyMessageFactory{};
//!
//! // Decode the message.
//! decoder.decode_vec(raw_data, &mut msg)?;
//! ```
//!
//! For message factory implementations see [`fastlib::text::TextMessageFactory`][crate::TextMessageFactory] or
//! [`crate::text::JsonMessageFactory`][crate::JsonMessageFactory] but more likely you will want to construct
//! you own message structs.
//!
pub use base::{decimal::Decimal, value::Value, value::ValueType};
pub use base::message::{MessageFactory, MessageVisitor};
pub use decoder::{decoder::Decoder, reader::Reader};
pub use encoder::{encoder::Encoder, writer::Writer};
pub use text::{JsonMessageFactory, TextMessageFactory, TextMessageVisitor};

#[cfg(feature = "serde")]
pub use de::*;
#[cfg(feature = "serde")]
pub use ser::*;

mod base;
mod common;
mod decoder;
mod encoder;
mod utils;
mod text;

#[cfg(feature = "serde")]
mod de;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod ser;

#[cfg(test)]
mod tests;


pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    ///! Errors happened due to malformed XML.
    #[error("Static Error: {0}")]
    Static(String),

    ///! Errors happened due to malformed binary data.
    #[error("Dynamic Error: {0}")]
    Dynamic(String),

    ///! Errors happened due to decoding algorithm.
    #[error("Runtime Error: {0}")]
    Runtime(String),

    ///! End of file/stream reached.
    #[error("End of file/stream reached")]
    Eof,

    ///! Unexpected end of file/stream reached.
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
