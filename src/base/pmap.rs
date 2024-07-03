#[derive(Debug, PartialEq)]
pub(crate) struct PresenceMap {
    pub(crate) bitmap: u64,
    pub(crate) mask: u64,
}

/// Represents the presence map field.
impl PresenceMap {
    pub(crate) fn new_empty() -> Self {
        Self {
            bitmap: 0,
            mask: 0x40, // 0100 0000
        }
    }

    pub(crate) fn new(bitmap: u64, size: u8) -> Self {
        Self {
            bitmap,
            mask: 1u64 << (size - 1),
        }
    }

    pub(crate) fn next_bit_set(&mut self) -> bool {
        let res = self.bitmap & self.mask != 0;
        self.mask >>= 1;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_map_next_bit_set() {
        let mut pmap = PresenceMap::new(0b1010110, 7);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), false);
    }
}
