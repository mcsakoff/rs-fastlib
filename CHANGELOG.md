# Changelog

## 0.4.0
- **Breaking:** `MessageVisitor::get_template_name`, `MessageVisitor::get_value` and
  `MessageVisitor::select_template_ref` returns `Cow`, so visitors can lend data to the encoder without copying it.
- **Breaking:** following the template schema, `dictionary` and `key` are read from operator elements
  (`<copy>`, `<increment>`, `<delta>`, `<tail>`), including decimal `<exponent>`/`<mantissa>` operators.
  Both attributes are rejected on filed elements and other operators. `dictionary` remains valid on
  `<templates>`, `<template>`, `<sequence>` and `<group>`.
- FAST 1.1 compliance fixes.
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
