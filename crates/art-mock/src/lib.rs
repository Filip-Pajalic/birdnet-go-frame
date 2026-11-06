//! Generated bird pictures, so the collage works out of the box.
//!
//! [`MockArt`] is an [`art_pack::ArtSource`] that paints a simple perched bird for any species:
//! the common garden birds of northern Europe get their own shape and plumage, and any other
//! species gets a shape and colours derived from its name, the same every time. The pictures
//! are cut-outs with transparent backgrounds, like real art, so everything downstream (sizing,
//! packing, drawing, dithering) is exercised without any image files. `no_std` with `alloc`.
//!
//! ```
//! use art_mock::MockArt;
//! use art_pack::ArtSource;
//!
//! let picture = MockArt::default().art_for("Erithacus rubecula").expect("always paints");
//! assert!(picture.resolution().width > 100);
//! ```

#![no_std]

extern crate alloc;

mod bird;
mod build;
mod layer;
mod mock_art;
mod painter;
mod plumage;
mod point;
mod shape;
mod species;

pub use mock_art::MockArt;
