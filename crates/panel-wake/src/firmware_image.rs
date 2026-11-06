//! Checks on a firmware image before it is written to flash.

use core::fmt;

/// The first byte of every ESP32 application image.
const IMAGE_MAGIC: u8 = 0xe9;
/// Bytes of the image header, extended header included.
const HEADER_BYTES: usize = 24;
/// Bytes of each segment's header: load address and length.
const SEGMENT_HEADER_BYTES: usize = 8;
/// The seed of the image's XOR checksum.
const CHECKSUM_SEED: u8 = 0xef;
/// Bytes of the SHA-256 digest some images carry after the checksum.
const DIGEST_BYTES: usize = 32;
/// The magic word that starts the application description in the first segment.
const APP_DESC_MAGIC: u32 = 0xabcd_5432;
/// Where the version and the project name sit in the application description.
const APP_DESC_VERSION: core::ops::Range<usize> = 16..48;
const APP_DESC_PROJECT: core::ops::Range<usize> = 48..80;

/// The chip an image is built for, from its extended header.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChipId(pub u16);

impl ChipId {
    /// The ESP32-S3.
    pub const ESP32_S3: Self = Self(9);
}

/// What a checked firmware image says about itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirmwareImage<'a> {
    /// The application's version, e.g. `0.2.0`.
    pub version: &'a str,
    /// The application's name, e.g. `ee02-thin`.
    pub project: &'a str,
    /// The image's length; anything after it is padding.
    pub len: usize,
}

impl<'a> FirmwareImage<'a> {
    /// Checks `bytes` as an application image for `chip` named `project`: the header, every
    /// segment, the XOR checksum and the application description. The bootloader checks the
    /// SHA-256 digest when it boots the image, and falls back to the running slot if that fails.
    ///
    /// # Errors
    ///
    /// Returns [`FirmwareImageError`] for anything the panel should not write to flash.
    pub fn check(bytes: &'a [u8], chip: ChipId, project: &str) -> Result<Self, FirmwareImageError> {
        let header: &[u8] = bytes
            .get(..HEADER_BYTES)
            .ok_or(FirmwareImageError::Truncated)?;
        if header[0] != IMAGE_MAGIC {
            return Err(FirmwareImageError::NotAnImage);
        }
        if u16::from_le_bytes([header[12], header[13]]) != chip.0 {
            return Err(FirmwareImageError::WrongChip);
        }
        let mut offset: usize = HEADER_BYTES;
        let mut checksum: u8 = CHECKSUM_SEED;
        let mut first: Option<&[u8]> = None;
        for _ in 0..header[1] {
            let segment: &[u8] = bytes
                .get(offset..offset + SEGMENT_HEADER_BYTES)
                .ok_or(FirmwareImageError::Truncated)?;
            let length: usize =
                u32::from_le_bytes([segment[4], segment[5], segment[6], segment[7]]) as usize;
            offset += SEGMENT_HEADER_BYTES;
            let data: &[u8] = offset
                .checked_add(length)
                .and_then(|end| bytes.get(offset..end))
                .ok_or(FirmwareImageError::Truncated)?;
            checksum = data.iter().fold(checksum, |sum, byte| sum ^ byte);
            first.get_or_insert(data);
            offset += length;
        }
        // The checksum is the last byte of a 16-byte block.
        let checksum_at: usize = offset | 0xf;
        let stored: u8 = *bytes
            .get(checksum_at)
            .ok_or(FirmwareImageError::Truncated)?;
        if stored != checksum {
            return Err(FirmwareImageError::Checksum);
        }
        let mut len: usize = checksum_at + 1;
        if header[23] == 1 {
            len += DIGEST_BYTES;
            if bytes.len() < len {
                return Err(FirmwareImageError::Truncated);
            }
        }
        let description: &[u8] = first
            .filter(|data| data.len() >= APP_DESC_PROJECT.end)
            .ok_or(FirmwareImageError::NoDescription)?;
        if u32::from_le_bytes([
            description[0],
            description[1],
            description[2],
            description[3],
        ]) != APP_DESC_MAGIC
        {
            return Err(FirmwareImageError::NoDescription);
        }
        let version: &str = text(&description[APP_DESC_VERSION])?;
        let found: &str = text(&description[APP_DESC_PROJECT])?;
        if found != project {
            return Err(FirmwareImageError::WrongProject);
        }
        Ok(Self {
            version,
            project: found,
            len,
        })
    }
}

/// Reads a NUL-padded string from the application description.
fn text(field: &[u8]) -> Result<&str, FirmwareImageError> {
    let end: usize = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(field.len());
    core::str::from_utf8(&field[..end]).map_err(|_| FirmwareImageError::NoDescription)
}

/// Why a download is not firmware the panel can boot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum FirmwareImageError {
    /// It does not start like an ESP32 application image.
    NotAnImage,
    /// It is built for another chip.
    WrongChip,
    /// It ends before its last segment, checksum or digest.
    Truncated,
    /// Its XOR checksum does not match: damaged on the way.
    Checksum,
    /// Its first segment has no application description.
    NoDescription,
    /// It is another application, e.g. the standalone firmware.
    WrongProject,
}

impl fmt::Display for FirmwareImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotAnImage => "the download is not an ESP32 firmware image",
            Self::WrongChip => "the firmware is built for another chip",
            Self::Truncated => "the firmware image is cut short",
            Self::Checksum => "the firmware image is damaged",
            Self::NoDescription => "the firmware image has no application description",
            Self::WrongProject => "the firmware is for another kind of frame",
        })
    }
}

impl core::error::Error for FirmwareImageError {}

#[cfg(test)]
mod test {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;

    /// An image with an application description and one more segment, as espflash lays it out.
    fn image(version: &str, project: &str, digest: bool) -> Vec<u8> {
        let mut description: Vec<u8> = vec![0; 256];
        description[..4].copy_from_slice(&APP_DESC_MAGIC.to_le_bytes());
        description[16..16 + version.len()].copy_from_slice(version.as_bytes());
        description[48..48 + project.len()].copy_from_slice(project.as_bytes());
        let code: Vec<u8> = (0..=200).collect();
        let mut bytes: Vec<u8> = vec![0; HEADER_BYTES];
        bytes[0] = IMAGE_MAGIC;
        bytes[1] = 2;
        bytes[12..14].copy_from_slice(&ChipId::ESP32_S3.0.to_le_bytes());
        bytes[23] = u8::from(digest);
        let mut checksum: u8 = CHECKSUM_SEED;
        for (address, data) in [(0x3c00_0020_u32, &description), (0x4037_0000, &code)] {
            bytes.extend_from_slice(&address.to_le_bytes());
            bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
            bytes.extend_from_slice(data);
            checksum = data.iter().fold(checksum, |sum, byte| sum ^ byte);
        }
        bytes.resize(bytes.len() | 0xf, 0);
        bytes.push(checksum);
        if digest {
            bytes.extend_from_slice(&[0x5a; DIGEST_BYTES]);
        }
        bytes
    }

    #[test]
    fn reads_a_good_image_and_its_padding() {
        let mut bytes: Vec<u8> = image("0.2.0", "ee02-thin", true);
        let len: usize = bytes.len();
        bytes.extend_from_slice(&[0xff; 100]);
        assert_eq!(
            FirmwareImage::check(&bytes, ChipId::ESP32_S3, "ee02-thin"),
            Ok(FirmwareImage {
                version: "0.2.0",
                project: "ee02-thin",
                len,
            })
        );
        let plain: Vec<u8> = image("1.0.0", "ee02-thin", false);
        assert_eq!(
            FirmwareImage::check(&plain, ChipId::ESP32_S3, "ee02-thin").map(|image| image.len),
            Ok(plain.len())
        );
    }

    #[test]
    fn refuses_damage_and_strangers() {
        let bytes: Vec<u8> = image("0.2.0", "ee02-thin", true);
        fn check(bytes: &[u8]) -> Result<FirmwareImage<'_>, FirmwareImageError> {
            FirmwareImage::check(bytes, ChipId::ESP32_S3, "ee02-thin")
        }
        let mut flipped: Vec<u8> = bytes.clone();
        flipped[200] ^= 1;
        assert_eq!(check(&flipped), Err(FirmwareImageError::Checksum));
        assert_eq!(
            check(&bytes[..bytes.len() - 1]),
            Err(FirmwareImageError::Truncated)
        );
        assert_eq!(check(&bytes[..300]), Err(FirmwareImageError::Truncated));
        assert_eq!(check(b"<html>"), Err(FirmwareImageError::Truncated));
        assert_eq!(check(&[0; 64]), Err(FirmwareImageError::NotAnImage));
        assert_eq!(
            FirmwareImage::check(&bytes, ChipId(5), "ee02-thin"),
            Err(FirmwareImageError::WrongChip)
        );
        assert_eq!(
            FirmwareImage::check(&bytes, ChipId::ESP32_S3, "ee02-standalone"),
            Err(FirmwareImageError::WrongProject)
        );
        let mut no_description: Vec<u8> = bytes;
        no_description[HEADER_BYTES + SEGMENT_HEADER_BYTES] = 0;
        assert!(check(&no_description).is_err());
    }
}
