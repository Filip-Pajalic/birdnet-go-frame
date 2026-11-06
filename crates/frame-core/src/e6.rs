//! Packing a page into `/frame.e6` for the 13.3" Spectra 6 panel.
//!
//! The format and its layout follow fugleramme's `e6.py` (MIT, Copyright (c) 2026 Arne Giacomo
//! Munthe-Kaas; see NOTICE.md), so this crate's pages can be shown by fugleramme's EE02
//! firmware and fugleramme's pages by ours.

use alloc::vec::Vec;

use crate::crc32::Crc32;
use crate::e6_header::{E6Error, E6Header, Encoding};
use crate::indexed_image::IndexedImage;
use crate::panel_size::PanelSize;
use crate::resolution::Resolution;
use crate::rotation::Rotation;

/// The only panel E6F1 packs for: two controllers, each driving one half of every row.
pub const PANEL: PanelSize = PanelSize::Inches13Point3;

/// Returns the resolution a page must have, as the viewer sees it, for `rotation`.
#[must_use]
pub fn page_resolution(rotation: Rotation) -> Resolution {
    let native: Resolution = PANEL.native_resolution();
    if rotation.is_portrait() {
        native.transposed()
    } else {
        native
    }
}

/// Packs `page`, as it hangs on the wall, into a complete E6F1 page: header and payload.
///
/// The page is turned back to the panel's native landscape by `rotation`, then a quarter turn
/// clockwise to the controllers' own orientation, and every row is split between the two
/// controllers.
///
/// # Errors
///
/// Returns [`E6Error::WrongResolution`] when `page` does not have the size
/// [`page_resolution`] gives for `rotation`.
pub fn encode(
    page: &IndexedImage,
    rotation: Rotation,
    poll_minutes: u8,
) -> Result<Vec<u8>, E6Error> {
    if page.resolution() != page_resolution(rotation) {
        return Err(E6Error::WrongResolution);
    }
    let glass: IndexedImage = page.rotated(rotation).rotated(Rotation::Degrees270);
    let Resolution { width, height } = glass.resolution();
    let half_width: usize = width / 2;
    let payload_len: usize = width * height / 2;

    let mut page_bytes: Vec<u8> = Vec::with_capacity(E6Header::LEN + payload_len);
    page_bytes.extend_from_slice(&[0u8; E6Header::LEN]);
    let pixels = glass.pixels();
    for half in 0..2 {
        for y in 0..height {
            let start: usize = y * width + half * half_width;
            let row = &pixels[start..start + half_width];
            for pair in row.chunks_exact(2) {
                page_bytes.push((pair[0].controller_code() << 4) | pair[1].controller_code());
            }
        }
    }

    let header = E6Header {
        encoding: Encoding::Nibbles,
        poll_minutes,
        width: u16::try_from(width).map_err(|_| E6Error::WrongResolution)?,
        height: u16::try_from(height).map_err(|_| E6Error::WrongResolution)?,
        payload_len: u32::try_from(payload_len).map_err(|_| E6Error::WrongResolution)?,
        payload_crc: Crc32::checksum(&page_bytes[E6Header::LEN..]),
    };
    page_bytes[..E6Header::LEN].copy_from_slice(&header.to_bytes());
    Ok(page_bytes)
}
