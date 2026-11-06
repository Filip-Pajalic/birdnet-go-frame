//! Decoding, trimming and shrinking one plate.

use std::io::Cursor;

use art_pack::{ArtImage, Rgba};
use frame_core::Resolution;
use image_webp::WebPDecoder;

/// Decodes webp `bytes` to straight-alpha pixels.
pub(crate) fn decode(bytes: &[u8]) -> Option<ArtImage> {
    let mut decoder = WebPDecoder::new(Cursor::new(bytes)).ok()?;
    let (width, height) = decoder.dimensions();
    let (width, height) = (usize::try_from(width).ok()?, usize::try_from(height).ok()?);
    let channels: usize = if decoder.has_alpha() { 4 } else { 3 };
    let mut buffer: Vec<u8> = vec![0; decoder.output_buffer_size()?];
    decoder.read_image(&mut buffer).ok()?;
    let pixels: Vec<Rgba> = buffer
        .chunks_exact(channels)
        .map(|pixel| {
            Rgba::new(
                pixel[0],
                pixel[1],
                pixel[2],
                pixel.get(3).copied().unwrap_or(255),
            )
        })
        .collect();
    ArtImage::new(Resolution::new(width, height), pixels)
}

/// Returns `picture` cropped to the box around its non-transparent pixels.
pub(crate) fn trim(picture: &ArtImage) -> ArtImage {
    let Resolution { width, height } = picture.resolution();
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    for y in 0..height {
        for x in 0..width {
            if picture.get(x, y).alpha > 0 {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    if left > right || top > bottom {
        return picture.clone();
    }
    let mut pixels: Vec<Rgba> = Vec::with_capacity((right - left + 1) * (bottom - top + 1));
    for y in top..=bottom {
        for x in left..=right {
            pixels.push(picture.get(x, y));
        }
    }
    ArtImage::new(Resolution::new(right - left + 1, bottom - top + 1), pixels)
        .unwrap_or_else(|| picture.clone())
}

/// Returns `picture` shrunk so its longest side is at most `max_side`, averaging with alpha
/// weighting so transparent pixels do not darken edges. Smaller pictures are returned as they are.
pub(crate) fn shrink(picture: &ArtImage, max_side: usize) -> ArtImage {
    let Resolution { width, height } = picture.resolution();
    let longest: usize = width.max(height);
    if longest <= max_side || max_side == 0 {
        return picture.clone();
    }
    let new_width: usize = (width * max_side / longest).max(1);
    let new_height: usize = (height * max_side / longest).max(1);
    let mut pixels: Vec<Rgba> = Vec::with_capacity(new_width * new_height);
    for y in 0..new_height {
        let (top, bottom) = (
            y * height / new_height,
            ((y + 1) * height).div_ceil(new_height),
        );
        for x in 0..new_width {
            let (left, right) = (x * width / new_width, ((x + 1) * width).div_ceil(new_width));
            let mut sums: [u64; 4] = [0; 4];
            let mut count: u64 = 0;
            for source_y in top..bottom.min(height) {
                for source_x in left..right.min(width) {
                    let pixel: Rgba = picture.get(source_x, source_y);
                    let alpha: u64 = u64::from(pixel.alpha);
                    sums[0] += u64::from(pixel.red) * alpha;
                    sums[1] += u64::from(pixel.green) * alpha;
                    sums[2] += u64::from(pixel.blue) * alpha;
                    sums[3] += alpha;
                    count += 1;
                }
            }
            let channel = |total: u64| -> u8 {
                total
                    .checked_div(sums[3])
                    .map_or(0, |value| u8::try_from(value).unwrap_or(u8::MAX))
            };
            pixels.push(Rgba::new(
                channel(sums[0]),
                channel(sums[1]),
                channel(sums[2]),
                u8::try_from(sums[3] / count.max(1)).unwrap_or(u8::MAX),
            ));
        }
    }
    ArtImage::new(Resolution::new(new_width, new_height), pixels).unwrap_or_else(|| picture.clone())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trims_and_shrinks() {
        let mut pixels: Vec<Rgba> = vec![Rgba::TRANSPARENT; 100];
        for y in 2..6 {
            for x in 3..9 {
                pixels[y * 10 + x] = Rgba::new(200, 100, 0, 255);
            }
        }
        let picture = ArtImage::new(Resolution::new(10, 10), pixels).expect("100 pixels");
        let trimmed = trim(&picture);
        assert_eq!(trimmed.resolution(), Resolution::new(6, 4));
        let small = shrink(&trimmed, 3);
        assert_eq!(small.resolution(), Resolution::new(3, 2));
        assert_eq!(small.get(0, 0), Rgba::new(200, 100, 0, 255));
        assert_eq!(shrink(&small, 10), small);
    }

    #[test]
    fn rejects_bytes_that_are_not_webp() {
        assert!(decode(b"RIFF....WEBPnope").is_none());
    }
}
