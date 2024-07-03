//! # Integer Field Instructions
//!
//! Integer Numbers have unlimited size in the transfer encoding. However, applications typically use
//! fixed sizes for integers. An integer field instruction must therefore specify the bounds of the integer.
//! The encoding and decoding of a value is not affected by the size of the integer.
//!
use bytes::Buf;

use crate::{Error, Result};

/// A trait that provides methods for reading basic primitive types.
pub trait Reader {
    /// Read the presence map. Return the bitmap and the number of bits in the bitmap.
    fn read_presence_map(&mut self) -> Result<(u64, u8)>;
    fn read_uint(&mut self) -> Result<u64>;
    fn read_uint_nullable(&mut self) -> Result<Option<u64>>;
    fn read_int(&mut self) -> Result<i64>;
    fn read_int_nullable(&mut self) -> Result<Option<i64>>;
    fn read_ascii_string(&mut self) -> Result<String>;
    fn read_ascii_string_nullable(&mut self) -> Result<Option<String>>;
    fn read_unicode_string(&mut self) -> Result<String>;
    fn read_unicode_string_nullable(&mut self) -> Result<Option<String>>;
    fn read_bytes(&mut self) -> Result<Vec<u8>>;
    fn read_bytes_nullable(&mut self) -> Result<Option<Vec<u8>>>;
}

impl Reader for bytes::Bytes {
    fn read_presence_map(&mut self) -> Result<(u64, u8)> {
        let mut bitmap: u64 = 0;
        let mut size: u8 = 0;
        loop {
            let byte = read_u8(self)?;
            bitmap <<= 7;
            bitmap |= (byte & 0x7f) as u64;
            size += 7;

            if byte & 0x80 == 0x80 {
                return Ok((bitmap, size))
            }
        }
    }

    fn read_uint(&mut self) -> Result<u64> {
        let mut value: u64 = 0;
        loop {
            let byte = read_u8(self)?;
            value <<= 7;
            value |= (byte & 0x7f) as u64;
            if byte & 0x80 == 0x80 {
                return Ok(value)
            }
        }
    }

    fn read_uint_nullable(&mut self) -> Result<Option<u64>> {
        let value = self.read_uint()?;
        if value == 0 {
            Ok(None)
        } else {
            Ok(Some(value - 1))
        }
    }

    fn read_int(&mut self) -> Result<i64> {
        let mut value: i64 = 0;

        let mut byte = read_u8(self)?;
        if byte & 0x40 != 0 { // Negative Integer
            value = -1;
        }
        loop {
            value <<= 7;
            value |= (byte & 0x7f) as i64;

            if byte & 0x80 == 0x80 {
                return Ok(value)
            }
            byte = read_u8(self)?;
        }
    }

    fn read_int_nullable(&mut self) -> Result<Option<i64>> {
        let value = self.read_int()?;
        if value > 0 {
            Ok(Some(value - 1))
        } else if value < 0 {
            Ok(Some(value))
        } else  {
            Ok(None)
        }
    }

    fn read_ascii_string(&mut self) -> Result<String> {
        let mut byte = read_u8(self)?;
        if byte == 0x80 {
            return Ok(String::new());
        }

        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.push(byte & 0x7f);
            if byte & 0x80 == 0x80 {
                break
            }
            byte = read_u8(self)?;
        }
        // SAFETY: `buf` contains ASCII 7-bit characters
        unsafe { Ok(String::from_utf8_unchecked(buf)) }
    }

    fn read_ascii_string_nullable(&mut self) -> Result<Option<String>> {
        let mut byte = read_u8(self)?;

        if byte == 0x80 {
            return Ok(None);
        } else if byte == 0x00 {
            byte = read_u8(self)?;
            if byte == 0x80 {
                return Ok(Some(String::new()));
            }
        }

        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.push(byte & 0x7f);
            if byte & 0x80 == 0x80 {
                break
            }
            byte = read_u8(self)?;
        }
        // SAFETY: `buf` contains ASCII 7-bit characters
        unsafe { Ok(Some(String::from_utf8_unchecked(buf))) }
    }

    fn read_unicode_string(&mut self) -> Result<String> {
        Ok(String::from_utf8(self.read_bytes()?)?)
    }

    fn read_unicode_string_nullable(&mut self) -> Result<Option<String>> {
        match self.read_bytes_nullable()? {
            None => Ok(None),
            Some(bytes) => {
                Ok(Some(String::from_utf8(bytes)?))
            }
        }
    }

    fn read_bytes(&mut self) -> Result<Vec<u8>> {
        let length = self.read_uint()?;
        let mut buf = Vec::with_capacity(length as usize);
        for _ in 0..length {
            buf.push(read_u8(self)?);
        }
        Ok(buf)
    }

    fn read_bytes_nullable(&mut self) -> Result<Option<Vec<u8>>> {
        match self.read_uint_nullable()? {
            None => Ok(None),
            Some(length) => {
                let mut buf = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    buf.push(read_u8(self)?);
                }
                Ok(Some(buf))
            }
        }
    }
}

#[inline]
fn read_u8(bytes: &mut bytes::Bytes) -> Result<u8> {
    if bytes.is_empty() {
        return Err(Error::Dynamic("Unexpected end of input".to_string()));
    }
    let b = bytes.get_u8();
    Ok(b)
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
}
