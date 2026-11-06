//! CRC-32 as zlib computes it (IEEE 802.3, reflected, polynomial `0xedb88320`).

const POLYNOMIAL: u32 = 0xedb8_8320;

const TABLE: [u32; 256] = build_table();

const fn build_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut value = index as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 == 1 {
                (value >> 1) ^ POLYNOMIAL
            } else {
                value >> 1
            };
            bit += 1;
        }
        table[index] = value;
        index += 1;
    }
    table
}

/// An incremental CRC-32, so a page can be checked while it streams in.
///
/// ```
/// use frame_core::Crc32;
///
/// let mut crc = Crc32::new();
/// crc.update(b"1234");
/// crc.update(b"56789");
/// assert_eq!(crc.finish(), 0xcbf4_3926);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crc32 {
    state: u32,
}

impl Crc32 {
    /// Starts a new checksum.
    #[must_use]
    pub fn new() -> Self {
        Self { state: u32::MAX }
    }

    /// Returns the checksum of `bytes` in one call.
    #[must_use]
    pub fn checksum(bytes: &[u8]) -> u32 {
        let mut crc = Self::new();
        crc.update(bytes);
        crc.finish()
    }

    /// Feeds more bytes into the checksum.
    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            let index = usize::from((self.state as u8) ^ byte);
            self.state = (self.state >> 8) ^ TABLE[index];
        }
    }

    /// Returns the checksum of everything fed so far.
    #[must_use]
    pub fn finish(&self) -> u32 {
        !self.state
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_input_is_zero() {
        assert_eq!(Crc32::checksum(b""), 0);
    }

    #[test]
    fn matches_the_standard_check_value() {
        assert_eq!(Crc32::checksum(b"123456789"), 0xcbf4_3926);
    }
}
