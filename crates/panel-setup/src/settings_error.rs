//! What can be wrong with the settings a person enters.

use core::fmt;

/// Why the setup page's settings cannot be saved. The message is shown on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum SettingsError {
    /// The Wi-Fi name is empty or longer than 32 bytes.
    Ssid,
    /// The password is neither empty nor 8 to 63 characters.
    Password,
    /// The server is not an IPv4 address with an optional port.
    Server,
    /// The rotation is not 0, 90, 180 or 270.
    Rotation,
    /// The frame name has something other than lower-case letters, digits and `-`, or is
    /// longer than 32.
    FrameName,
    /// The form was not encoded as a browser encodes it.
    Encoding,
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Ssid => "The Wi-Fi name must be 1 to 32 characters.",
            Self::Password => {
                "The Wi-Fi password must be 8 to 63 characters, or tick \"open network\"."
            }
            Self::Server => "The server must be an address like 192.168.1.20:8080.",
            Self::Rotation => "The rotation must be 0, 90, 180 or 270.",
            Self::FrameName => {
                "The frame name may use only a-z, 0-9 and -, and at most 32 of them."
            }
            Self::Encoding => "The form arrived garbled; please try again.",
        })
    }
}

impl core::error::Error for SettingsError {}
