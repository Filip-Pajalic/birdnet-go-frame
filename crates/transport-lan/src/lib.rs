//! An [`birdnet_client::HttpTransport`] for std targets, built on `lan-http`.

mod lan_transport;

pub use lan_transport::LanTransport;
