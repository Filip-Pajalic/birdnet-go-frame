//! The frame's settings, built into the firmware from environment variables.

use frame_standalone::{SettingsError, StandaloneSettings};

/// Everything the firmware was built with.
pub struct Settings {
    /// Where BirdNET-Go is and how the page looks.
    pub frame: StandaloneSettings,
    /// The Wi-Fi's name (`FRAME_WIFI_SSID`).
    pub ssid: &'static str,
    /// The Wi-Fi's password (`FRAME_WIFI_PASSWORD`), empty for an open network.
    pub password: &'static str,
}

/// Returns the settings given at build time: `FRAME_WIFI_SSID`, `FRAME_WIFI_PASSWORD`,
/// `FRAME_BIRDNET` and the optional ones [`StandaloneSettings::from_lookup`] reads.
///
/// # Errors
///
/// Returns [`SettingsError`] naming the first variable that is missing or malformed.
pub fn load() -> Result<Settings, SettingsError> {
    let ssid: &'static str = option_env!("FRAME_WIFI_SSID")
        .filter(|ssid| !ssid.trim().is_empty())
        .ok_or(SettingsError {
            variable: "FRAME_WIFI_SSID",
        })?;
    Ok(Settings {
        frame: StandaloneSettings::from_lookup(built_in)?,
        ssid,
        password: option_env!("FRAME_WIFI_PASSWORD").unwrap_or(""),
    })
}

/// The value `variable` had when the firmware was built. `option_env!` takes only literals,
/// hence the list.
fn built_in(variable: &str) -> Option<&'static str> {
    match variable {
        "FRAME_BIRDNET" => option_env!("FRAME_BIRDNET"),
        "FRAME_ROTATION" => option_env!("FRAME_ROTATION"),
        "FRAME_LAYOUT" => option_env!("FRAME_LAYOUT"),
        "FRAME_LANGUAGE" => option_env!("FRAME_LANGUAGE"),
        "FRAME_UTC_OFFSET_MINUTES" => option_env!("FRAME_UTC_OFFSET_MINUTES"),
        "FRAME_POLL_MINUTES" => option_env!("FRAME_POLL_MINUTES"),
        "FRAME_WINDOW_HOURS" => option_env!("FRAME_WINDOW_HOURS"),
        "FRAME_MAX_BIRDS" => option_env!("FRAME_MAX_BIRDS"),
        "FRAME_NAMES" => option_env!("FRAME_NAMES"),
        _ => None,
    }
}
