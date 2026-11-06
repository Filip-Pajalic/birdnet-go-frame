//! The HTTP side of the server: packing a rendered page for every kind of client and serving it.
//!
//! | Path | Body |
//! | --- | --- |
//! | `/frame.e6` | E6F1 page for a panel (ETag, 304 when unchanged) |
//! | `/frame.png` | Six-colour PNG: a panel's native orientation, or a canvas as seen |
//! | `/preview.png` | Full-colour PNG as the viewer sees it |
//! | `/healthz` | `ok` once a page has been published, 503 before |
//!
//! Every page path takes `panel=7.3|13.3` and `rotation=0|90|180|270`, or `w` and `h` for a
//! canvas of any size (PNG only); without them the server's configured panel is served. See
//! [`Target::from_query`].

mod deflate;
mod frame_server;
mod page_store;
mod png_writer;
mod rendered_page;
mod target;

pub use frame_server::FrameServer;
pub use page_store::{PageMaker, PageStore};
pub use rendered_page::{RenderError, RenderedPage};
pub use target::Target;
