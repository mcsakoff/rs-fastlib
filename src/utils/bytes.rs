use crate::{Error, Result};

pub(crate) fn bytes_to_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    return s;
}

pub(crate) fn string_to_bytes(s: &str) -> Result<Vec<u8>> {
    let s = s.trim().replace(" ", "");
    if s.len() % 2 != 0 {
        return Err(Error::Runtime(format!("Invalid hex string (length): '{}'", s)));
    }
    let v = s.chars()
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
    if c >= '0' && c <= '9' {
        Ok((c as u8) - ('0' as u8))
    } else if c >= 'a' && c <= 'f' {
        Ok((c as u8) - ('a' as u8) + 10)
    } else if c >= 'A' && c <= 'F' {
        Ok((c as u8) - ('A' as u8) + 10)
    } else {
        Err(Error::Runtime(format!("Invalid hex char: '{c}'")))
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
}
