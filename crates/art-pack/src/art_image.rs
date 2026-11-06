//! A bird cut-out and its `.artp` encoding.

use alloc::vec::Vec;

use frame_core::Resolution;

use crate::art_error::ArtError;
use crate::rgba::Rgba;

const MAGIC: [u8; 4] = *b"ARTP";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 10;
const RUN_LEN: usize = 6;

/// A picture with transparency, row by row.
///
/// `.artp` layout, little-endian: magic `ARTP`, version (u16, 1), width (u16), height (u16),
/// then runs of identical pixels, each a count (u16, at least 1) followed by red, green, blue
/// and alpha bytes, until width times height pixels are covered. Transparent backgrounds become
/// a few long runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtImage {
    resolution: Resolution,
    pixels: Vec<Rgba>,
}

impl ArtImage {
    /// Creates a picture, or returns `None` when the pixel count does not match `resolution`.
    #[must_use]
    pub fn new(resolution: Resolution, pixels: Vec<Rgba>) -> Option<Self> {
        (pixels.len() == resolution.area()).then_some(Self { resolution, pixels })
    }

    /// Returns the picture's size.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Returns all pixels, row by row.
    #[must_use]
    pub fn pixels(&self) -> &[Rgba] {
        &self.pixels
    }

    /// Returns the pixel at (`x`, `y`), or transparent outside the picture.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Rgba {
        if x >= self.resolution.width || y >= self.resolution.height {
            return Rgba::TRANSPARENT;
        }
        self.pixels
            .get(y * self.resolution.width + x)
            .copied()
            .unwrap_or(Rgba::TRANSPARENT)
    }

    /// Returns width divided by height.
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        self.resolution.width as f32 / self.resolution.height.max(1) as f32
    }

    /// Encodes the picture as `.artp`. Pictures wider or taller than 65,535 pixels are not
    /// representable and encode as an empty picture.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let (Ok(width), Ok(height)) = (
            u16::try_from(self.resolution.width),
            u16::try_from(self.resolution.height),
        ) else {
            return Self::empty_header();
        };
        let mut bytes: Vec<u8> = Vec::with_capacity(HEADER_LEN + RUN_LEN * 64);
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        let mut index: usize = 0;
        while index < self.pixels.len() {
            let pixel: Rgba = self.pixels[index];
            let mut count: u16 = 1;
            while count < u16::MAX && self.pixels.get(index + usize::from(count)) == Some(&pixel) {
                count += 1;
            }
            bytes.extend_from_slice(&count.to_le_bytes());
            bytes.extend_from_slice(&[pixel.red, pixel.green, pixel.blue, pixel.alpha]);
            index += usize::from(count);
        }
        bytes
    }

    fn empty_header() -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(HEADER_LEN);
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes
    }

    /// Decodes `.artp` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ArtError`] when the bytes are not `.artp`, use another version, or their runs
    /// do not exactly cover the declared size.
    pub fn decode(bytes: &[u8]) -> Result<Self, ArtError> {
        let Some(header) = bytes.get(..HEADER_LEN) else {
            return Err(ArtError::NotArt);
        };
        if header[0..4] != MAGIC {
            return Err(ArtError::NotArt);
        }
        let version = u16::from_le_bytes([header[4], header[5]]);
        if version != VERSION {
            return Err(ArtError::UnsupportedVersion(version));
        }
        let width = usize::from(u16::from_le_bytes([header[6], header[7]]));
        let height = usize::from(u16::from_le_bytes([header[8], header[9]]));
        let resolution = Resolution::new(width, height);
        let mut pixels: Vec<Rgba> = Vec::with_capacity(resolution.area());
        for run in bytes[HEADER_LEN..].chunks(RUN_LEN) {
            let [low, high, red, green, blue, alpha] = run else {
                return Err(ArtError::Corrupt);
            };
            let count = usize::from(u16::from_le_bytes([*low, *high]));
            if count == 0 || pixels.len() + count > resolution.area() {
                return Err(ArtError::Corrupt);
            }
            pixels.resize(pixels.len() + count, Rgba::new(*red, *green, *blue, *alpha));
        }
        Self::new(resolution, pixels).ok_or(ArtError::Corrupt)
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::*;

    fn sample() -> ArtImage {
        let mut pixels: Vec<Rgba> = vec![Rgba::TRANSPARENT; 70_000];
        pixels[5] = Rgba::new(200, 100, 50, 255);
        pixels[69_999] = Rgba::new(1, 2, 3, 128);
        ArtImage::new(Resolution::new(350, 200), pixels).expect("sizes match")
    }

    #[test]
    fn round_trips_including_runs_longer_than_a_u16() {
        let picture = sample();
        let bytes: Vec<u8> = picture.encode();
        assert!(
            bytes.len() < 100,
            "transparent runs should compress, got {} bytes",
            bytes.len()
        );
        assert_eq!(ArtImage::decode(&bytes), Ok(picture));
    }

    #[test]
    fn rejects_damage() {
        let bytes: Vec<u8> = sample().encode();
        assert_eq!(
            ArtImage::decode(&bytes[..bytes.len() - 6]),
            Err(ArtError::Corrupt)
        );
        assert_eq!(ArtImage::decode(b"PNG"), Err(ArtError::NotArt));
        let mut wrong_version = bytes.clone();
        wrong_version[4] = 9;
        assert_eq!(
            ArtImage::decode(&wrong_version),
            Err(ArtError::UnsupportedVersion(9))
        );
    }
}
