//! The ESP32's side of the network: join the Wi-Fi, fast when it can, and fetch one HTTP
//! response; or open a hotspot and answer what a phone asks it.
//!
//! `no_std` with `alloc`, on [`esp-radio`](https://docs.rs/esp-radio) for the radio and
//! [`embassy-net`](https://docs.rs/embassy-net) for TCP/IP. The decisions (what to ask for, what
//! to remember, how long to sleep) live in `panel-wake`, which this crate only carries bytes for:
//!
//! ```text
//! join(controller, credentials, rejoin)   known access point and channel, or a full scan
//! network_config(rejoin)                  the last lease as a static address, or DHCP
//! wait_for_address(stack)                 an address, within a time limit
//! get(stack, frame, request, response)    one request, the whole response, connection closed
//! rejoin(joined, stack)                   what to keep for next time
//!
//! scan(controller)                        the networks in range, for the setup page
//! open_hotspot(controller, name)          the frame's own open network
//! serve_udp(stack, port, reply_to, f)     DHCP and DNS answers on the hotspot
//! serve_http(stack, port, limit, idle, f) the setup page
//! ```

#![no_std]

extern crate alloc;

mod get;
mod hotspot;
mod join;
mod serve;
mod transport_error;
mod wifi_credentials;

pub use get::{get, wait_for_address};
pub use hotspot::{HOTSPOT_ADDRESS, hotspot_config, open_hotspot, scan};
pub use join::{Joined, join, network_config, rejoin};
pub use serve::{Answer, ReplyTo, serve_http, serve_udp};
pub use transport_error::TransportError;
pub use wifi_credentials::WifiCredentials;
