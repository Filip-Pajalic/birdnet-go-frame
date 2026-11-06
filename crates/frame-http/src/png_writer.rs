//! PNG encoding for the two picture endpoints.

use frame_core::{IndexedImage, Ink, Resolution, Rgb};
use frame_raster::Canvas;

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

fn dimensions(resolution: Resolution) -> Result<(u32, u32), png::EncodingError> {
    let too_big = |_| png::EncodingError::LimitsExceeded;
    Ok((
        u32::try_from(resolution.width).map_err(too_big)?,
        u32::try_from(resolution.height).map_err(too_big)?,
    ))
}

/// Encodes `image` as an 8-bit palette PNG with the six inks as its palette.
pub(crate) fn indexed_png(image: &IndexedImage) -> Result<Vec<u8>, png::EncodingError> {
    let (width, height) = dimensions(image.resolution())?;
    let mut palette: Vec<u8> = Vec::with_capacity(18);
    for ink in Ink::ALL {
        let colour: Rgb = display_colour(ink);
        palette.extend_from_slice(&[colour.red, colour.green, colour.blue]);
    }
    let indices: Vec<u8> = image
        .pixels()
        .iter()
        .map(|ink| ink.palette_index())
        .collect();
    let mut bytes: Vec<u8> = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(palette);
    encoder.write_header()?.write_image_data(&indices)?;
    Ok(bytes)
}

/// Encodes `canvas` as an 8-bit RGB PNG.
pub(crate) fn rgb_png(canvas: &Canvas) -> Result<Vec<u8>, png::EncodingError> {
    let (width, height) = dimensions(canvas.resolution())?;
    let mut samples: Vec<u8> = Vec::with_capacity(canvas.pixels().len() * 3);
    for pixel in canvas.pixels() {
        samples.extend_from_slice(&[pixel.red, pixel.green, pixel.blue]);
    }
    let mut bytes: Vec<u8> = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&samples)?;
    Ok(bytes)
}
