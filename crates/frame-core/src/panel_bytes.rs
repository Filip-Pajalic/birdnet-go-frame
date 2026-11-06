//! Laying a page out as the bytes a Spectra 6 panel's controllers take.
//!
//! The layout follows fugleramme's `e6.py` payload (MIT, Copyright (c) 2026 Arne Giacomo
//! Munthe-Kaas; see NOTICE.md), without its header: the server sends it zlib-compressed over HTTP,
//! and the response carries what the header did.

use alloc::vec::Vec;

use crate::indexed_image::IndexedImage;
use crate::page_size_error::PageSizeError;
use crate::panel_size::PanelSize;
use crate::resolution::Resolution;
use crate::rotation::Rotation;

/// Returns the resolution a page for `panel` must have, as the viewer sees it, for `rotation`.
#[must_use]
pub fn page_resolution(panel: PanelSize, rotation: Rotation) -> Resolution {
    let native: Resolution = panel.native_resolution();
    if rotation.is_portrait() {
        native.transposed()
    } else {
        native
    }
}

/// Returns how many bytes [`pack`] gives for `panel`: two pixels a byte.
#[must_use]
pub fn len(panel: PanelSize) -> usize {
    panel.native_resolution().area() / 2
}

/// Lays `page`, as it hangs on the wall, out as `panel`'s controllers take it.
///
/// Each pixel is its ink's 4-bit [controller code](crate::Ink::controller_code), two pixels a
/// byte with the left one in the high nibble.
///
/// For the 13.3" panel (fugleramme's layout) the page is turned back to the panel's native
/// landscape by `rotation`, then a quarter turn clockwise to the controllers' own orientation
/// (1200x1600), and every row is split between the two controllers: all left halves, then all
/// right halves. The 7.3" panel has one controller and takes its native landscape rows (800 wide)
/// in order.
///
/// # Errors
///
/// Returns [`PageSizeError`] when `page` does not have the size [`page_resolution`] gives for
/// `panel` and `rotation`.
pub fn pack(
    panel: PanelSize,
    page: &IndexedImage,
    rotation: Rotation,
) -> Result<Vec<u8>, PageSizeError> {
    if page.resolution() != page_resolution(panel, rotation) {
        return Err(PageSizeError);
    }
    let (glass, halves): (IndexedImage, usize) = match panel {
        PanelSize::Inches13Point3 => (page.rotated(rotation).rotated(Rotation::Degrees270), 2),
        PanelSize::Inches7Point3 => (page.rotated(rotation), 1),
    };
    let Resolution { width, height } = glass.resolution();
    let part_width: usize = width / halves;
    let mut bytes: Vec<u8> = Vec::with_capacity(len(panel));
    let pixels = glass.pixels();
    for half in 0..halves {
        for y in 0..height {
            let start: usize = y * width + half * part_width;
            let row = &pixels[start..start + part_width];
            for pair in row.chunks_exact(2) {
                bytes.push((pair[0].controller_code() << 4) | pair[1].controller_code());
            }
        }
    }
    Ok(bytes)
}
