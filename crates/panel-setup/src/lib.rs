//! A panel's setup hotspot, apart from driving the radio.
//!
//! `no_std` with `alloc`, so the ESP32 firmware and a test on a laptop run the same code. When
//! a frame has no settings, or its refresh button is held at power-up, it opens its own Wi-Fi
//! network and serves one page:
//!
//! ```text
//! phone joins "BirdFrame-1a2b"   DhcpServer::answer()       an address, this frame as router and DNS
//! phone looks up any name        dns::captive_answer()       every name is this frame
//! phone opens the page           SetupRequest::split()       GET / → setup_page::render()
//! person presses Save            FrameSettings::from_form()  checked, then FrameSettings::to_record()
//! ```
//!
//! The firmware writes the record to its own flash partition and restarts; each wake reads it
//! back with [`FrameSettings::from_record`].

#![no_std]

extern crate alloc;

mod dhcp_server;
pub mod dns;
mod form;
mod frame_settings;
mod settings_error;
pub mod setup_page;
mod setup_request;

pub use dhcp_server::DhcpServer;
pub use frame_settings::FrameSettings;
pub use settings_error::SettingsError;
pub use setup_request::{Method, SetupRequest, http_response, redirect_to_page};
