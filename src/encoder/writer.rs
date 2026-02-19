use std::{
    io::Write,
    ops::{BitAnd, Shr},
};

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
            return Err(Error::Runtime(
                "write_presence_map: size must be multiple of 7".to_string(),
            ));
        }

        let trailing_bits = bitmap.trailing_zeros() as usize;
        // Only if all 7 bits are 0 we treat byte as trailing
        let trailing_bytes = trailing_bits / 7;
        let bitmap = bitmap >> (trailing_bytes * 7);

        // Skipping useless bytes.
        let len = usize::from(size / 7).saturating_sub(trailing_bytes);
        if len == 0 {
            self.write_u8(0x80)?;
            return Ok(());
        }
        // For u64 there's only 10 bytes we can write (if number is u64::MAX) and meaning_bits is 64.
        // Then 64.div_ceil(7) is 10;
        let buf = encode_number(bitmap, len);
        self.write_buf(&buf[..len])
    }

    fn write_uint(&mut self, value: u64) -> Result<()> {
        // If number is zero we only have to add last byte marker.
        if value == 0 {
            return self.write_u8(0x80);
        }

        // Calculating position of the last 1 in number.
        let meaning_bits = u64::BITS - value.leading_zeros();

        // Since we can write only 7 bits (8 bit is the last bit marker), getting number of bytes to write.
        // 1..=7 => 1 byte
        // 8..=15 => 2 byte
        // etc
        let bytes_to_write = meaning_bits.div_ceil(7) as usize;
        let buf = encode_number(value, bytes_to_write);
        self.write_buf(&buf[..bytes_to_write])
    }

    fn write_uint_nullable(&mut self, value: Option<u64>) -> Result<()> {
        match value {
            None => self.write_uint(0),
            Some(v) => self.write_uint(v + 1),
        }
    }

    fn write_int(&mut self, value: i64) -> Result<()> {
        // If number contains only meaningless_bits, just write it with last byte marker.
        if value == 0 || value == -1 {
            return self.write_u8(value as u8 | 0x80);
        }

        let is_pos = value >= 0;

        // For posititive numbers we ignore 0 in the MSB since we only search for last 1
        // For negatitive number logic is reversed.
        let useless_bits = if is_pos {
            value.leading_zeros()
        } else {
            value.leading_ones()
        };

        // Calculating position of the last meaning sign in number.
        let meaning_bits = i64::BITS - useless_bits;

        // Since we can write only 7 bits (8 bit is the last bit marker), getting number of bytes to write.
        // 1..=7 => 1 byte
        // 8..=15 => 2 byte
        // etc
        let bytes_to_write = meaning_bits.div_ceil(7) as usize;

        // Additional byte is required if the last signed bit position is divisible by 7.
        let additional_byte = usize::from(
            (value.unbounded_shr(bytes_to_write as u32 * 7 - 1)) & 0x1 == i64::from(is_pos),
        );
        let bytes_to_write = bytes_to_write + additional_byte;
        let buf = encode_number(value, bytes_to_write);
        self.write_buf(&buf[..bytes_to_write])
    }

    fn write_int_nullable(&mut self, value: Option<i64>) -> Result<()> {
        match value {
            None => self.write_int(0),
            Some(v) if v >= 0 => self.write_int(v + 1),
            Some(v) => self.write_int(v),
        }
    }

    fn write_ascii_string(&mut self, value: &str) -> Result<()> {
        self.write_ascii_str(value, &[0x80])
    }

    fn write_ascii_string_nullable(&mut self, value: Option<&str>) -> Result<()> {
        match value {
            None => self.write_u8(0x80),
            Some(s) => self.write_ascii_str(s, &[0x00, 0x80]),
        }
    }

    fn write_ascii_str(&mut self, value: &str, empty: &[u8]) -> Result<()> {
        // Checking is string contains only ASCII chars.
        // If so we can just use them as slice of bytes with only last byte changed.
        if !value.is_ascii() {
            return Err(Error::Runtime(
                "write_ascii_string: invalid ASCII char".to_string(),
            ));
        }

        // Splitting up last bytes since it have to be marked.
        // If string is empty we should write only empty.
        let [buf @ .., last_byte] = value.as_bytes() else {
            return self.write_buf(empty);
        };

        self.write_buf(buf)?;
        self.write_u8(*last_byte | 0x80)
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

trait ToByte {
    fn to_byte(&self) -> u8;
}

impl ToByte for i64 {
    fn to_byte(&self) -> u8 {
        *self as u8
    }
}

impl ToByte for u64 {
    fn to_byte(&self) -> u8 {
        *self as u8
    }
}

fn encode_number<T>(number: T, len: usize) -> [u8; 10]
where
    T: ToByte + Shr<usize, Output = T> + BitAnd<Output = T> + From<u8> + Copy,
{
    // For 64 bit number there's only 10 bytes we can write (if number is T::MAX) and meaning_bits is 64.
    // Then 64.div_ceil(7) is 10;
    let mut buf = [0; 10];
    let values = (0..len).map(|i| {
        // Writing in reversed order because most signed bits have to be written first.
        let offset_bits_index = len - i - 1;
        let shifted_bitmap = number >> (offset_bits_index * 7);
        (shifted_bitmap & T::from(0x7f)).to_byte()
    });
    for (buf, value) in buf.iter_mut().zip(values) {
        *buf = value;
    }

    // set stop bit
    buf[len - 1] |= 0x80;
    buf
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
            assert_eq!(buf.to_vec(), tc.value, "Invalid encoder for {}", tc.input);
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
