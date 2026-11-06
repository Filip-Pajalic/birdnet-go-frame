//! Drawing a [`frame_layout::PageLayout`] into pixels and dithering it to the six inks.
//!
//! `no_std` with `alloc`, so the ESP32 can render a page itself.

#![no_std]

extern crate alloc;

mod canvas;
mod dither;
mod font_error;
mod renderer;

pub use canvas::Canvas;
pub use dither::dither;
pub use font_error::FontError;
pub use renderer::Renderer;
