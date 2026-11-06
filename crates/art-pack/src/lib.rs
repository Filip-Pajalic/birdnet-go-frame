//! Bird pictures for the collage.
//!
//! [`ArtImage`] is a cut-out with transparency. [`ArtSource`] finds the picture for a species,
//! wherever pictures come from: generated (`art-mock`), files on disk, or an SD card on the
//! ESP32. The `.artp` encoding ([`ArtImage::encode`], [`ArtImage::decode`]) is a simple
//! run-length format that needs no image library to read, so it suits microcontrollers.
//! `no_std` with `alloc`.
//!
//! ```
//! use art_pack::{ArtImage, Rgba};
//! use frame_core::Resolution;
//!
//! let picture = ArtImage::new(Resolution::new(3, 1), vec![Rgba::TRANSPARENT; 3]).expect("3 pixels");
//! let bytes: Vec<u8> = picture.encode();
//! assert_eq!(ArtImage::decode(&bytes), Ok(picture));
//! ```

#![no_std]

extern crate alloc;

mod art_error;
mod art_image;
mod art_source;
mod rgba;
mod subject;

pub use art_error::ArtError;
pub use art_image::ArtImage;
pub use art_source::ArtSource;
pub use rgba::Rgba;
pub use subject::Subject;
