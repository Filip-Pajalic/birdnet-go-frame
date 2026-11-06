//! Packing a page into `/frame.e6` for a Spectra 6 panel.
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

/// The panel [`encode`] packs for: the 13.3" glass, two controllers, each driving one half of
/// every row.
pub const PANEL: PanelSize = PanelSize::Inches13Point3;

/// Returns the resolution a page for the 13.3" panel must have, as the viewer sees it, for
/// `rotation`.
#[must_use]
pub fn page_resolution(rotation: Rotation) -> Resolution {
    page_resolution_for(PANEL, rotation)
}

/// Returns the resolution a page for `panel` must have, as the viewer sees it, for `rotation`.
#[must_use]
pub fn page_resolution_for(panel: PanelSize, rotation: Rotation) -> Resolution {
    let native: Resolution = panel.native_resolution();
    if rotation.is_portrait() {
        native.transposed()
    } else {
        native
    }
}

/// Packs `page`, as it hangs on the wall, into a complete E6F1 page for the 13.3" panel.
///
/// The same as [`encode_for`] with [`PANEL`].
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
    encode_for(PANEL, page, rotation, poll_minutes)
}

/// Packs `page`, as it hangs on the wall, into a complete E6F1 page for `panel`: header and
/// payload.
///
/// For the 13.3" panel (fugleramme's layout) the page is turned back to the panel's native
/// landscape by `rotation`, then a quarter turn clockwise to the controllers' own orientation,
/// and every row is split between the two controllers: all left halves, then all right halves.
/// The 7.3" panel has one controller and takes its native landscape rows (800 wide) in order.
/// The header's width and height tell a client which layout it holds.
///
/// # Errors
///
/// Returns [`E6Error::WrongResolution`] when `page` does not have the size
/// [`page_resolution_for`] gives for `panel` and `rotation`.
pub fn encode_for(
    panel: PanelSize,
    page: &IndexedImage,
    rotation: Rotation,
    poll_minutes: u8,
) -> Result<Vec<u8>, E6Error> {
    if page.resolution() != page_resolution_for(panel, rotation) {
        return Err(E6Error::WrongResolution);
    }
    let (glass, halves): (IndexedImage, usize) = match panel {
        PanelSize::Inches13Point3 => (page.rotated(rotation).rotated(Rotation::Degrees270), 2),
        PanelSize::Inches7Point3 => (page.rotated(rotation), 1),
    };
    let Resolution { width, height } = glass.resolution();
    let part_width: usize = width / halves;
    let payload_len: usize = width * height / 2;

    let mut page_bytes: Vec<u8> = Vec::with_capacity(E6Header::LEN + payload_len);
    page_bytes.extend_from_slice(&[0u8; E6Header::LEN]);
    let pixels = glass.pixels();
    for half in 0..halves {
        for y in 0..height {
            let start: usize = y * width + half * part_width;
            let row = &pixels[start..start + part_width];
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
