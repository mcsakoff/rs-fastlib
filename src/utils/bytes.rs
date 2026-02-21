use std::{cmp::Ordering, fmt::Write};

use crate::{Error, Result};

pub(crate) fn bytes_to_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

pub(crate) fn string_to_bytes(s: &str) -> Result<Vec<u8>> {
    let s = s.trim().replace(' ', "");
    if !s.len().is_multiple_of(2) {
        return Err(Error::Runtime(format!(
            "Invalid hex string (length): '{s}'"
        )));
    }
    let v = s
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|ch| {
            let hi = hexchar2byte(ch[0])?;
            let lo = hexchar2byte(ch[1])?;
            Ok((hi << 4) | lo)
        })
        .collect::<Result<Vec<u8>>>()?;
    Ok(v)
}

fn hexchar2byte(c: char) -> Result<u8> {
    if c.is_ascii_digit() {
        Ok((c as u8) - b'0')
    } else if ('a'..='f').contains(&c) {
        Ok((c as u8) - b'a' + 10)
    } else if ('A'..='F').contains(&c) {
        Ok((c as u8) - b'A' + 10)
    } else {
        Err(Error::Runtime(format!("Invalid hex char: '{c}'")))
    }
}

pub fn string_delta<'a>(a: &'a str, b: &'a str) -> (&'a str, i32) {
    let common_front = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
    let common_back = a
        .bytes()
        .rev()
        .zip(b.bytes().rev())
        .take_while(|(x, y)| x == y)
        .count();
    match common_front.cmp(&common_back) {
        Ordering::Equal | Ordering::Greater => {
            let sub = a.len() - common_front;
            let delta = &b[common_front..];
            (delta, sub as i32)
        }
        Ordering::Less => {
            let sub = a.len() - common_back;
            let idx = b.len() - common_back;
            let delta = &b[..idx];
            (delta, -((sub + 1) as i32))
        }
    }
}

pub fn bytes_delta<'a>(a: &'a [u8], b: &'a [u8]) -> (&'a [u8], i32) {
    let common_front = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();
    let common_back = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    match common_front.cmp(&common_back) {
        Ordering::Equal | Ordering::Greater => {
            let sub = a.len() - common_front;
            let delta = &b[common_front..];
            (delta, sub as i32)
        }
        Ordering::Less => {
            let sub = a.len() - common_back;
            let idx = b.len() - common_back;
            let delta = &b[..idx];
            (delta, -((sub + 1) as i32))
        }
    }
}

pub fn string_tail<'a>(a: &'a str, b: &'a str) -> Result<&'a str> {
    match b.len().cmp(&a.len()) {
        Ordering::Equal => {
            let common_front = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
            Ok(&b[common_front..])
        }
        Ordering::Greater => Ok(b),
        Ordering::Less => Err(Error::Dynamic(
            "tail: next value length is less than the previous one".to_string(),
        )),
    }
}

pub fn bytes_tail<'a>(a: &'a [u8], b: &'a [u8]) -> Result<&'a [u8]> {
    match b.len().cmp(&a.len()) {
        Ordering::Equal => {
            let common_front = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();
            Ok(&b[common_front..])
        }
        Ordering::Greater => Ok(b),
        Ordering::Less => Err(Error::Dynamic(
            "tail: next value length is less than the previous one".to_string(),
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_to_bytes() {
        let s = "123456789abcdef0";
        let v = string_to_bytes(s).unwrap();
        assert_eq!(v, vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
    }

    #[test]
    fn test_bytes_to_string() {
        let v = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let s = bytes_to_string(&v);
        assert_eq!(&s, "123456789abcdef0");
    }

    #[test]
    fn test_string_delta() {
        assert_eq!(string_delta("", "GEH6"), ("GEH6", 0));
        assert_eq!(string_delta("GEH6", "GEM6"), ("M6", 2));
        assert_eq!(string_delta("GEM6", "ESM6"), ("ES", -3)); // -2 - 1
        assert_eq!(string_delta("ESM6", "RSESM6"), ("RS", -1)); // -0 - 1
        assert_eq!(string_delta("GEH6", "GE"), ("", 2));
        assert_eq!(string_delta("GEH6", "H6"), ("", -3));
        assert_eq!(string_delta("GEH6", "GEH6"), ("", 0));
    }

    #[test]
    fn test_string_tail() {
        assert_eq!(string_tail("", "GEH6").unwrap(), "GEH6");
        assert_eq!(string_tail("GEH6", "GEM6").unwrap(), "M6");
        assert_eq!(string_tail("ABC", "ABCD").unwrap(), "ABCD");
    }
}
