# FIX/FAST Protocol Decoder

**FAST** (**F**IX **A**dapted for **ST**reaming protocol) is a space and processing efficient encoding method for message oriented data streams.

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
    // process template id
    fn start_template(&mut self, id: u32, name: &str);
    fn stop_template(&mut self);
    // process field value
    fn set_value(&mut self, id: u32, name: &str, value: Option<Value>);
    // process sequence
    fn start_sequence(&mut self, id: u32, name: &str, length: u32);
    fn start_sequence_item(&mut self, index: u32);
    fn stop_sequence_item(&mut self);
    fn stop_sequence(&mut self);
    // process group
    fn start_group(&mut self, name: &str);
    fn stop_group(&mut self);
    // process template ref
    fn start_template_ref(&mut self, name: &str, dynamic: bool);
    fn stop_template_ref(&mut self);
}
```

For examples see implementation for `fastlib::text::TextMessageFactory` or `fastlib::text::JsonMessageFactory` but more likely you will want to construct you own message structs.

Then create a decoder from templates XML file and decode a message:

```rust
fn decode_message(input: Vec<u8>) {
    let mut decoder = Decoder::new_from_xml(include_str!("templates.xml")).unwrap();
    let mut msg = MyMessageFactory{};
    decoder.decode_vec(input, &mut msg).unwrap();
}
```


## TODO

- Serde interface.
- Encoder.


## License

This project is licensed under the [MIT license](LICENSE-MIT).
