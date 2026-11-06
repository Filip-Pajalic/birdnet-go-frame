//! The page format shared by every birdnet-go-frame binary.
//!
//! This crate is `no_std` (it needs only `alloc`) so the server, the Linux panel app and the
//! ESP32 firmware all compile the same definitions of the six inks, the panel geometry and the
//! byte layout the panels' controllers take ([`panel_bytes`]), which `/panel` serves.

#![no_std]

extern crate alloc;

mod crc32;
mod glass;
mod indexed_image;
mod inflate;
mod ink;
mod page_size_error;
pub mod panel_bytes;
mod panel_size;
mod resolution;
mod rgb;
mod rotation;

pub use crc32::Crc32;
pub use glass::Glass;
pub use indexed_image::IndexedImage;
pub use inflate::{InflateError, inflate_zlib};
pub use ink::Ink;
pub use page_size_error::PageSizeError;
pub use panel_size::{PanelSize, ParsePanelSizeError};
pub use resolution::Resolution;
pub use rgb::Rgb;
pub use rotation::{ParseRotationError, Rotation};
