//! The source with no pictures.

use crate::art_image::ArtImage;
use crate::art_source::ArtSource;

/// Has no picture for any species: what a frame uses before fugleramme's plates are installed.
///
/// A collage with no pictures is drawn as the list instead; see `frame_artist::Artist`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoArt;

impl ArtSource for NoArt {
    fn art_for(&mut self, _scientific_name: &str) -> Option<ArtImage> {
        None
    }
}
