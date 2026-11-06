//! # Integer Field Instructions
//!
//! Integer Numbers have unlimited size in the transfer encoding. However, applications typically use
//! fixed sizes for integers. An integer field instruction must therefore specify the bounds of the integer.
//! The encoding and decoding of a value is not affected by the size of the integer.
//!
use std::io::{ErrorKind, Read};

use bytes::Buf;

use crate::common::definitions::MAX_PMAP_BITS;
use crate::{Error, Result};

/// A trait that provides methods for reading basic primitive types.
pub trait Reader {
    /// Reads a single byte.
    /// # Errors
    /// Do not return [`Error::Eof`][crate::Error::Eof] from this method.
    /// Return [`Error::UnexpectedEof`][crate::Error::UnexpectedEof] instead.
    fn read_u8(&mut self) -> Result<u8>;

    /// Read the presence map. Return the bitmap and the number of bits in the bitmap.
    /// # Errors
    /// In case of error, return [`Error::Eof`][crate::Error::Eof] if the end of the stream is reached at the first byte
    /// of the presence map. Otherwise, return any other error, e.g.: [`Error::UnexpectedEof`][crate::Error::UnexpectedEof].
    fn read_presence_map(&mut self) -> Result<(u64, u8)> {
        let mut bitmap: u64 = 0;
        let mut size: u8 = 0;
        let mut byte = match self.read_u8() {
            Ok(b) => b,
            Err(Error::UnexpectedEof) => return Err(Error::Eof),
            Err(e) => return Err(e),
        };
        loop {
            bitmap <<= 7;
            bitmap |= u64::from(byte & 0x7f);
            size += 7;

            if byte & 0x80 == 0x80 {
                return Ok((bitmap, size));
            }
            if usize::from(size) >= MAX_PMAP_BITS {
                return Err(Error::Dynamic(format!(
                    "presence map is longer that {MAX_PMAP_BITS} bits"
                )));
            }
            byte = self.read_u8()?;
        }
    }

    /// Returns decoded non-nullable 64 bits unsigned integer.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_uint(&mut self) -> Result<u64> {
        let mut value: u64 = 0;
        loop {
            let byte = self.read_u8()?;
            if value > u64::MAX >> 7 {
                return Err(integer_overflow()); // [ERR D2]
            }
            value <<= 7;
            value |= u64::from(byte & 0x7f);
            if byte & 0x80 == 0x80 {
                return Ok(value);
            }
        }
    }

    /// Returns decoded nullable 64 bits unsigned integer.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_uint_nullable(&mut self) -> Result<Option<u64>> {
        let value = self.read_uint()?;
        if value == 0 {
            Ok(None)
        } else {
            Ok(Some(value - 1))
        }
    }

    /// Returns decoded non-nullable 64 bits signed integer.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_int(&mut self) -> Result<i64> {
        let mut value: i64 = 0;

        let mut byte = self.read_u8()?;
        if byte & 0x40 != 0 {
            // Negative Integer
            value = -1;
        }
        loop {
            if !(i64::MIN >> 7..=i64::MAX >> 7).contains(&value) {
                return Err(integer_overflow()); // [ERR D2]
            }
            value <<= 7;
            value |= i64::from(byte & 0x7f);

            if byte & 0x80 == 0x80 {
                return Ok(value);
            }
            byte = self.read_u8()?;
        }
    }

    /// Returns decoded nullable 64 bits signed integer.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_int_nullable(&mut self) -> Result<Option<i64>> {
        let value = self.read_int()?;
        match value {
            0 => Ok(None),
            value if value < 0 => Ok(Some(value)),
            value => Ok(Some(value - 1)),
        }
    }

    /// Returns decoded non-nullable ASCII encoded string.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_ascii_string(&mut self) -> Result<String> {
        let mut byte = self.read_u8()?;
        if byte == 0x80 {
            return Ok(String::new());
        }

        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.push(byte & 0x7f);
            if byte & 0x80 == 0x80 {
                break;
            }
            byte = self.read_u8()?;
        }
        // SAFETY: `buf` contains ASCII 7-bit characters
        unsafe { Ok(String::from_utf8_unchecked(buf)) }
    }

    /// Returns decoded nullable ASCII encoded string.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_ascii_string_nullable(&mut self) -> Result<Option<String>> {
        let mut byte = self.read_u8()?;

        if byte == 0x80 {
            return Ok(None);
        } else if byte == 0x00 {
            byte = self.read_u8()?;
            if byte == 0x80 {
                return Ok(Some(String::new()));
            }
        }

        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.push(byte & 0x7f);
            if byte & 0x80 == 0x80 {
                break;
            }
            byte = self.read_u8()?;
        }
        // SAFETY: `buf` contains ASCII 7-bit characters
        unsafe { Ok(Some(String::from_utf8_unchecked(buf))) }
    }

    /// Returns decoded non-nullable unicode encoded string.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_unicode_string(&mut self) -> Result<String> {
        Ok(String::from_utf8(self.read_bytes()?)?)
    }

    /// Returns decoded nullable unicode encoded string.
    /// # Errors
    /// Returns error if can not read bytes len or bytes len mismatch.
    fn read_unicode_string_nullable(&mut self) -> Result<Option<String>> {
        match self.read_bytes_nullable()? {
            None => Ok(None),
            Some(bytes) => Ok(Some(String::from_utf8(bytes)?)),
        }
    }

    /// Returns decoded non-nullable bytes.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_bytes(&mut self) -> Result<Vec<u8>> {
        let length = self.read_uint()?;
        let mut buf = Vec::with_capacity(length as usize);
        for _ in 0..length {
            buf.push(self.read_u8()?);
        }
        Ok(buf)
    }

    /// Returns decoded nullable bytes.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_bytes_nullable(&mut self) -> Result<Option<Vec<u8>>> {
        match self.read_uint_nullable()? {
            None => Ok(None),
            Some(length) => {
                let mut buf = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    buf.push(self.read_u8()?);
                }
                Ok(Some(buf))
            }
        }
    }
}

#[cold]
fn integer_overflow() -> Error {
    Error::Dynamic("integer value does not fit into 64 bits".to_string())
}

impl Reader for bytes::Bytes {
    fn read_u8(&mut self) -> Result<u8> {
        if self.is_empty() {
            return Err(Error::UnexpectedEof);
        }
        let b = self.get_u8();
        Ok(b)
    }
}

/// Wrapper around `std::io::Read` that implements [`fastlib::Reader`][crate::decoder::reader::Reader].
pub(crate) struct StreamReader<'a> {
    stream: &'a mut dyn Read,
}

impl<'a> StreamReader<'a> {
    pub fn new(stream: &'a mut dyn Read) -> Self {
        Self { stream }
    }
}

impl Reader for StreamReader<'_> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        match self.stream.read_exact(&mut buf) {
            Ok(()) => {}
            Err(err) => {
                if err.kind() == ErrorKind::UnexpectedEof {
                    return Err(Error::UnexpectedEof);
                }
                return Err(Error::Dynamic(format!("Stream read error: {err}")));
            }
        }
        Ok(buf[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_presence_map() {
        struct TestCase {
            input: Vec<u8>,
            pmap: (u64, u8),
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                pmap: (0b0, 7),
            },
            TestCase {
                input: vec![0x81],
                pmap: (0b1, 7),
            },
            TestCase {
                input: vec![0x0f, 0x8f],
                pmap: (0b11110001111, 14),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let pmap = buf.read_presence_map().unwrap();
            assert_eq!(pmap, tc.pmap);
        }
    }

    #[test]
    fn read_uint() {
        struct TestCase {
            input: Vec<u8>,
            value: u64,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: 0,
            },
            TestCase {
                input: vec![0x81],
                value: 1,
            },
            TestCase {
                input: vec![0x39, 0x45, 0xa3],
                value: 942755,
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_uint().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_uint_nullable() {
        struct TestCase {
            input: Vec<u8>,
            value: Option<u64>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: None,
            },
            TestCase {
                input: vec![0x81],
                value: Some(0),
            },
            TestCase {
                input: vec![0x39, 0x45, 0xa4],
                value: Some(942755),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_uint_nullable().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_int() {
        struct TestCase {
            input: Vec<u8>,
            value: i64,
        }
        let test_cases: Vec<TestCase> = vec![
            // Mandatory Positive Number
            TestCase {
                input: vec![0x39, 0x45, 0xa3],
                value: 942755,
            },
            // Mandatory Negative Number
            TestCase {
                input: vec![0x7c, 0x1b, 0x1b, 0x9d],
                value: -7942755,
            },
            // Mandatory Positive Number with sign-bit extension
            TestCase {
                input: vec![0x00, 0x40, 0x81],
                value: 8193,
            },
            // Mandatory Negative Number with sign-bit extension
            TestCase {
                input: vec![0x7f, 0x3f, 0xff],
                value: -8193,
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_int().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_int_nullable() {
        struct TestCase {
            input: Vec<u8>,
            value: Option<i64>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: None,
            },
            // Optional Positive Number
            TestCase {
                input: vec![0x39, 0x45, 0xa4],
                value: Some(942755),
            },
            // Optional Negative Number
            TestCase {
                input: vec![0x46, 0x3a, 0xdd],
                value: Some(-942755),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_int_nullable().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_ascii_string() {
        struct TestCase {
            input: Vec<u8>,
            value: String,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: "".to_string(),
            },
            TestCase {
                input: vec![0x41, 0x42, 0xc3],
                value: "ABC".to_string(),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_ascii_string().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_ascii_string_nullable() {
        struct TestCase {
            input: Vec<u8>,
            value: Option<String>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: None,
            },
            TestCase {
                input: vec![0x00, 0x80],
                value: Some("".to_string()),
            },
            TestCase {
                input: vec![0x41, 0x42, 0xc3],
                value: Some("ABC".to_string()),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_ascii_string_nullable().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_unicode_string() {
        struct TestCase {
            input: Vec<u8>,
            value: String,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: "".to_string(),
            },
            TestCase {
                input: vec![0x83, 0x41, 0x42, 0x43],
                value: "ABC".to_string(),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_unicode_string().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_unicode_string_nullable() {
        struct TestCase {
            input: Vec<u8>,
            value: Option<String>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: None,
            },
            TestCase {
                input: vec![0x81],
                value: Some("".to_string()),
            },
            TestCase {
                input: vec![0x84, 0x41, 0x42, 0x43],
                value: Some("ABC".to_string()),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_unicode_string_nullable().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_bytes() {
        struct TestCase {
            input: Vec<u8>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: vec![],
            },
            TestCase {
                input: vec![0x83, 0x41, 0x42, 0x43],
                value: vec![0x41, 0x42, 0x43],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_bytes().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_bytes_nullable() {
        struct TestCase {
            input: Vec<u8>,
            value: Option<Vec<u8>>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![0x80],
                value: None,
            },
            TestCase {
                input: vec![0x81],
                value: Some(vec![]),
            },
            TestCase {
                input: vec![0x84, 0x41, 0x42, 0x43],
                value: Some(vec![0x41, 0x42, 0x43]),
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::Bytes::from(tc.input);
            let value = buf.read_bytes_nullable().unwrap();
            assert_eq!(value, tc.value);
        }
    }

    #[test]
    fn read_integer_limits() {
        use crate::encoder::writer::Writer;

        for v in [0, 1, u64::from(u32::MAX), u64::MAX >> 7, u64::MAX] {
            let mut buf = bytes::BytesMut::new();
            buf.write_uint(v).unwrap();
            assert_eq!(buf.freeze().read_uint().unwrap(), v);
        }

        for v in [0, -1, 1, i64::MIN >> 7, i64::MAX >> 7, i64::MIN, i64::MAX] {
            let mut buf = bytes::BytesMut::new();
            buf.write_int(v).unwrap();
            assert_eq!(buf.freeze().read_int().unwrap(), v);
        }
    }

    #[test]
    fn read_integer_overflow() {
        // 2^64 doesn't fit into u64
        let mut buf = bytes::Bytes::from(vec![0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0x80]);
        assert!(matches!(buf.read_uint(), Err(Error::Dynamic(_))));

        // 2^63 doesn't fit into i64
        let mut buf = bytes::Bytes::from(vec![0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x80]);
        assert!(matches!(buf.read_int(), Err(Error::Dynamic(_))));

        // -2^63 - 1 doesn't fit into i64
        let mut buf = bytes::Bytes::from(vec![
            0x7e, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
        ]);
        assert!(matches!(buf.read_int(), Err(Error::Dynamic(_))));
    }

    #[test]
    fn read_pmap_long() {
        let mut buf =
            bytes::Bytes::from(vec![0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff]);
        assert_eq!(buf.read_presence_map().unwrap(), (u64::MAX >> 1, 63));

        let mut buf = bytes::Bytes::from(vec![
            0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xff,
        ]);
        assert!(matches!(buf.read_presence_map(), Err(Error::Dynamic(_))));
    }
}
