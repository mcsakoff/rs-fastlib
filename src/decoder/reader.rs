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
///
/// Only [`read_u8`][Reader::read_u8] is required. Reader backed by contiguous memory should also override
/// the bulk methods ([`read_stop_bit_run`][Reader::read_stop_bit_run], [`read_exact_into`][Reader::read_exact_into])
/// and integer methods for better performance, as [`SliceReader`] does.
pub trait Reader {
    /// Reads a single byte.
    /// # Errors
    /// Do not return [`Error::Eof`][crate::Error::Eof] from this method.
    /// Return [`Error::UnexpectedEof`][crate::Error::UnexpectedEof] instead.
    fn read_u8(&mut self) -> Result<u8>;

    /// Append bytes to `out` up to and including the first byte with the stop bit set.
    /// Bytes are appended as is, i.e. the stop bit is not cleared.
    /// # Errors
    /// Returns error if the stream ends before the stop bit.
    fn read_stop_bit_run(&mut self, out: &mut Vec<u8>) -> Result<()> {
        loop {
            let byte = self.read_u8()?;
            out.push(byte);
            if byte & 0x80 == 0x80 {
                return Ok(());
            }
        }
    }

    /// Appends exactly `n` bytes to `out`.
    /// # Errors
    /// Returns error if the stream ends before `n` bytes are read.
    fn read_exact_into(&mut self, n: usize, out: &mut Vec<u8>) -> Result<()> {
        // Do not trust `n` for pre-allocation: it comes from the stream.
        out.reserve(n.min(4096));
        for _ in 0..n {
            out.push(self.read_u8()?);
        }
        Ok(())
    }

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
                return Err(pmap_overflow());
            }
            byte = self.read_u8()?;
        }
    }

    /// Returns decoded non-nullable 64 bits unsigned integer.
    /// # Errors
    /// Returns error if it can not read bytes or the value does not fit into 64 bit.
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
    /// Returns error if it can not read bytes or the value does not fit into 64 bit.
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
    /// Returns error if it can not read bytes or the value does not fit into 64 bit.
    fn read_int(&mut self) -> Result<i64> {
        let mut byte = self.read_u8()?;
        // Negative integers are sign-extended.
        let mut value: i64 = if byte & 0x40 != 0 { -1 } else { 0 };
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
    /// Returns error if it can not read bytes or the value does not fit into 64 bit.
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
        let mut buf: Vec<u8> = Vec::with_capacity(128); // just a guess
        self.read_stop_bit_run(&mut buf)?;
        Ok(ascii_from_run(buf))
    }

    /// Returns decoded nullable ASCII encoded string.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_ascii_string_nullable(&mut self) -> Result<Option<String>> {
        let mut buf: Vec<u8> = Vec::with_capacity(128); // just a guess
        self.read_stop_bit_run(&mut buf)?;
        match buf.as_slice() {
            [0x80] => Ok(None),
            [0x00, 0x80] => Ok(Some(String::new())),
            [0x80, ..] => {
                buf.remove(0);
                Ok(Some(ascii_from_run(buf)))
            }
            _ => Ok(Some(ascii_from_run(buf))),
        }
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
        let length = usize::try_from(self.read_uint()?).map_err(|_| integer_overflow())?;
        let mut buf = Vec::new();
        self.read_exact_into(length, &mut buf)?;
        Ok(buf)
    }

    /// Returns decoded nullable bytes.
    /// # Errors
    /// Returns error if it can not read bytes len or bytes len mismatch.
    fn read_bytes_nullable(&mut self) -> Result<Option<Vec<u8>>> {
        match self.read_uint_nullable()? {
            None => Ok(None),
            Some(length) => {
                let length = usize::try_from(length).map_err(|_| integer_overflow())?;
                let mut buf = Vec::new();
                self.read_exact_into(length, &mut buf)?;
                Ok(Some(buf))
            }
        }
    }
}

#[cold]
fn pmap_overflow() -> Error {
    Error::Dynamic(format!("presence map is longer that {MAX_PMAP_BITS} bits"))
}

#[cold]
fn integer_overflow() -> Error {
    Error::Dynamic("integer value does not fit into 64 bits".to_string())
}

// Converts a stop bit encoded run into an ASCII string.
// A single 0x80 byte is an empty string.
fn ascii_from_run(mut buf: Vec<u8>) -> String {
    if buf == [0x80] {
        return String::new();
    }
    if let Some(last) = buf.last_mut() {
        *last &= 0x7f;
    }
    // SAFETY: `buf` contains ASCII 7-bit characters.
    //         A stop bit run has the 8th bit set only in the last byte, which was cleared above.
    unsafe { String::from_utf8_unchecked(buf) }
}

// Helpers that parse values from contiguous memory. Each returns the value and the number of consumed bytes.
mod slice {
    use super::{MAX_PMAP_BITS, integer_overflow, pmap_overflow};
    use crate::{Error, Result};

    #[inline]
    pub(super) fn stop_bit_len(buf: &[u8]) -> Result<usize> {
        match buf.iter().position(|b| b & 0x80 != 0) {
            Some(i) => Ok(i + 1),
            None => Err(Error::UnexpectedEof),
        }
    }

    #[inline]
    pub(super) fn uint(buf: &[u8]) -> Result<(u64, usize)> {
        let mut value: u64 = 0;
        for (i, &byte) in buf.iter().enumerate() {
            if value > u64::MAX >> 7 {
                return Err(integer_overflow()); // [ERR D2]
            }
            value = (value << 7) | u64::from(byte & 0x7f);
            if byte & 0x80 != 0 {
                return Ok((value, i + 1));
            }
        }
        Err(Error::UnexpectedEof)
    }

    #[inline]
    pub(super) fn int(buf: &[u8]) -> Result<(i64, usize)> {
        let Some(&first) = buf.first() else {
            return Err(Error::UnexpectedEof);
        };
        // Negative integers are sign-extended.
        let mut value: i64 = if first & 0x40 != 0 { -1 } else { 0 };
        for (i, &byte) in buf.iter().enumerate() {
            if !(i64::MIN >> 7..=i64::MAX >> 7).contains(&value) {
                return Err(integer_overflow()); // [ERR D2]
            }
            value = (value << 7) | i64::from(byte & 0x7f);
            if byte & 0x80 != 0 {
                return Ok((value, i + 1));
            }
        }
        Err(Error::UnexpectedEof)
    }

    #[inline]
    pub(super) fn presence_map(buf: &[u8]) -> Result<((u64, u8), usize)> {
        if buf.is_empty() {
            return Err(Error::Eof);
        }
        let mut bitmap: u64 = 0;
        for (i, &byte) in buf.iter().enumerate() {
            if i * 7 >= MAX_PMAP_BITS {
                return Err(pmap_overflow());
            }
            bitmap = (bitmap << 7) | u64::from(byte & 0x7f);
            if byte & 0x80 != 0 {
                return Ok(((bitmap, (i as u8 + 1) * 7), i + 1));
            }
        }
        Err(Error::UnexpectedEof)
    }
}

// Readers backed by contiguous memory.
trait Contiguous {
    // Returns remaining bytes.
    fn rest(&self) -> &[u8];
    // Consume `n` bytes.
    fn consume(&mut self, n: usize);
}

// Implements the fast `Reader` methods for a type implementing `Contiguous`.
macro_rules! impl_contiguous_reader {
    () => {
        #[inline]
        fn read_u8(&mut self) -> Result<u8> {
            let Some(&b) = Contiguous::rest(self).first() else {
                return Err(Error::UnexpectedEof);
            };
            Contiguous::consume(self, 1);
            Ok(b)
        }

        #[inline]
        fn read_stop_bit_run(&mut self, out: &mut Vec<u8>) -> Result<()> {
            let rest = Contiguous::rest(self);
            let n = slice::stop_bit_len(rest)?;
            out.extend_from_slice(&rest[..n]);
            Contiguous::consume(self, n);
            Ok(())
        }

        #[inline]
        fn read_exact_into(&mut self, n: usize, out: &mut Vec<u8>) -> Result<()> {
            let rest = Contiguous::rest(self);
            if rest.len() < n {
                return Err(Error::UnexpectedEof);
            }
            out.extend_from_slice(&rest[..n]);
            Contiguous::consume(self, n);
            Ok(())
        }

        #[inline]
        fn read_presence_map(&mut self) -> Result<(u64, u8)> {
            let (v, n) = slice::presence_map(Contiguous::rest(self))?;
            Contiguous::consume(self, n);
            Ok(v)
        }

        #[inline]
        fn read_uint(&mut self) -> Result<u64> {
            let (v, n) = slice::uint(Contiguous::rest(self))?;
            Contiguous::consume(self, n);
            Ok(v)
        }

        #[inline]
        fn read_int(&mut self) -> Result<i64> {
            let (v, n) = slice::int(Contiguous::rest(self))?;
            Contiguous::consume(self, n);
            Ok(v)
        }
    };
}

impl Contiguous for bytes::Bytes {
    fn rest(&self) -> &[u8] {
        self.as_ref()
    }

    fn consume(&mut self, n: usize) {
        self.advance(n);
    }
}

impl Reader for bytes::Bytes {
    impl_contiguous_reader!();
}

/// Reader over a byte slice.
///
/// It is the fastest [`Reader`] implementation.
pub struct SliceReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the number of byres consumed so far.
    #[must_use]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the bytes not consumed yet.
    #[must_use]
    pub fn remaining(&self) -> &'a [u8] {
        &self.buf[self.pos..]
    }
}

impl Contiguous for SliceReader<'_> {
    #[inline]
    fn rest(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    #[inline]
    fn consume(&mut self, n: usize) {
        self.pos += n;
    }
}

impl Reader for SliceReader<'_> {
    impl_contiguous_reader!();
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

    // Fast paths of contiguous reader must behave exactly like the default byte-by-byte implementation.
    #[test]
    fn contiguous_reader_match_default() {
        use std::io::Cursor;

        // Stops at the first error: decoding is aborted after an error anyway.
        fn run<R: Reader>(r: &mut R) -> Vec<String> {
            let mut out = Vec::new();
            macro_rules! step {
                ($e:expr) => {
                    let v = $e;
                    out.push(format!("{v:?}"));
                    if v.is_err() {
                        return out;
                    }
                };
            }
            step!(r.read_presence_map());
            step!(r.read_uint());
            step!(r.read_int());
            step!(r.read_ascii_string_nullable());
            step!(r.read_ascii_string());
            step!(r.read_bytes_nullable());
            step!(r.read_unicode_string());
            step!(r.read_int_nullable());
            out
        }
        let inputs: Vec<Vec<u8>> = vec![
            vec![],
            vec![0x80],
            vec![0x0f, 0x8f, 0x81, 0xff, 0x80, 0x80],
            vec![
                0x80, 0x39, 0x45, 0xa3, 0x7c, 0x1b, 0xdd, 0x00, 0x80, 0x41, 0xc2,
            ],
            vec![
                0x91, 0x81, 0x81, 0x00, 0x41, 0xc2, 0x80, 0x84, 0x41, 0x42, 0x43, 0x82, 0xc3, 0xa9,
                0x80,
            ],
            vec![0x81, 0x81, 0x81, 0x80, 0x85, 0x41],
            vec![0x7f; 12],
            vec![0x81, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0x80],
            vec![0x81, 0x80, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x80],
            vec![0x81, 0x80, 0x80, 0x80, 0x81, 0xff],
        ];
        for input in inputs {
            let mut cursor = Cursor::new(input.clone());
            let expected = run(&mut StreamReader::new(&mut cursor));
            assert_eq!(run(&mut SliceReader::new(&input)), expected, "{input:x?}");
            assert_eq!(run(&mut bytes::Bytes::from(input.clone())), expected, "{input:x?}");
        }
    }
}
