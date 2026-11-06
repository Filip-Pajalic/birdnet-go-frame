//! One rendered page in every format the server hands out.

use std::fmt;

use frame_core::{Crc32, E6Error, IndexedImage, Rotation, e6};
use frame_raster::Canvas;

use crate::png_writer;

/// Why a page could not be packed.
#[derive(Debug)]
#[must_use]
pub enum RenderError {
    /// The page does not fit the E6F1 panel.
    E6(E6Error),
    /// PNG encoding failed.
    Png(png::EncodingError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::E6(error) => write!(formatter, "cannot pack /frame.e6: {error}"),
            Self::Png(error) => write!(formatter, "cannot encode PNG: {error}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// A page packed for every endpoint, with one ETag for all of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedPage {
    /// The E6F1 page.
    pub e6: Vec<u8>,
    /// Six-colour PNG in the panel's native orientation.
    pub frame_png: Vec<u8>,
    /// Full-colour PNG as the viewer sees it.
    pub preview_png: Vec<u8>,
    /// Quoted ETag derived from the E6F1 bytes.
    pub etag: String,
}

impl RenderedPage {
    /// Packs a dithered `page` and its full-colour `preview`, both as the viewer sees them.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when the page does not fit the panel for `rotation` or PNG
    /// encoding fails.
    pub fn new(
        page: &IndexedImage,
        preview: &Canvas,
        rotation: Rotation,
        poll_minutes: u8,
    ) -> Result<Self, RenderError> {
        let e6: Vec<u8> = e6::encode(page, rotation, poll_minutes).map_err(RenderError::E6)?;
        let glass: IndexedImage = page.rotated(rotation);
        let frame_png: Vec<u8> = png_writer::indexed_png(&glass).map_err(RenderError::Png)?;
        let preview_png: Vec<u8> = png_writer::rgb_png(preview).map_err(RenderError::Png)?;
        let etag: String = format!("\"{:08x}\"", Crc32::checksum(&e6));
        Ok(Self {
            e6,
            frame_png,
            preview_png,
            etag,
        })
    }
}
