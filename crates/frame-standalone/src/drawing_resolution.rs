//! How big a standalone frame draws its page.

use frame_core::{PanelSize, Resolution, Rotation, panel_bytes};

/// Returns the size an ESP32 draws the page at for `panel` hung at `rotation`: the 7.3" page
/// whole, the 13.3" page at half its width and height, which `frame_artist::panel_bytes`
/// dithers back up to the panel's full resolution.
///
/// A whole 13.3" page in full colour takes 5.8 MB, and boosting it for the panel another
/// 1.9 MB, which leaves too little of 8 MB of PSRAM for the birds' pictures; at half size it
/// takes a quarter of that.
#[must_use]
pub fn drawing_resolution(panel: PanelSize, rotation: Rotation) -> Resolution {
    let full: Resolution = panel_bytes::page_resolution(panel, rotation);
    match panel {
        PanelSize::Inches7Point3 => full,
        PanelSize::Inches13Point3 => Resolution::new(full.width / 2, full.height / 2),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn halves_only_the_large_page() {
        assert_eq!(
            drawing_resolution(PanelSize::Inches13Point3, Rotation::Degrees90),
            Resolution::new(600, 800)
        );
        assert_eq!(
            drawing_resolution(PanelSize::Inches7Point3, Rotation::Degrees0),
            Resolution::new(800, 480)
        );
    }
}
