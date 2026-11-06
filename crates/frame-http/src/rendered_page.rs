//! One rendered page in every format the server hands out.

use std::fmt;

use frame_core::{Crc32, IndexedImage, PageSizeError, PanelSize, Rotation, panel_bytes};
use frame_raster::Canvas;

use crate::panel_body::PanelBody;
use crate::target::Target;
use crate::{deflate, png_writer};

/// Why a page could not be packed.
#[derive(Debug)]
#[must_use]
pub enum RenderError {
    /// The page is not the size its target needs.
    PageSize(PageSizeError),
    /// The page is too large for PNG.
    PngTooLarge,
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PageSize(error) => error.fmt(formatter),
            Self::PngTooLarge => formatter.write_str("page too large for PNG"),
        }
    }
}

impl std::error::Error for RenderError {}

/// A page packed for every endpoint, with one ETag for all of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedPage {
    /// What `/panel` sends; `None` for a canvas, which no panel shows.
    pub panel: Option<PanelBody>,
    /// Six-colour PNG: in the panel's native orientation, or as the viewer sees a canvas.
    pub frame_png: Vec<u8>,
    /// Full-colour PNG as the viewer sees it.
    pub preview_png: Vec<u8>,
    /// The quoted CRC-32, in hex, of the panel's bytes (a canvas's six-colour PNG). Panels may
    /// check a download against it, so it stays a CRC-32.
    pub etag: String,
    /// Minutes a panel should sleep before asking again; 0 for as soon as it likes.
    pub poll_minutes: u8,
}

impl RenderedPage {
    /// Packs a dithered `page` and its full-colour `preview` for the 13.3" panel, both as the
    /// viewer sees them.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when the page does not fit the panel for `rotation` or PNG
    /// cannot hold it.
    pub fn new(
        page: &IndexedImage,
        preview: &Canvas,
        rotation: Rotation,
        poll_minutes: u8,
    ) -> Result<Self, RenderError> {
        let target = Target::Panel {
            panel: PanelSize::Inches13Point3,
            rotation,
        };
        Self::for_target(target, page, preview, poll_minutes)
    }

    /// Packs a dithered `page` and its full-colour `preview`, both as the viewer sees them, for
    /// `target`.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when the page is not the size `target` needs or PNG cannot
    /// hold it.
    pub fn for_target(
        target: Target,
        page: &IndexedImage,
        preview: &Canvas,
        poll_minutes: u8,
    ) -> Result<Self, RenderError> {
        let preview_png: Vec<u8> = png_writer::rgb_png(preview).ok_or(RenderError::PngTooLarge)?;
        let (panel, frame_png): (Option<PanelBody>, Vec<u8>) = match target {
            Target::Panel { panel, rotation } => {
                let plain: Vec<u8> =
                    panel_bytes::pack(panel, page, rotation).map_err(RenderError::PageSize)?;
                let zlib: Vec<u8> = deflate::zlib(&plain);
                let glass: IndexedImage = page.rotated(rotation);
                let png: Vec<u8> =
                    png_writer::indexed_png(&glass).ok_or(RenderError::PngTooLarge)?;
                (Some(PanelBody { plain, zlib }), png)
            }
            Target::Canvas(resolution) => {
                if page.resolution() != resolution {
                    return Err(RenderError::PageSize(PageSizeError));
                }
                (
                    None,
                    png_writer::indexed_png(page).ok_or(RenderError::PngTooLarge)?,
                )
            }
        };
        let tagged: &[u8] = panel.as_ref().map_or(&frame_png, |body| &body.plain);
        let etag: String = format!("\"{:08x}\"", Crc32::checksum(tagged));
        Ok(Self {
            panel,
            frame_png,
            preview_png,
            etag,
            poll_minutes,
        })
    }
}
