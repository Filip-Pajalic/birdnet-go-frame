//! Many `.artp` pictures in one file, looked up by species: art for a device with no file
//! system, built into its flash.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::art_error::ArtError;
use crate::art_image::ArtImage;
use crate::art_source::ArtSource;

const MAGIC: [u8; 4] = *b"ARTB";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 8;
/// The largest picture a bundle may unpack to: 4096 by 4096 pixels of single runs.
const MAX_UNPACKED: usize = 4096 * 4096 * 6;

/// One picture going into a bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BundleEntry<'a> {
    /// The species' scientific name, e.g. `Parus major`.
    pub name: &'a str,
    /// The picture's `.artp` bytes ([`ArtImage::encode`]) as a zlib stream.
    pub zlib: &'a [u8],
    /// How many bytes the `.artp` picture has unpacked.
    pub unpacked_len: usize,
}

/// A bundle of zlib-compressed `.artp` pictures, one per species, read in place from borrowed
/// bytes.
///
/// Layout, little-endian: magic `ARTB`, version (u16, 1), count (u16); then for each picture
/// the name's length (u8), the scientific name in lower case, the picture's offset from the
/// start of the bundle (u32), its length (u32) and its length unpacked (u32); then the
/// pictures. Pictures are unpacked only when asked for, so the bundle can stay in flash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtBundle<'a> {
    bytes: &'a [u8],
    /// Each species' picture, sorted by name.
    entries: Vec<Located<'a>>,
}

/// Where one species' picture lies in the bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Located<'a> {
    name: &'a str,
    offset: usize,
    len: usize,
    unpacked_len: usize,
}

impl<'a> ArtBundle<'a> {
    /// Reads the index of a bundle. An empty slice reads as an empty bundle, so a firmware
    /// built without art still has a bundle to ask.
    ///
    /// # Errors
    ///
    /// Returns [`ArtError::NotArt`] when the bytes are not a bundle,
    /// [`ArtError::UnsupportedVersion`] for another version, or [`ArtError::Corrupt`] when the
    /// index points outside the bytes or a name is not UTF-8.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ArtError> {
        if bytes.is_empty() {
            return Ok(Self {
                bytes,
                entries: Vec::new(),
            });
        }
        let Some(header) = bytes.get(..HEADER_LEN) else {
            return Err(ArtError::NotArt);
        };
        if header[0..4] != MAGIC {
            return Err(ArtError::NotArt);
        }
        let version: u16 = u16::from_le_bytes([header[4], header[5]]);
        if version != VERSION {
            return Err(ArtError::UnsupportedVersion(version));
        }
        let count: usize = usize::from(u16::from_le_bytes([header[6], header[7]]));
        let mut entries: Vec<Located<'a>> = Vec::with_capacity(count);
        let mut at: usize = HEADER_LEN;
        for _ in 0..count {
            let name_len: usize = usize::from(*bytes.get(at).ok_or(ArtError::Corrupt)?);
            let name_bytes: &[u8] = bytes
                .get(at + 1..at + 1 + name_len)
                .ok_or(ArtError::Corrupt)?;
            let name: &str = core::str::from_utf8(name_bytes).map_err(|_| ArtError::Corrupt)?;
            at += 1 + name_len;
            let located = Located {
                name,
                offset: read_u32(bytes, at)?,
                len: read_u32(bytes, at + 4)?,
                unpacked_len: read_u32(bytes, at + 8)?,
            };
            at += 12;
            let end: Option<usize> = located.offset.checked_add(located.len);
            if end.is_none_or(|end| end > bytes.len()) || located.unpacked_len > MAX_UNPACKED {
                return Err(ArtError::Corrupt);
            }
            entries.push(located);
        }
        entries.sort_unstable_by(|left, right| left.name.cmp(right.name));
        Ok(Self { bytes, entries })
    }

    /// Lays `entries` out as a bundle. Names are stored in lower case and cut to 255 bytes; a
    /// later picture of the same species replaces an earlier one.
    #[must_use]
    pub fn encode<'n>(entries: impl IntoIterator<Item = BundleEntry<'n>>) -> Vec<u8> {
        let mut named: Vec<(String, BundleEntry<'n>)> = Vec::new();
        for entry in entries {
            let mut key: String = entry.name.trim().to_lowercase();
            while key.len() > usize::from(u8::MAX) {
                key.pop();
            }
            named.retain(|(kept, _)| *kept != key);
            named.push((key, entry));
        }
        named.truncate(usize::from(u16::MAX));
        let index_len: usize = named.iter().map(|(name, _)| 1 + name.len() + 12).sum();
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&word16(named.len()));
        let mut offset: usize = HEADER_LEN + index_len;
        for (name, entry) in &named {
            bytes.push(u8::try_from(name.len()).unwrap_or(u8::MAX));
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&word32(offset));
            bytes.extend_from_slice(&word32(entry.zlib.len()));
            bytes.extend_from_slice(&word32(entry.unpacked_len));
            offset += entry.zlib.len();
        }
        for (_, entry) in &named {
            bytes.extend_from_slice(entry.zlib);
        }
        bytes
    }

    /// Returns how many pictures the bundle holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the bundle holds no pictures.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the picture of the species with this scientific name, compared without regard
    /// to case, unpacked.
    ///
    /// # Errors
    ///
    /// Returns [`ArtError::Corrupt`] when the picture does not unpack, or another [`ArtError`]
    /// when it is not a `.artp` picture. A species that is not in the bundle is `Ok(None)`.
    pub fn picture(&self, scientific_name: &str) -> Result<Option<ArtImage>, ArtError> {
        let key: String = scientific_name.trim().to_lowercase();
        let Ok(found) = self
            .entries
            .binary_search_by(|entry| entry.name.cmp(key.as_str()))
        else {
            return Ok(None);
        };
        let located: Located<'_> = self.entries[found];
        let packed: &[u8] = self
            .bytes
            .get(located.offset..located.offset + located.len)
            .ok_or(ArtError::Corrupt)?;
        let mut artp: Vec<u8> = vec![0; located.unpacked_len];
        let filled: usize =
            frame_core::inflate_zlib(packed, &mut artp).map_err(|_| ArtError::Corrupt)?;
        if filled != located.unpacked_len {
            return Err(ArtError::Corrupt);
        }
        ArtImage::decode(&artp).map(Some)
    }
}

impl ArtSource for ArtBundle<'_> {
    fn art_for(&mut self, scientific_name: &str) -> Option<ArtImage> {
        self.picture(scientific_name).ok().flatten()
    }
}

fn read_u32(bytes: &[u8], at: usize) -> Result<usize, ArtError> {
    let word: &[u8] = bytes.get(at..at + 4).ok_or(ArtError::Corrupt)?;
    let value: u32 = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
    usize::try_from(value).map_err(|_| ArtError::Corrupt)
}

fn word16(value: usize) -> [u8; 2] {
    u16::try_from(value).unwrap_or(u16::MAX).to_le_bytes()
}

fn word32(value: usize) -> [u8; 4] {
    u32::try_from(value).unwrap_or(u32::MAX).to_le_bytes()
}

#[cfg(test)]
mod test {
    use frame_core::Resolution;

    use super::*;
    use crate::rgba::Rgba;

    fn picture(red: u8) -> ArtImage {
        let mut pixels: Vec<Rgba> = vec![Rgba::TRANSPARENT; 12];
        pixels[4] = Rgba::new(red, 10, 20, 255);
        ArtImage::new(Resolution::new(4, 3), pixels).expect("sizes match")
    }

    /// `data` as a zlib stream of one stored block, which is all a test needs.
    fn stored(data: &[u8]) -> Vec<u8> {
        let len: u16 = u16::try_from(data.len()).expect("short");
        let mut out: Vec<u8> = vec![0x78, 0x01, 0x01];
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(data);
        let (mut low, mut high): (u32, u32) = (1, 0);
        for &byte in data {
            low = (low + u32::from(byte)) % 65_521;
            high = (high + low) % 65_521;
        }
        out.extend_from_slice(&((high << 16) | low).to_be_bytes());
        out
    }

    #[test]
    fn finds_each_species_by_name_in_any_case() {
        let tit: ArtImage = picture(200);
        let robin: ArtImage = picture(90);
        let (tit_artp, robin_artp): (Vec<u8>, Vec<u8>) = (tit.encode(), robin.encode());
        let (tit_zlib, robin_zlib): (Vec<u8>, Vec<u8>) = (stored(&tit_artp), stored(&robin_artp));
        let bytes: Vec<u8> = ArtBundle::encode([
            BundleEntry {
                name: "Parus major",
                zlib: &tit_zlib,
                unpacked_len: tit_artp.len(),
            },
            BundleEntry {
                name: "Erithacus rubecula",
                zlib: &robin_zlib,
                unpacked_len: robin_artp.len(),
            },
        ]);
        let mut bundle: ArtBundle<'_> = ArtBundle::parse(&bytes).expect("a bundle");
        assert_eq!(bundle.len(), 2);
        assert_eq!(bundle.art_for("PARUS MAJOR"), Some(tit));
        assert_eq!(bundle.art_for("erithacus rubecula"), Some(robin));
        assert_eq!(bundle.picture("Pica pica"), Ok(None));
    }

    #[test]
    fn reads_nothing_as_empty_and_refuses_damage() {
        assert!(ArtBundle::parse(&[]).expect("empty").is_empty());
        assert_eq!(ArtBundle::parse(b"ARTP\x01\x00"), Err(ArtError::NotArt));
        let artp: Vec<u8> = picture(1).encode();
        let zlib: Vec<u8> = stored(&artp);
        let entry = BundleEntry {
            name: "Parus major",
            zlib: &zlib,
            unpacked_len: artp.len(),
        };
        let mut bytes: Vec<u8> = ArtBundle::encode([entry]);
        let last: usize = bytes.len() - 1;
        bytes[last] ^= 0xff;
        let bundle: ArtBundle<'_> = ArtBundle::parse(&bytes).expect("the index is intact");
        assert_eq!(bundle.picture("Parus major"), Err(ArtError::Corrupt));
        bytes.truncate(last);
        assert_eq!(ArtBundle::parse(&bytes), Err(ArtError::Corrupt));
    }
}
