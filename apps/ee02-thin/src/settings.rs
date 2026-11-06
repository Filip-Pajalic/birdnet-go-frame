//! The firmware's configuration, fixed when it is built.

use frame_core::Rotation;
use panel_wake::{FrameAddress, ParseFrameAddressError};
use transport_esp::WifiCredentials;

/// The Wi-Fi network, from `FRAME_WIFI_SSID` and `FRAME_WIFI_PASSWORD` at build time.
pub const WIFI: WifiCredentials<'static> = WifiCredentials {
    ssid: env!("FRAME_WIFI_SSID"),
    password: env!("FRAME_WIFI_PASSWORD"),
};

/// The frame server, from `FRAME_SERVER` at build time, e.g. `192.168.1.20:8081`.
const SERVER: &str = env!("FRAME_SERVER");

/// The page's rotation, from `FRAME_ROTATION` at build time; without it the server's
/// `ROTATION` decides.
const ROTATION: Option<&str> = option_env!("FRAME_ROTATION");

/// Returns the frame server's address.
///
/// # Errors
///
/// Returns [`ParseFrameAddressError`] when `FRAME_SERVER` is not an IPv4 address and port.
pub fn frame() -> Result<FrameAddress, ParseFrameAddressError> {
    SERVER.parse()
}

/// Returns the rotation to ask for, or `None` to leave it to the server. A value that is not
/// 0, 90, 180 or 270 is reported and left to the server.
pub fn rotation() -> Option<Rotation> {
    let text: &str = ROTATION.filter(|text| !text.trim().is_empty())?;
    match text.parse() {
        Ok(rotation) => Some(rotation),
        Err(error) => {
            esp_println::println!("ee02: FRAME_ROTATION {text:?}: {error}; the server decides");
            None
        }
    }
}
