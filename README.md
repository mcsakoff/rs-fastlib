# FIX/FAST Protocol Decoder
[![Crates.io](https://img.shields.io/crates/v/fastlib?style=flat-square)](https://crates.io/crates/fastlib)
[![Build Status](https://img.shields.io/github/actions/workflow/status/mcsakoff/rs-fastlib/rust.yml?branch=main&style=flat-square)](https://github.com/mcsakoff/rs-fastlib/actions/workflows/rust.yml?query=branch%3Amain)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)

**FAST** (**F**IX **A**dapted for **ST**reaming protocol) is a space and processing efficient encoding method for message oriented data streams.

The FAST protocol has been developed as part of the FIX Market Data Optimization Working Group.
FAST data compression algorithm is designed to optimize electronic exchange of financial data, particularly for high volume,
low latency data dissemination. It significantly reduces bandwidth requirements and latency between sender and receiver.
FAST works especially well at improving performance during periods of peak message rates.

_Technical Specification_: https://www.fixtrading.org/standards/fast-online/  
_Supported version_: 1.x.1


## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
fastlib = "0.1"
```

### Using serde

Not supported yet.

### Using own message factory

Make a new struct that implements `fastlib::MessageFactory` trait:

```rust
use fastlib::{MessageFactory, Value};

pub struct MyMessageFactory {
}

impl MessageFactory for MyMessageFactory {
    // ... your implementation here ...
}
```

You have to implement callbacks that will be called during message decoding:

```rust
pub trait MessageFactory {
    // Process template id
    fn start_template(&mut self, id: u32, name: &str);
    fn stop_template(&mut self);
    
    // Process field value
    fn set_value(&mut self, id: u32, name: &str, value: Option<Value>);
    
    // Process sequence
    fn start_sequence(&mut self, id: u32, name: &str, length: u32);
    fn start_sequence_item(&mut self, index: u32);
    fn stop_sequence_item(&mut self);
    fn stop_sequence(&mut self);
    
    // Process group
    fn start_group(&mut self, name: &str);
    fn stop_group(&mut self);
    
    // Process template ref
    fn start_template_ref(&mut self, name: &str, dynamic: bool);
    fn stop_template_ref(&mut self);
}
```

For examples see implementation for `fastlib::text::TextMessageFactory` or `fastlib::text::JsonMessageFactory` but more likely you will want to construct you own message structs.

Then create a decoder from templates XML file and decode a message:

```rust
use fastlib::Decoder;

// Raw data that contains one message.
let raw_data: Vec<u8> = vec![ ... ];

// Create a decoder from XML templates.
let mut decoder = Decoder::new_from_xml(include_str!("templates.xml")).unwrap();

// Create a message factory.
let mut msg = MyMessageFactory{};

// Decode the message.
decoder.decode_vec(raw_data, &mut msg).unwrap();
```

## Examples

- [fast-tools](https://github.com/mcsakoff/rs-fast-tools)


## TODO

- Serde interface.
- Encoder.


## License

This project is licensed under the [MIT license](LICENSE-MIT).
