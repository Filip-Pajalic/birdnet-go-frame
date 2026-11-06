//! The page's border: a spray of a plant from a botanical plate in two of its corners.

use alloc::vec::Vec;

use crate::image_block::ImageBlock;
use crate::silhouette::Silhouette;

/// A plant's picture for one corner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spray {
    /// Index of the picture in the art passed to the renderer.
    pub art: usize,
    /// The picture's outline, so nothing is packed over the plant.
    pub silhouette: Silhouette,
}

/// The border: a spray hanging from the top-left corner and one rising from the bottom-right,
/// sized to the page's heading and footer so text never runs into them. Either may be left
/// out; [`CornerSprays::default`] is no border.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CornerSprays {
    /// Hangs from the top-left corner.
    pub top_left: Option<Spray>,
    /// Rises from the bottom-right corner.
    pub bottom_right: Option<Spray>,
}

/// A placed spray, in page pixels, and which of its pixels the plant covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlacedSpray {
    pub(crate) block: ImageBlock,
    pub(crate) opaque: Vec<bool>,
}

impl Spray {
    /// Places the spray to fit a `side`-pixel square whose outer corner is `(corner_x,
    /// corner_y)`, inset from the page's edges by `inset`. `top_left` decides which way the
    /// square runs from the corner.
    pub(crate) fn place(
        &self,
        corner: (i32, i32),
        side: i32,
        inset: i32,
        top_left: bool,
    ) -> Option<PlacedSpray> {
        let room: i32 = side - inset;
        if room < 2 {
            return None;
        }
        let aspect: f32 = self.silhouette.aspect_ratio();
        let (width, height): (i32, i32) = if aspect >= 1.0 {
            (room, ((room as f32 / aspect) as i32).max(1))
        } else {
            (((room as f32 * aspect) as i32).max(1), room)
        };
        let (x, y): (i32, i32) = if top_left {
            (corner.0 + inset, corner.1 + inset)
        } else {
            (corner.0 - inset - width, corner.1 - inset - height)
        };
        let size: (usize, usize) = (usize::try_from(width).ok()?, usize::try_from(height).ok()?);
        Some(PlacedSpray {
            block: ImageBlock {
                art: self.art,
                x,
                y,
                width,
                height,
                mirrored: false,
            },
            opaque: self.silhouette.mask(size.0, size.1, false),
        })
    }
}
