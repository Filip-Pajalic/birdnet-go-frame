//! Page layout: which text and art go where, independent of how they are drawn.
//!
//! The layout is plain data ([`PageLayout`]) so it can be tested without fonts or images, and
//! drawn by `frame-raster` on any target. `no_std` with `alloc`.

#![no_std]

extern crate alloc;

mod align;
mod page_layout;
mod text_block;
mod text_list;
mod text_style;

pub use align::Align;
pub use page_layout::PageLayout;
pub use text_block::TextBlock;
pub use text_list::{TextList, TextListOptions};
pub use text_style::TextStyle;
