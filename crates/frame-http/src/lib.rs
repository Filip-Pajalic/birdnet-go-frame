//! The HTTP side of the server: packing a rendered page for every kind of client and serving it.
//!
//! | Path | Body |
//! | --- | --- |
//! | `/panel` | The panel's controller bytes, zlib-compressed for clients that accept `deflate` |
//! | `/frame.png` | Six-colour PNG: a panel's native orientation, or a canvas as seen |
//! | `/preview.png` | Full-colour PNG as the viewer sees it |
//! | `/healthz` | `ok` once a page has been published, 503 before |
//! | `/api/frames` | JSON: every frame's last wake, battery and firmware ([`FrameLog`]) |
//! | `/api/frames/<name>` | JSON: one frame's |
//! | `/firmware/<name>.bin` | A firmware image, when [`FrameServer::with_firmware`] names a folder |
//!
//! Every page path takes `panel=7.3|13.3` and `rotation=0|90|180|270`, or `w` and `h` for a
//! canvas of any size (PNGs only); without them the server's configured panel is served. See
//! [`Target::from_query`]. An app may read more of the query, such as `frame=` and `view=`,
//! with [`FrameServer::with_router`], and publish a page per view with
//! [`PageStore::publish_views`]. `/panel` notes the frame's `X-Frame-Battery-mV` and
//! `X-Frame-Firmware` headers in the [`FrameLog`].

mod deflate;
mod frame_log;
mod frame_server;
mod page_store;
mod panel_body;
mod png_writer;
mod query;
mod rendered_page;
mod target;

pub use deflate::zlib;
pub use frame_log::{DEFAULT_FRAME, FrameLog, FrameVisit, MOST_NAME_BYTES, frame_name};
pub use frame_server::{FIRMWARE, FrameServer, Router, SleepPlan};
pub use page_store::{PageMaker, PageStore, ViewMaker};
pub use panel_body::PanelBody;
pub use query::query_value;
pub use rendered_page::{RenderError, RenderedPage};
pub use target::Target;
