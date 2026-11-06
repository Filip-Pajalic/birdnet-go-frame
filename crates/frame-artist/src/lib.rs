//! Draws the page for a list of birds, the same way on every platform.
//!
//! [`Artist`] picks the layout ([`Layout`]), finds each bird's picture and size, and renders a
//! [`frame_raster::Canvas`]; [`panel_bytes`] turns that into the bytes a Spectra 6 panel's
//! controllers take. [`Artist::draw_featured`] puts one bird first and biggest (the rare-bird
//! alert), [`Artist::draw_story`] draws the day's story from [`DayFacts`] and
//! [`Artist::draw_season`] the year so far from [`YearFacts`]. The server, the Raspberry Pi app and the standalone ESP32 firmware all draw
//! with it. `no_std` with `alloc`.
//!
//! ```
//! use art_pack::NoArt;
//! use birdnet_model::SpeciesSummary;
//! use frame_artist::{Artist, Layout, PageStyle};
//! use frame_core::{PanelSize, Resolution, Rotation};
//! use frame_raster::Renderer;
//!
//! let font: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");
//! let renderer = Renderer::new(font).expect("the bundled font loads");
//! let style = PageStyle { layout: Layout::List, ..PageStyle::default() };
//! let mut artist = Artist::new(renderer, NoArt, style);
//! let species: Vec<SpeciesSummary> =
//!     SpeciesSummary::parse_list(br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":3}]"#)
//!         .expect("valid JSON");
//! let page = artist.draw(&species, "Updated 07:30", Resolution::new(800, 480));
//! let paper = artist.paper();
//! let bytes = frame_artist::panel_bytes(page, paper, PanelSize::Inches7Point3, Rotation::Degrees0)
//!     .expect("an 800x480 page fits the 7.3-inch panel");
//! assert_eq!(bytes.len(), 192_000);
//! ```

#![no_std]

extern crate alloc;

mod artist;
mod day_facts;
mod layout;
mod page_style;
mod page_view;
mod panel_bytes;
mod year_facts;

pub use artist::Artist;
pub use day_facts::{CHART_SPECIES, DayFacts};
pub use layout::{Layout, ParseLayoutError};
pub use page_style::PageStyle;
pub use page_view::{PageView, ParseViewError};
pub use panel_bytes::panel_bytes;
pub use year_facts::{ABSENCE_DAYS, ARRIVAL_DAYS, YearFacts};
