//! The generated-art source.

use art_pack::{ArtImage, ArtSource};

use crate::{bird, painter, species};

/// Paints a bird for any species. See the crate documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MockArt {
    size_px: usize,
}

impl MockArt {
    /// Creates a source whose pictures are `size_px` pixels along their longest side.
    #[must_use]
    pub fn new(size_px: usize) -> Self {
        Self { size_px }
    }

    /// Returns the picture of a species. Every species gets one.
    #[must_use]
    pub fn paint(&self, scientific_name: &str) -> ArtImage {
        let (build, plumage) = species::look(scientific_name);
        painter::paint(
            &bird::layers(&build, &plumage),
            bird::OUTLINE_WIDTH,
            self.size_px,
        )
    }
}

impl Default for MockArt {
    /// Pictures 480 pixels on their longest side: enough for the largest bird on a 13.3" page.
    fn default() -> Self {
        Self::new(480)
    }
}

impl ArtSource for MockArt {
    fn art_for(&mut self, scientific_name: &str) -> Option<ArtImage> {
        Some(self.paint(scientific_name))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn paints_cut_outs_of_the_requested_size() {
        let art = MockArt::new(200);
        for name in [
            "Parus major",
            "Pica pica",
            "Dendrocopos major",
            "Strix aluco",
        ] {
            let picture = art.paint(name);
            let size = picture.resolution();
            assert!(
                size.width.max(size.height).abs_diff(200) <= 3,
                "{name}: {size:?}"
            );
            // Cut-outs: the corners are see-through and most of the middle row is not.
            assert_eq!(picture.get(0, 0).alpha, 0, "{name}");
            let middle_row = (0..size.width).filter(|&x| picture.get(x, size.height / 2).alpha > 0);
            assert!(middle_row.count() > size.width / 3, "{name}");
        }
    }

    #[test]
    fn the_same_species_paints_the_same_picture() {
        let art = MockArt::new(120);
        assert_eq!(art.paint("Turdus merula"), art.paint("turdus merula"));
        assert_ne!(art.paint("Turdus merula"), art.paint("Erithacus rubecula"));
    }
}
