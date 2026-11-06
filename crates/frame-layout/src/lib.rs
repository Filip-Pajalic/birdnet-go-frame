//! Page layout: which text and art go where, independent of how they are drawn.
//!
//! The layout is plain data ([`PageLayout`]) so it can be tested without fonts or images, and
//! drawn by `frame-raster` on any target. The pages offered: [`Collage`], every bird heard
//! packed by its outline as fugleramme does; [`TextList`], names and counts only;
//! [`StoryPage`], the day so far with a chart of when each bird sang; and [`SeasonPage`], the
//! year so far. [`SeasonBorder`] engraves a seasonal or holiday border ([`BorderTheme`]) round
//! any of them. `no_std` with `alloc`.

#![no_std]

extern crate alloc;

mod align;
mod border;
mod border_theme;
mod collage;
mod engraver;
mod footprint;
mod image_block;
mod page_frame;
mod page_layout;
mod pen_mark;
mod point;
mod season_border;
mod season_page;
mod shape_block;
mod silhouette;
mod size_weights;
mod spiral;
mod story_page;
mod text_block;
mod text_list;
mod text_style;

pub use align::Align;
pub use border::{Border, ParseBorderError};
pub use border_theme::{BorderTheme, ParseBorderThemeError};
pub use collage::{Collage, CollageBird, CollageOptions};
pub use image_block::ImageBlock;
pub use page_layout::PageLayout;
pub use pen_mark::PenMark;
pub use point::Point;
pub use season_border::SeasonBorder;
pub use season_page::{SeasonColumn, SeasonOptions, SeasonPage};
pub use shape_block::ShapeBlock;
pub use silhouette::Silhouette;
pub use size_weights::{SIZE_EXPONENT, size_weights};
pub use story_page::{ChartMark, ChorusRow, StoryOptions, StoryPage};
pub use text_block::TextBlock;
pub use text_list::{TextList, TextListOptions};
pub use text_style::TextStyle;
