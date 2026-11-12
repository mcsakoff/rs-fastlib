# Changelog

## 0.4.0
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
