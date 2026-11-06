//! Page layout: which text and art go where, independent of how they are drawn.
//!
//! The layout is plain data ([`PageLayout`]) so it can be tested without fonts or images, and
//! drawn by `frame-raster` on any target. Two pages are offered: [`Collage`], every bird heard
//! packed by its outline as fugleramme does, and [`TextList`], names and counts only. `no_std` with `alloc`.

#![no_std]

extern crate alloc;

mod align;
mod collage;
mod footprint;
mod image_block;
mod page_frame;
mod page_layout;
mod silhouette;
mod size_weights;
mod spiral;
mod text_block;
mod text_list;
mod text_style;

pub use align::Align;
pub use collage::{Collage, CollageBird, CollageOptions};
pub use image_block::ImageBlock;
pub use page_layout::PageLayout;
pub use silhouette::Silhouette;
pub use size_weights::{SIZE_EXPONENT, size_weights};
pub use text_block::TextBlock;
pub use text_list::{TextList, TextListOptions};
pub use text_style::TextStyle;
