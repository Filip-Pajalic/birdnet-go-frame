//! The HTTP side of the server: packing a rendered page for every kind of client and serving it.
//!
//! | Path | Body |
//! | --- | --- |
//! | `/frame.e6` | E6F1 page for the 13.3" panel (ETag, 304 when unchanged) |
//! | `/frame.png` | Six-colour PNG in the panel's native orientation |
//! | `/preview.png` | Full-colour PNG as the viewer sees it |
//! | `/healthz` | `ok` once a page has been rendered, 503 before |

mod frame_server;
mod page_store;
mod png_writer;
mod rendered_page;

pub use frame_server::FrameServer;
pub use page_store::PageStore;
pub use rendered_page::{RenderError, RenderedPage};
