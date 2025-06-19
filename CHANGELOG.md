# Changelog

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
