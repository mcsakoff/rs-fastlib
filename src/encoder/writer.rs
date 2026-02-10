use std::io::Write;

use bytes::BufMut;

use crate::{Error, Result};

/// A trait that provides methods for writing basic primitive types.
pub trait Writer {
    fn write_u8(&mut self, value: u8) -> Result<()>;

    /// Implement this method for more efficient writing of multiple bytes.
    fn write_buf(&mut self, buf: &[u8]) -> Result<()> {
        for b in buf {
            self.write_u8(*b)?;
        }
        Ok(())
    }

    /// Write the presence map.
    fn write_presence_map(&mut self, bitmap: u64, size: u8) -> Result<()> {
        if size == 0 {
            self.write_u8(0x80)?;
            return Ok(());
        }
        if !size.is_multiple_of(7) {
            return Err(Error::Runtime("write_presence_map: size must be multiple of 7".to_string()));
        }

        let mut trim = true;
        let len = (size / 7) as usize;
        let mut bitmap = bitmap;
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        for _ in 0..len {
            let b7 = (bitmap & 0x7f) as u8;
            if trim && b7 == 0 {
                // trim trailing zeros
            } else {
                buf.push(b7);
                trim = false;
            }
            bitmap >>= 7;
        }
        if buf.is_empty() {
            buf.push(0x00);
        }
        // set stop bit
        *buf.get_mut(0).unwrap() |= 0x80;
        buf.reverse();
        self.write_buf(&buf)
    }

    fn write_uint(&mut self, value: u64) -> Result<()> {
        let mut value = value;
        let mut buf: Vec<u8> = Vec::with_capacity(10);
        buf.push(((value & 0x7f) as u8) | 0x80);
        loop {
            value >>= 7;
            if value == 0 {
                break;
            }
            buf.push((value & 0x7f) as u8);
        }
        buf.reverse();
        self.write_buf(&buf)
    }

    fn write_uint_nullable(&mut self, value: Option<u64>) -> Result<()> {
        match value {
            None => self.write_uint(0),
            Some(v) => self.write_uint(v + 1),
        }
    }

    fn write_int(&mut self, value: i64) -> Result<()> {
        let is_pos = value >= 0;
        let mut buf: Vec<u8> = Vec::with_capacity(10);
        let mut value = value;
        loop {
            let b7 = (value & 0x7f) as u8;
            buf.push(b7);
            value >>= 7;
            if is_pos {
                // stop condition for positive numbers
                if value == 0 && (b7 & 0x40 == 0) {
                    break;
                }
            } else {
                // stop condition for negative numbers
                if value == -1 && (b7 & 0x40 != 0) {
                    break;
                }
            }
        }
        // set stop bit
        *buf.get_mut(0).unwrap() |= 0x80;

        buf.reverse();
        self.write_buf(&buf)
    }

    fn write_int_nullable(&mut self, value: Option<i64>) -> Result<()> {
        match value {
            None => self.write_int(0),
            Some(v) if v >= 0 => self.write_int(v + 1),
            Some(v) => self.write_int(v),
        }
    }

    fn write_ascii_string(&mut self, value: &str) -> Result<()> {
        if value.is_empty() {
            self.write_u8(0x80)
        } else {
            self._write_ascii_str(value)
        }
    }

    fn write_ascii_string_nullable(&mut self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                self.write_u8(0x80)
            }
            Some(s) => {
                if s.is_empty() {
                    self.write_buf(&[0x00, 0x80])
                } else {
                    self._write_ascii_str(s)
                }
            }
        }
    }

    fn _write_ascii_str(&mut self, value: &str) -> Result<()> {
        let mut buf = value
            .chars()
            .map(|ch| if ch.is_ascii() { Some(ch as u8) } else { None })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error::Runtime("write_ascii_string: invalid ASCII char".to_string()))?;
        *buf.iter_mut().last().unwrap() |= 0x80;
        self.write_buf(&buf)
    }

    fn write_unicode_string(&mut self, value: &str) -> Result<()> {
        self.write_bytes(value.as_bytes())
    }

    fn write_unicode_string_nullable(&mut self, value: Option<&str>) -> Result<()> {
        match value {
            None => self.write_bytes_nullable(None),
            Some(s) => self.write_bytes_nullable(Some(s.as_bytes())),
        }
    }

    fn write_bytes(&mut self, value: &[u8]) -> Result<()> {
        self.write_uint(value.len() as u64)?;
        self.write_buf(value)
    }

    fn write_bytes_nullable(&mut self, value: Option<&[u8]>) -> Result<()> {
        match value {
            None => self.write_uint_nullable(None),
            Some(b) => {
                if b.is_empty() {
                    self.write_uint_nullable(Some(0))
                } else {
                    self.write_uint_nullable(Some(b.len() as u64))?;
                    self.write_buf(b)
                }
            }
        }
    }
}


impl Writer for bytes::BytesMut {
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.put_u8(value);
        Ok(())
    }

    fn write_buf(&mut self, buf: &[u8]) -> Result<()> {
        self.put(buf);
        Ok(())
    }
}


/// Wrapper around `std::io::Write` that implements [`fastlib::Writer`][crate::encoder::writer::Writer].
pub(crate) struct StreamWriter<'a> {
    stream: &'a mut dyn Write,
}

impl<'a> StreamWriter<'a> {
    pub fn new(stream: &'a mut dyn Write) -> Self {
        Self { stream }
    }
}

impl Writer for StreamWriter<'_> {
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.stream.write_all(&[value])?;
        Ok(())
    }

    fn write_buf(&mut self, buf: &[u8]) -> Result<()> {
        self.stream.write_all(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_presence_map() {
        struct TestCase {
            pmap: (u64, u8),
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                pmap: (0b0, 7),
                value: vec![0x80],
            },
            TestCase {
                pmap: (0b1, 7),
                value: vec![0x81],
            },
            TestCase {
                pmap: (0b11110001111, 14),
                value: vec![0x0f, 0x8f],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_presence_map(tc.pmap.0, tc.pmap.1).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_uint() {
        struct TestCase {
            input: u64,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: 0,
                value: vec![0x80],
            },
            TestCase {
                input: 1,
                value: vec![0x81],
            },
            TestCase {
                input: 942755,
                value: vec![0x39, 0x45, 0xa3],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_uint(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_uint_nullable() {
        struct TestCase {
            input: Option<u64>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: None,
                value: vec![0x80],
            },
            TestCase {
                input: Some(0),
                value: vec![0x81],
            },
            TestCase {
                input: Some(942755),
                value: vec![0x39, 0x45, 0xa4],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_uint_nullable(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_int() {
        struct TestCase {
            input: i64,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            // Mandatory Positive Number
            TestCase {
                input: 942755,
                value: vec![0x39, 0x45, 0xa3],
            },
            // Mandatory Negative Number
            TestCase {
                input: -7942755,
                value: vec![0x7c, 0x1b, 0x1b, 0x9d],
            },
            // Mandatory Positive Number with sign-bit extension
            TestCase {
                input: 8193,
                value: vec![0x00, 0x40, 0x81],
            },
            // Mandatory Negative Number with sign-bit extension
            TestCase {
                input: -8193,
                value: vec![0x7f, 0x3f, 0xff],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_int(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_int_nullable() {
        struct TestCase {
            input: Option<i64>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: None,
                value: vec![0x80],
            },
            // Optional Positive Number
            TestCase {
                input: Some(942755),
                value: vec![0x39, 0x45, 0xa4],
            },
            // Optional Negative Number
            TestCase {
                input: Some(-942755),
                value: vec![0x46, 0x3a, 0xdd],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_int_nullable(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_ascii_string() {
        struct TestCase {
            input: &'static str,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: "",
                value: vec![0x80],
            },
            TestCase {
                input: "ABC",
                value: vec![0x41, 0x42, 0xc3],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_ascii_string(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_ascii_string_nullable() {
        struct TestCase {
            input: Option<&'static str>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: None,
                value: vec![0x80],
            },
            TestCase {
                input: Some(""),
                value: vec![0x00, 0x80],
            },
            TestCase {
                input: Some("ABC"),
                value: vec![0x41, 0x42, 0xc3],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_ascii_string_nullable(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_unicode_string() {
        struct TestCase {
            input: &'static str,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: "",
                value: vec![0x80],
            },
            TestCase {
                input: "ABC",
                value: vec![0x83, 0x41, 0x42, 0x43],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_unicode_string(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_unicode_string_nullable() {
        struct TestCase {
            input: Option<&'static str>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: None,
                value: vec![0x80],
            },
            TestCase {
                input: Some(""),
                value: vec![0x81],
            },
            TestCase {
                input: Some("ABC"),
                value: vec![0x84, 0x41, 0x42, 0x43],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_unicode_string_nullable(tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_bytes() {
        struct TestCase {
            input: Vec<u8>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: vec![],
                value: vec![0x80],
            },
            TestCase {
                input: vec![0x41, 0x42, 0x43],
                value: vec![0x83, 0x41, 0x42, 0x43],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_bytes(&tc.input).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }

    #[test]
    fn write_bytes_nullable() {
        struct TestCase {
            input: Option<Vec<u8>>,
            value: Vec<u8>,
        }
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: None,
                value: vec![0x80],
            },
            TestCase {
                input: Some(vec![]),
                value: vec![0x81],
            },
            TestCase {
                input: Some(vec![0x41, 0x42, 0x43]),
                value: vec![0x84, 0x41, 0x42, 0x43],
            },
        ];
        for tc in test_cases {
            let mut buf = bytes::BytesMut::new();
            buf.write_bytes_nullable(tc.input.as_deref()).unwrap();
            assert_eq!(buf.to_vec(), tc.value);
        }
    }
}
