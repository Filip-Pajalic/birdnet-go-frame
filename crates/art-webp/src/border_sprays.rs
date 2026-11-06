//! The border's plants, bundled with the program, as an [`ArtSource`].

use art_pack::{ArtImage, ArtSource};

use crate::plate;

/// Includes the sprays `assets/borders/<theme>-<corner>.webp` for each theme named.
macro_rules! sprays {
    ($($theme:literal),* $(,)?) => {
        &[$(
            (
                concat!($theme, "-top"),
                include_bytes!(concat!("../../../assets/borders/", $theme, "-top.webp")),
            ),
            (
                concat!($theme, "-bottom"),
                include_bytes!(concat!("../../../assets/borders/", $theme, "-bottom.webp")),
            ),
        )*]
    };
}

/// Every spray, by name: a theme of `frame_layout::BorderTheme` and its corner.
const SPRAYS: &[(&str, &[u8])] = sprays![
    "frost",
    "blossom",
    "meadow",
    "autumn",
    "christmas",
    "easter",
    "midsummer",
];

/// The plants the page's borders are made of, cut from the hand-coloured plates of Thomé's
/// *Flora von Deutschland, Österreich und der Schweiz* (1885, public domain; see
/// `assets/borders/ATTRIBUTION.md`). Each theme has a spray named `<theme>-top`, which hangs
/// from the page's top-left corner, and `<theme>-bottom`, which rises from its bottom-right.
///
/// ```
/// use art_pack::ArtSource;
/// use art_webp::BorderSprays;
///
/// let mut sprays = BorderSprays::bundled();
/// assert!(sprays.art_for("christmas-top").is_some());
/// assert!(sprays.art_for("Parus major").is_none());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct BorderSprays {
    max_side: usize,
}

impl BorderSprays {
    /// Sprays are shrunk to at most this many pixels on their longest side by default: a
    /// corner of a 1600-pixel page.
    pub const MAX_SIDE: usize = 400;

    /// The sprays built into the program.
    #[must_use]
    pub fn bundled() -> Self {
        Self {
            max_side: Self::MAX_SIDE,
        }
    }

    /// Returns the sprays shrunk to at most `max_side` pixels on their longest side, e.g. to
    /// carry them in a small device's flash.
    #[must_use]
    pub fn with_max_side(mut self, max_side: usize) -> Self {
        self.max_side = max_side;
        self
    }

    /// Returns every spray's name, e.g. `frost-top`.
    pub fn names() -> impl Iterator<Item = &'static str> {
        SPRAYS.iter().map(|(name, _)| *name)
    }
}

impl ArtSource for BorderSprays {
    fn art_for(&mut self, name: &str) -> Option<ArtImage> {
        let (_, bytes) = SPRAYS.iter().find(|(spray, _)| *spray == name)?;
        let picture: ArtImage = plate::decode(bytes)?;
        Some(plate::shrink(&plate::trim(&picture), self.max_side))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn every_spray_decodes_with_some_plant_on_it() {
        let mut sprays = BorderSprays::bundled();
        for name in BorderSprays::names() {
            let picture: ArtImage = sprays.art_for(name).expect(name);
            let longest: usize = picture.resolution().width.max(picture.resolution().height);
            assert!(
                longest <= BorderSprays::MAX_SIDE,
                "{name} is {longest} pixels"
            );
            let opaque: usize = picture
                .pixels()
                .iter()
                .filter(|pixel| pixel.alpha > 128)
                .count();
            let share: f32 = opaque as f32 / picture.pixels().len() as f32;
            assert!((0.05..0.9).contains(&share), "{name} is {share} plant");
        }
        assert_eq!(BorderSprays::names().count(), 14);
    }

    #[test]
    fn shrinks_on_request() {
        let mut sprays = BorderSprays::bundled().with_max_side(120);
        let picture: ArtImage = sprays.art_for("autumn-bottom").expect("autumn");
        assert!(picture.resolution().width.max(picture.resolution().height) <= 120);
    }
}
