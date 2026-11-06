//! From a drawn page to the bytes a panel takes.

use alloc::vec::Vec;

use frame_core::{IndexedImage, PageSizeError, PanelSize, Rgb, Rotation, panel_bytes};
use frame_raster::Canvas;

/// Boosts `page` for the panel ([`frame_raster::pop`] against `paper`), dithers it to the six
/// inks and lays it out for `panel`'s controllers, as `/panel` serves it before compression.
///
/// # Errors
///
/// Returns [`PageSizeError`] when `page` is not the size `panel` takes in `rotation`
/// ([`panel_bytes::page_resolution`]).
pub fn panel_bytes(
    page: &Canvas,
    paper: Rgb,
    panel: PanelSize,
    rotation: Rotation,
) -> Result<Vec<u8>, PageSizeError> {
    let inks: IndexedImage = frame_raster::dither(&frame_raster::pop(page, paper));
    panel_bytes::pack(panel, &inks, rotation)
}
