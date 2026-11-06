//! The HTTP side of the server: packing a rendered page for every kind of client and serving it.
//!
//! | Path | Body |
//! | --- | --- |
//! | `/panel` | The panel's controller bytes, zlib-compressed for clients that accept `deflate` |
//! | `/frame.png` | Six-colour PNG: a panel's native orientation, or a canvas as seen |
//! | `/preview.png` | Full-colour PNG as the viewer sees it |
//! | `/healthz` | `ok` once a page has been published, 503 before |
//!
//! Every page path takes `panel=7.3|13.3` and `rotation=0|90|180|270`, or `w` and `h` for a
//! canvas of any size (PNGs only); without them the server's configured panel is served. See
//! [`Target::from_query`].

mod deflate;
mod frame_server;
mod page_store;
mod panel_body;
mod png_writer;
mod rendered_page;
mod target;

pub use frame_server::FrameServer;
pub use page_store::{PageMaker, PageStore};
pub use panel_body::PanelBody;
pub use rendered_page::{RenderError, RenderedPage};
pub use target::Target;
