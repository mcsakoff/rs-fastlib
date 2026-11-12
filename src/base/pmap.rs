use crate::common::definitions::MAX_PMAP_BITS;

// Number of 7-bit groups that fit into the bitmap.
const WINDOW_GROUPS: usize = MAX_PMAP_BITS / 7;

/// Represents the presence map field.
///
/// Bits are kept in `bitmap` (up to 63 bits). A longer presence map keeps the other 7-bit groups in `groups`:
/// the decoder keeps the groups following the bitmap that are not loaded yet; the encoder keeps the groups
/// preceding the bitmap.
#[derive(Debug, PartialEq)]
pub(crate) struct PresenceMap {
    pub(crate) bitmap: u64,
    pub(crate) mask: u64,
    pub(crate) size: u8,
    // Only long presence maps allocate.
    long: Option<Box<Groups>>,
}

#[derive(Debug, PartialEq, Default)]
struct Groups {
    groups: Vec<u8>,
    // Decoder: index of the first group not loaded into the bitmap.
    pos: usize,
}

impl PresenceMap {
    pub(crate) fn new_empty() -> Self {
        Self::new(0, 7)
    }

    pub(crate) fn new(bitmap: u64, size: u8) -> Self {
        Self {
            bitmap,
            mask: 1u64 << (size - 1),
            size,
            long: None,
        }
    }

    /// Creates a presence map from 7-bit groups (the stop bit cleared).
    pub(crate) fn from_groups(groups: Vec<u8>) -> Self {
        let mut pmap = Self::new(0, 7);
        pmap.long = Some(Box::new(Groups { groups, pos: 0 }));
        pmap.refill();
        pmap
    }

    // Loads the next groups into the bitmap. Returns `false` if there are no more groups.
    #[cold]
    fn refill(&mut self) -> bool {
        let Some(long) = self.long.as_mut() else {
            return false;
        };
        let rest = &long.groups[long.pos..];
        if rest.is_empty() {
            return false;
        }
        let n = rest.len().min(WINDOW_GROUPS);
        self.bitmap = rest[..n]
            .iter()
            .fold(0, |bitmap, g| (bitmap << 7) | u64::from(*g));
        self.size = (n * 7) as u8;
        self.mask = 1u64 << (self.size - 1);
        long.pos += n;
        true
    }

    #[inline]
    pub(crate) fn next_bit_set(&mut self) -> bool {
        // Logically a presence map has an infinite suffix of zeroes.
        if self.mask == 0 && !self.refill() {
            return false;
        }
        let res = self.bitmap & self.mask != 0;
        self.mask >>= 1;
        res
    }

    /// Returns `true` if any of the bits that were not read yet is set.
    pub(crate) fn has_unread_bits_set(&self) -> bool {
        let window = if self.mask == 0 {
            0
        } else {
            self.bitmap & ((self.mask << 1) - 1)
        };
        window != 0
            || self
                .long
                .as_ref()
                .is_some_and(|l| l.groups[l.pos..].iter().any(|g| *g != 0))
    }

    #[inline]
    pub(crate) fn set_next_bit(&mut self, value: bool) {
        if self.mask == 0 {
            self.grow();
        }
        if value {
            self.bitmap |= self.mask;
        }
        self.mask >>= 1;
    }

    // Adds a 7-bit group to the bitmap; the full bitmap is moved to `groups`.
    #[cold]
    fn grow(&mut self) {
        if usize::from(self.size) >= MAX_PMAP_BITS {
            let long = self.long.get_or_insert_default();
            for i in (0..usize::from(self.size) / 7).rev() {
                long.groups.push((self.bitmap >> (i * 7)) as u8 & 0x7f);
            }
            self.bitmap = 0;
            self.size = 0;
        }
        self.bitmap <<= 7;
        self.mask = 0x40;
        self.size += 7;
    }

    /// Returns `true` if the presence map is longer than the bitmap.
    pub(crate) fn is_long(&self) -> bool {
        self.long.is_some()
    }

    /// Returns the stop bit encoded presence map with trailing zero groups truncated.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self
            .long
            .as_ref()
            .map_or_else(Vec::new, |l| l.groups.clone());
        for i in (0..usize::from(self.size) / 7).rev() {
            bytes.push((self.bitmap >> (i * 7)) as u8 & 0x7f);
        }
        while bytes.len() > 1 && bytes.last() == Some(&0) {
            bytes.pop();
        }
        if let Some(last) = bytes.last_mut() {
            *last |= 0x80;
        }
        bytes
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
