//! The page format shared by every birdnet-go-frame binary.
//!
//! This crate is `no_std` (it needs only `alloc`) so the server, the Linux panel app and the
//! ESP32 firmware all compile the same definitions of the six inks, the panel geometry and the
//! `/frame.e6` wire format. Nothing here allocates on the hot path of the firmware: a client
//! that streams a page reads it through [`E6Header`] and [`Crc32`] alone.

#![no_std]

extern crate alloc;

mod crc32;
pub mod e6;
mod e6_header;
mod indexed_image;
mod ink;
mod panel_size;
mod resolution;
mod rgb;
mod rotation;

pub use crc32::Crc32;
pub use e6_header::{E6Error, E6Header, Encoding};
pub use indexed_image::IndexedImage;
pub use ink::Ink;
pub use panel_size::{PanelSize, ParsePanelSizeError};
pub use resolution::Resolution;
pub use rgb::Rgb;
pub use rotation::{ParseRotationError, Rotation};
