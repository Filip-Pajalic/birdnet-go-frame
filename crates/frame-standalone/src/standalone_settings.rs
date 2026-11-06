//! A standalone frame's settings, read from text such as build-time environment variables.

use alloc::string::String;
use core::fmt;
use core::str::FromStr;

use frame_artist::Layout;
use frame_core::Rotation;
use panel_wake::FrameAddress;

/// How a standalone frame finds BirdNET-Go and sets out its page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandaloneSettings {
    /// BirdNET-Go's IPv4 address and port (`FRAME_BIRDNET`).
    pub birdnet: FrameAddress,
    /// How the panel hangs (`FRAME_ROTATION`, default 0).
    pub rotation: Rotation,
    /// Which page to draw (`FRAME_LAYOUT`, default `collage`).
    pub layout: Layout,
    /// Language code for the page's own words (`FRAME_LANGUAGE`, default `en`).
    pub language: String,
    /// The garden's time zone, minutes east of UTC (`FRAME_UTC_OFFSET_MINUTES`, default 0).
    pub utc_offset_minutes: i32,
    /// Minutes between wakes (`FRAME_POLL_MINUTES`, default 15).
    pub poll_minutes: u16,
    /// How far back detections count (`FRAME_WINDOW_HOURS`, default 24).
    pub window_hours: u32,
    /// Most birds on a page (`FRAME_MAX_BIRDS`, default 12; memory sets the limit).
    pub max_birds: usize,
    /// Whether the collage names its birds (`FRAME_NAMES`, default true).
    pub names: bool,
}

/// A setting that could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct SettingsError {
    /// The variable at fault.
    pub variable: &'static str,
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} is missing or cannot be read", self.variable)
    }
}

impl core::error::Error for SettingsError {}

impl StandaloneSettings {
    /// Most birds on a page unless `FRAME_MAX_BIRDS` says otherwise: each bird's picture is held
    /// while the page is drawn, and an ESP32 has 8 MB.
    pub const MAX_BIRDS: usize = 12;

    /// Reads the settings through `lookup`, which returns a variable's value if set. An empty
    /// value counts as unset.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] naming the first variable that is missing (only
    /// `FRAME_BIRDNET` must be set) or malformed.
    pub fn from_lookup<'a>(
        lookup: impl Fn(&str) -> Option<&'a str>,
    ) -> Result<Self, SettingsError> {
        let get = |variable: &str| {
            lookup(variable)
                .map(str::trim)
                .filter(|text| !text.is_empty())
        };
        let birdnet: FrameAddress = get("FRAME_BIRDNET")
            .and_then(|text| text.parse().ok())
            .ok_or(SettingsError {
                variable: "FRAME_BIRDNET",
            })?;
        Ok(Self {
            birdnet,
            rotation: parsed(&get, "FRAME_ROTATION", Rotation::Degrees0)?,
            layout: parsed(&get, "FRAME_LAYOUT", Layout::Collage)?,
            language: String::from(get("FRAME_LANGUAGE").unwrap_or("en")),
            utc_offset_minutes: parsed(&get, "FRAME_UTC_OFFSET_MINUTES", 0)?,
            poll_minutes: parsed(&get, "FRAME_POLL_MINUTES", 15)?,
            window_hours: parsed(&get, "FRAME_WINDOW_HOURS", 24)?,
            max_birds: parsed(&get, "FRAME_MAX_BIRDS", Self::MAX_BIRDS)?,
            names: parsed(&get, "FRAME_NAMES", true)?,
        })
    }
}

fn parsed<'a, T: FromStr>(
    get: &impl Fn(&str) -> Option<&'a str>,
    variable: &'static str,
    default: T,
) -> Result<T, SettingsError> {
    match get(variable) {
        None => Ok(default),
        Some(text) => text.parse().map_err(|_| SettingsError { variable }),
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use super::*;

    fn settings(pairs: &[(&str, &'static str)]) -> Result<StandaloneSettings, SettingsError> {
        StandaloneSettings::from_lookup(|name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| *value)
        })
    }

    #[test]
    fn needs_only_birdnet() {
        let read: StandaloneSettings =
            settings(&[("FRAME_BIRDNET", "192.168.1.10:8080"), ("FRAME_LAYOUT", "")])
                .expect("valid");
        assert_eq!(read.birdnet.to_string(), "192.168.1.10:8080");
        assert_eq!(read.layout, Layout::Collage);
        assert_eq!(
            (read.poll_minutes, read.window_hours, read.max_birds),
            (15, 24, 12)
        );
        assert!(read.names);
        assert_eq!(read.language, "en");
    }

    #[test]
    fn names_what_it_cannot_read() {
        assert_eq!(
            settings(&[]),
            Err(SettingsError {
                variable: "FRAME_BIRDNET"
            })
        );
        assert_eq!(
            settings(&[("FRAME_BIRDNET", "birdnet.local:8080")]),
            Err(SettingsError {
                variable: "FRAME_BIRDNET"
            })
        );
        let bad = settings(&[("FRAME_BIRDNET", "10.0.0.2"), ("FRAME_ROTATION", "45")]);
        assert_eq!(
            bad,
            Err(SettingsError {
                variable: "FRAME_ROTATION"
            })
        );
        let list: StandaloneSettings = settings(&[
            ("FRAME_BIRDNET", "10.0.0.2"),
            ("FRAME_LAYOUT", "list"),
            ("FRAME_UTC_OFFSET_MINUTES", "-300"),
        ])
        .expect("valid");
        assert_eq!((list.layout, list.utc_offset_minutes), (Layout::List, -300));
    }
}
