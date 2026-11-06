//! Where bird pictures come from.

use crate::art_image::ArtImage;

/// Finds the picture for a species.
pub trait ArtSource {
    /// Returns a picture of the species with this scientific name (e.g. `Parus major`, compared
    /// without regard to case), or `None` when there is none.
    fn art_for(&mut self, scientific_name: &str) -> Option<ArtImage>;
}
