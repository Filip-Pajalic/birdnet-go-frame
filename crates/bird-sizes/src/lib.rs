//! Body mass per bird species, for drawing heavier birds bigger.
//!
//! [`BirdSizes::bundled`] embeds AVONET's masses for every species BirdNET can detect (about
//! 160 KB, CC BY 4.0; see `data/ATTRIBUTION.md`). [`BirdSizes::parse_csv`] reads
//! any other `scientific_name,mass_g` table. `no_std`, no allocation, no dependencies.
//!
//! ```
//! use bird_sizes::BirdSizes;
//!
//! let sizes = BirdSizes::bundled();
//! assert_eq!(sizes.mass_grams("Parus major"), 16.3);
//! assert_eq!(sizes.mass_grams("parus  MAJOR"), 16.3);
//! assert_eq!(sizes.mass_grams("Not a bird"), sizes.median_grams());
//! ```

#![no_std]

mod bird_sizes;
mod name_order;

pub use bird_sizes::BirdSizes;
