//! The 20-byte header in front of every `/frame.e6` page.

use core::fmt;

use crate::crc32::Crc32;

/// How the payload after the header is encoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Encoding {
    /// Controller colour codes, two pixels a byte with the left one in the high nibble, every
    /// row's left half (first controller) followed by every row's right half (second).
    Nibbles,
}

impl Encoding {
    fn to_byte(self) -> u8 {
        match self {
            Self::Nibbles => 1,
        }
    }

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::Nibbles),
            _ => None,
        }
    }
}

/// Why a page could not be encoded or read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum E6Error {
    /// Fewer than [`E6Header::LEN`] bytes were given.
    TooShort,
    /// The magic is not `E6F1`.
    BadMagic,
    /// The version is not one this crate reads.
    UnsupportedVersion(u16),
    /// The encoding byte is not one this crate reads.
    UnsupportedEncoding(u8),
    /// The payload's length or checksum does not match the header.
    PayloadMismatch,
    /// The page does not fit the only panel the format packs for, the 13.3".
    WrongResolution,
}

impl fmt::Display for E6Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => formatter.write_str("header is shorter than 20 bytes"),
            Self::BadMagic => formatter.write_str("magic is not E6F1"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported version {version}"),
            Self::UnsupportedEncoding(encoding) => {
                write!(formatter, "unsupported encoding {encoding}")
            }
            Self::PayloadMismatch => formatter.write_str("payload does not match the header"),
            Self::WrongResolution => {
                formatter.write_str("page is not laid out for the 13.3\" panel")
            }
        }
    }
}

impl core::error::Error for E6Error {}

/// The header of an E6F1 page: little-endian, 20 bytes.
///
/// | Offset | Field | Type |
/// | --- | --- | --- |
/// | 0 | magic `E6F1` | 4 bytes |
/// | 4 | version | u16 |
/// | 6 | encoding | u8 |
/// | 7 | poll minutes | u8 |
/// | 8 | width | u16 |
/// | 10 | height | u16 |
/// | 12 | payload length | u32 |
/// | 16 | payload CRC-32 | u32 |
///
/// The layout is fugleramme's, so either side can talk to the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct E6Header {
    /// Payload encoding.
    pub encoding: Encoding,
    /// Minutes the client should sleep before asking again; 0 means as soon as it likes.
    pub poll_minutes: u8,
    /// Width of the payload's glass, in pixels.
    pub width: u16,
    /// Height of the payload's glass, in pixels.
    pub height: u16,
    /// Payload length in bytes.
    pub payload_len: u32,
    /// CRC-32 of the payload.
    pub payload_crc: u32,
}

impl E6Header {
    /// The header's size in bytes.
    pub const LEN: usize = 20;

    /// The magic every page starts with.
    pub const MAGIC: [u8; 4] = *b"E6F1";

    /// The format version this crate writes and reads.
    pub const VERSION: u16 = 1;

    /// Returns the header as it goes on the wire.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..4].copy_from_slice(&Self::MAGIC);
        bytes[4..6].copy_from_slice(&Self::VERSION.to_le_bytes());
        bytes[6] = self.encoding.to_byte();
        bytes[7] = self.poll_minutes;
        bytes[8..10].copy_from_slice(&self.width.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.height.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.payload_len.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.payload_crc.to_le_bytes());
        bytes
    }

    /// Reads a header from the first [`E6Header::LEN`] bytes of `bytes`.
    ///
    /// # Errors
    ///
    /// Returns an error when `bytes` is too short or the magic, version or encoding is not one
    /// this crate understands.
    pub fn parse(bytes: &[u8]) -> Result<Self, E6Error> {
        let Some(bytes) = bytes.get(..Self::LEN) else {
            return Err(E6Error::TooShort);
        };
        if bytes[0..4] != Self::MAGIC {
            return Err(E6Error::BadMagic);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != Self::VERSION {
            return Err(E6Error::UnsupportedVersion(version));
        }
        let encoding =
            Encoding::from_byte(bytes[6]).ok_or(E6Error::UnsupportedEncoding(bytes[6]))?;
        Ok(Self {
            encoding,
            poll_minutes: bytes[7],
            width: u16::from_le_bytes([bytes[8], bytes[9]]),
            height: u16::from_le_bytes([bytes[10], bytes[11]]),
            payload_len: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            payload_crc: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        })
    }

    /// Checks that `payload` has the length and CRC-32 this header declares.
    ///
    /// # Errors
    ///
    /// Returns [`E6Error::PayloadMismatch`] when either differs.
    pub fn verify_payload(&self, payload: &[u8]) -> Result<(), E6Error> {
        let length_matches = u32::try_from(payload.len()).is_ok_and(|len| len == self.payload_len);
        if length_matches && Crc32::checksum(payload) == self.payload_crc {
            Ok(())
        } else {
            Err(E6Error::PayloadMismatch)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample() -> E6Header {
        E6Header {
            encoding: Encoding::Nibbles,
            poll_minutes: 15,
            width: 1200,
            height: 1600,
            payload_len: 960_000,
            payload_crc: 0x6466_5809,
        }
    }

    #[test]
    fn round_trips() {
        let header = sample();
        assert_eq!(E6Header::parse(&header.to_bytes()), Ok(header));
    }

    #[test]
    fn rejects_short_input() {
        assert_eq!(E6Header::parse(&[0u8; 19]), Err(E6Error::TooShort));
    }

    #[test]
    fn rejects_unknown_encoding() {
        let mut bytes = sample().to_bytes();
        bytes[6] = 9;
        assert_eq!(
            E6Header::parse(&bytes),
            Err(E6Error::UnsupportedEncoding(9))
        );
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = sample().to_bytes();
        bytes[0] = b'X';
        assert_eq!(E6Header::parse(&bytes), Err(E6Error::BadMagic));
    }
}
