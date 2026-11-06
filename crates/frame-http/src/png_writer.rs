//! PNG encoding for the two picture endpoints.

use frame_core::{Crc32, IndexedImage, Ink, Resolution, Rgb};
use frame_raster::Canvas;

use crate::deflate;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
const COLOUR_INDEXED: u8 = 3;
const COLOUR_RGB: u8 = 2;

/// How a six-ink page looks on a computer screen: the inks' nominal colours.
fn display_colour(ink: Ink) -> Rgb {
    match ink {
        Ink::Black => Rgb::new(0, 0, 0),
        Ink::White => Rgb::new(255, 255, 255),
        Ink::Yellow => Rgb::new(255, 255, 0),
        Ink::Red => Rgb::new(255, 0, 0),
        Ink::Blue => Rgb::new(0, 0, 255),
        Ink::Green => Rgb::new(0, 255, 0),
    }
}

/// Encodes `image` as a 4-bit palette PNG with the six inks as its palette. Returns `None`
/// for images too large for PNG.
pub(crate) fn indexed_png(image: &IndexedImage) -> Option<Vec<u8>> {
    let Resolution { width, height } = image.resolution();
    let mut palette: Vec<u8> = Vec::with_capacity(18);
    for ink in Ink::ALL {
        let colour: Rgb = display_colour(ink);
        palette.extend_from_slice(&[colour.red, colour.green, colour.blue]);
    }
    let row_bytes: usize = width.div_ceil(2);
    let mut raw: Vec<u8> = Vec::with_capacity((row_bytes + 1) * height);
    for row in image.pixels().chunks(width.max(1)) {
        raw.push(0); // No filter.
        for pair in row.chunks(2) {
            let high: u8 = pair[0].palette_index();
            let low: u8 = pair.get(1).map_or(0, |ink| ink.palette_index());
            raw.push((high << 4) | low);
        }
    }
    encode(image.resolution(), 4, COLOUR_INDEXED, Some(&palette), &raw)
}

/// Encodes `canvas` as an 8-bit RGB PNG. Returns `None` for images too large for PNG.
pub(crate) fn rgb_png(canvas: &Canvas) -> Option<Vec<u8>> {
    let width: usize = canvas.resolution().width;
    let mut raw: Vec<u8> =
        Vec::with_capacity(canvas.pixels().len() * 3 + canvas.resolution().height);
    for row in canvas.pixels().chunks(width.max(1)) {
        raw.push(0); // No filter.
        for pixel in row {
            raw.extend_from_slice(&[pixel.red, pixel.green, pixel.blue]);
        }
    }
    encode(canvas.resolution(), 8, COLOUR_RGB, None, &raw)
}

fn encode(
    resolution: Resolution,
    bit_depth: u8,
    colour_type: u8,
    palette: Option<&[u8]>,
    raw: &[u8],
) -> Option<Vec<u8>> {
    let width: u32 = u32::try_from(resolution.width).ok()?;
    let height: u32 = u32::try_from(resolution.height).ok()?;
    let mut header: Vec<u8> = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    // Bit depth, colour type, deflate compression, adaptive filtering, no interlace.
    header.extend_from_slice(&[bit_depth, colour_type, 0, 0, 0]);
    let compressed: Vec<u8> = deflate::zlib(raw);
    let mut png: Vec<u8> = Vec::with_capacity(compressed.len() + 128);
    png.extend_from_slice(&SIGNATURE);
    chunk(&mut png, *b"IHDR", &header)?;
    if let Some(palette) = palette {
        chunk(&mut png, *b"PLTE", palette)?;
    }
    chunk(&mut png, *b"IDAT", &compressed)?;
    chunk(&mut png, *b"IEND", &[])?;
    Some(png)
}

fn chunk(png: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) -> Option<()> {
    png.extend_from_slice(&u32::try_from(data.len()).ok()?.to_be_bytes());
    let mut crc = Crc32::new();
    crc.update(&kind);
    crc.update(data);
    png.extend_from_slice(&kind);
    png.extend_from_slice(data);
    png.extend_from_slice(&crc.finish().to_be_bytes());
    Some(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn writes_a_well_formed_indexed_png() {
        let mut image = IndexedImage::new(Resolution::new(3, 2), Ink::White);
        image.set(2, 1, Ink::Red);
        let png: Vec<u8> = indexed_png(&image).expect("small");
        assert_eq!(png[..8], SIGNATURE);
        // IHDR: length 13, then width 3, height 2, depth 4, indexed.
        assert_eq!(png[8..12], 13_u32.to_be_bytes());
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(png[16..26], [0, 0, 0, 3, 0, 0, 0, 2, 4, 3]);
        assert!(png.windows(4).any(|window| window == b"PLTE"));
        assert!(png.ends_with(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82]));
    }
}
