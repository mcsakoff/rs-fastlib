# Changelog

## 0.4.1
- **Breaking:** `Delta` has a new `Decimal` variant (exponent and mantissa deltas).
- **Breaking:** `<length>`, `<exponent>` and `<mantissa>` are rejected with a static error where the template
  schema doesn't allow them: as instructions of a template, group or sequence, and `<length>` anywhere
  but in front of the sequence instructions (FAST 1.1 Appendix 1, [ERR S1]).
- **Breaking:** a root element in a foreign namespace is rejected with a static error instead of loading
  an empty set of templates. The template definition namespace `http://www.fixprotocol.org/ns/fast/td/1.1`
  and documents without namespace are accepted (FAST 1.1 §3.1).
- **Breaking:** templates with the same `id` are rejected with a static error instead of the last one
  silently winning.
- Fix: `<delta/>` on a whole `<decimal>` uses the decimal's own previous value, so it shares the previous value
  with other operators of the same key (e.g. `<copy key="px"/>`), as required by FAST 1.1 §6.3.1
  ([#11](https://github.com/mcsakoff/rs-fastlib/issues/11)).
- Fix: a sequence `<length>` and a decimal `<exponent>` are validated after they inherit the presence of their
  field, so an optional sequence or decimal accepts `<default/>` without an initial value (FAST 1.1 §6.3.4)
  ([#12](https://github.com/mcsakoff/rs-fastlib/issues/12)).
- Fix: an unnamed sequence `<length>` uses its generated name as the default dictionary key, so the length
  operators of different sequences no longer share one previous value (FAST 1.1 §6.2.5, §6.3.1)
  ([#13](https://github.com/mcsakoff/rs-fastlib/issues/13)).
- Fix: the decoder reports [ERR D4] when a value present in the stream would overwrite a dictionary entry of
  a different type, and the encoder reports [ERR D4] as `Error::Dynamic` instead of `Error::Runtime`
  (FAST 1.1 §6.3.1)
  ([#14](https://github.com/mcsakoff/rs-fastlib/issues/14)).
- Fix: a `<decimal>` with more than one operator, `<exponent>` or `<mantissa>` is rejected with a static error
  instead of the last one silently winning (FAST 1.1 Appendix 1)
  ([#15](https://github.com/mcsakoff/rs-fastlib/issues/15)).
- Fix: generated dictionary keys (template identifier, unnamed sequence `<length>`, decimal `<exponent>` and
  `<mantissa>`) never share a previous value with fields or keys of the same name specified in templates,
  e.g. a field named `Px:exponent` (FAST 1.1 §6.2.2, §6.2.5, §10).
- Fix: the encoder rescales a decimal to the exponent of `<exponent><constant/></exponent>`, e.g. `1.5` is
  encoded with the constant exponent `-2` as mantissa `150`. A value that cannot be represented with
  the constant exponent is reported as [ERR D3] `Error::Dynamic` instead of `Error::Runtime`
  (FAST 1.1 §6.2.2).
- Fix: `TextMessageVisitor` parses messages, groups and sequence items without present fields, e.g. `T=<>`
  or `T=<G=<>|S=<><A=1>>`, as `TextMessageFactory` produces them.
- Performance: the serde decoder (`from_slice`, `from_bytes`, …) no longer allocates a string for the name 
  of every decoded field.
- Performance: dictionary entries are accessed faster. The type of a previous value [ERR D4] is checked
  at runtime only for the entries that fields of different types share (this is resolved when templates
  are loaded), and entries of `global` and user defined dictionaries are allocated up front.
- Performance: `Encoder::encode_vec` (and `encode_bytes`, `to_vec`, `to_bytes`) allocates the buffer for
  the largest recent messages, so it is not reallocated when a message is longer than the previous one.
- Performance: `<delta/>` and `<tail/>` on Unicode strings validate only the changed part of the string
  as UTF-8 instead of the whole combined value.

## 0.4.0 (yanked)
- **Breaking:** `MessageFactory::set_value` takes the value as `Option<Cow<'_, Value>>`, so the decoder lends
  previous and initial values instead of copying them. Use `Cow::into_owned` where an owned value is needed.
- **Breaking:** `MessageVisitor::get_template_name`, `MessageVisitor::get_value` and
  `MessageVisitor::select_template_ref` now returns `Cow`.
- **Breaking:** following the template schema, `dictionary` and `key` are read from operator elements
  (`<copy>`, `<increment>`, `<delta>`, `<tail>`), including decimal `<exponent>`/`<mantissa>` operators.
  Both attributes are rejected on filed elements and other operators.
- **Breaking:** `Value::find_delta` returns new `Delta` type and `Value::apply_delta`/`Value::apply_delta_in_place` take
  `&Delta`; `Value::find_delta_ref` and `Value::find_delta` return `Result` and fail on type mismatch.
  Integer deltas are `i128`, so deltas of 64-bit fields can use the full 65-bit range.
- **Breaking:** string conversions (`Decimal::from_string`, `Value::set_from_string`, `ValueType::str_to_value`)
  trim whitespace from integers and decimals, accept `.5`, check the decimal exponent range [-63 … 63],
  reject non-ASCII ASCII strings and strip any whitespace from hex strings.
- **Breaking:** a named `<length>` of `<byteVector>` or unicode `<string>` is decoded as an extra `uInt32` field.
- **Breaking:** `<increment/>` on a whole `<decimal>` is rejected.
- FAST 1.1 compliance fixes.
- AI generated compliance tests.
- Performance improvements and fixes.
- Add benchmarks.

## 0.3.8
- Fix for processing `typeRef` element.

## 0.3.7
- Context performance improvements.

## 0.3.6
- Add some `rust_decimal` support.
- Optimized encoding algorithms.
- Format and Clippy pedantic fixes.

## 0.3.5
- Un-deprecate `from_vec` and `Decoder::decode_vec()`.
- Minor code style changes.

## 0.3.4
- Add functions and methods to decode from a `&[u8]` slice.
- Add functions and methods to encode into pre-allocated `&mut [u8]` buffer.

## 0.3.3
- Make `Decimal` derive `Eq` and `Hash`.

## 0.3.2
- Libraries updated to the latest version.

## 0.3.1
- Fix presence map encoding with trailing zeros.
- Add encoding text messages.

## 0.3.0
- Implement encoding.
- Fix decoder context issues.

## 0.2.1
- Add deserialization with serde.
- Fix decoding optional groups.

## 0.2.0
- Add stream decoding method.
- Introduce `Error::Eof` and `Error::Unexpected` errors.
- Return decimal values as `Decimal` instead of `f64`.

## 0.1.0
- Initial release
