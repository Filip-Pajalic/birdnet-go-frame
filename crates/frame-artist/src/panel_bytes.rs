//! From a drawn page to the bytes a panel takes.

use alloc::vec::Vec;

use frame_core::{IndexedImage, PageSizeError, PanelSize, Resolution, Rgb, Rotation, panel_bytes};
use frame_raster::Canvas;

/// Boosts `page` for the panel ([`frame_raster::pop`] against `paper`), dithers it to the six
/// inks and lays it out for `panel`'s controllers, as `/panel` serves it before compression.
///
/// `page` is drawn at the size `panel` takes in `rotation` ([`panel_bytes::page_resolution`]),
/// or at a half, a third and so on of it: a smaller page is dithered as if enlarged, which is how
/// a device short of memory draws the 13.3" page. The page is changed in place and dropped
/// before the bytes are laid out, so at most one page and its inks are held at a time.
///
/// # Errors
///
/// Returns [`PageSizeError`] when `page` is neither the size `panel` takes in `rotation` nor a
/// whole fraction of it.
pub fn panel_bytes(
    mut page: Canvas,
    paper: Rgb,
    panel: PanelSize,
    rotation: Rotation,
) -> Result<Vec<u8>, PageSizeError> {
    let full: Resolution = panel_bytes::page_resolution(panel, rotation);
    let drawn: Resolution = page.resolution();
    if drawn.width == 0 || !full.width.is_multiple_of(drawn.width) {
        return Err(PageSizeError);
    }
    let scale: usize = full.width / drawn.width;
    if drawn.height * scale != full.height {
        return Err(PageSizeError);
    }
    frame_raster::pop_in_place(&mut page, paper);
    let inks: IndexedImage = frame_raster::dither_scaled(&page, scale);
    drop(page);
    panel_bytes::pack(panel, &inks, rotation)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn takes_the_full_page_or_a_whole_fraction_of_it() {
        let paper = Rgb::new(233, 225, 202);
        let full = Canvas::new(Resolution::new(800, 480), paper);
        let half = Canvas::new(Resolution::new(400, 240), paper);
        let odd = Canvas::new(Resolution::new(300, 180), paper);
        let squashed = Canvas::new(Resolution::new(400, 480), paper);
        let small = PanelSize::Inches7Point3;
        let upright = Rotation::Degrees0;
        let from_full: Vec<u8> = panel_bytes(full, paper, small, upright).expect("full size");
        let from_half: Vec<u8> = panel_bytes(half, paper, small, upright).expect("half size");
        assert_eq!(from_full.len(), 192_000);
        assert_eq!(from_full, from_half);
        assert_eq!(panel_bytes(odd, paper, small, upright), Err(PageSizeError));
        assert_eq!(
            panel_bytes(squashed, paper, small, upright),
            Err(PageSizeError)
        );
    }
}
