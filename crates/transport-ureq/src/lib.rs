//! An [`birdnet_client::HttpTransport`] for std targets, built on `ureq`.

mod ureq_transport;

pub use ureq_transport::UreqTransport;
