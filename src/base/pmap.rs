#[derive(Debug, PartialEq)]
pub(crate) struct PresenceMap {
    pub(crate) bitmap: u64,
    pub(crate) mask: u64,
    pub(crate) size: u8,
}

/// Represents the presence map field.
impl PresenceMap {
    pub(crate) fn new_empty() -> Self {
        Self {
            bitmap: 0,
            mask: 0x40, // 0100 0000
            size: 7,
        }
    }

    pub(crate) fn new(bitmap: u64, size: u8) -> Self {
        Self {
            bitmap,
            mask: 1u64 << (size - 1),
            size,
        }
    }

    pub(crate) fn next_bit_set(&mut self) -> bool {
        let res = self.bitmap & self.mask != 0;
        self.mask >>= 1;
        res
    }

    pub(crate) fn set_next_bit(&mut self, value: bool) {
        if self.mask == 0 {
            self.bitmap <<= 7;
            self.mask = 0x40;
            self.size += 7;
        }
        if value {
            self.bitmap |= self.mask;
        }
        self.mask >>= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_map_next_bit_set() {
        let mut pmap = PresenceMap::new(0b1010110, 7);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), true);
        assert_eq!(pmap.next_bit_set(), false);
        // all other bits are false
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), false);
        assert_eq!(pmap.next_bit_set(), false);
    }

    #[test]
    fn presence_map_set_next_bit() {
        let mut pmap = PresenceMap::new_empty();
        pmap.set_next_bit(true);
        pmap.set_next_bit(false);
        pmap.set_next_bit(true);
        pmap.set_next_bit(false);
        pmap.set_next_bit(true);
        pmap.set_next_bit(true);
        pmap.set_next_bit(false);
        assert_eq!(pmap.bitmap, 0b1010110);
        assert_eq!(pmap.size, 7);
        // next bits extend the bitmap by 7 bits
        pmap.set_next_bit(true);
        pmap.set_next_bit(false);
        pmap.set_next_bit(true);
        assert_eq!(pmap.bitmap, 0b10101101010000);
        assert_eq!(pmap.size, 14);
    }
}
