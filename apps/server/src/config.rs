//! Settings, read from environment variables.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::{env, fmt};

use frame_core::{PanelSize, Rotation};

use crate::quiet_hours::QuietHours;

/// Where detections come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A real BirdNET-Go at this base URL.
    Live(String),
    /// The built-in simulated station (`BIRDNET_URL=mock`).
    Mock,
}

/// Which page to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// A picture of every bird, heavier birds bigger (`LAYOUT=collage`, the default).
    Collage,
    /// Names and counts only (`LAYOUT=list`).
    List,
}

impl FromStr for Layout {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.to_ascii_lowercase().as_str() {
            "collage" => Ok(Self::Collage),
            "list" => Ok(Self::List),
            _ => Err(String::from("expected `collage` or `list`")),
        }
    }
}

/// Server settings.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Detection source.
    pub source: Source,
    /// Address to listen on.
    pub listen: String,
    /// The panel served to clients that do not name one.
    pub panel: PanelSize,
    /// How the panel hangs, for clients that do not say.
    pub rotation: Rotation,
    /// Minutes the panel sleeps between checks, sent as `Cache-Control: max-age` on `/panel`.
    pub poll_minutes: u8,
    /// Local hours the panel sleeps through in one go, e.g. 22 to 6.
    pub quiet_hours: Option<QuietHours>,
    /// How far back detections count.
    pub window_hours: u32,
    /// The garden's time zone, minutes east of UTC.
    pub utc_offset_minutes: i32,
    /// How often the server asks BirdNET-Go for news.
    pub refresh: Duration,
    /// A heading to print over the birds; `None` for none on the collage and the translated
    /// one on the list.
    pub heading: Option<String>,
    /// Which page to draw.
    pub layout: Layout,
    /// Most birds on a page; 0 for every bird heard.
    pub max_birds: usize,
    /// Whether the collage writes each bird's name under it.
    pub names: bool,
    /// Language code for the page's own words, e.g. `sv`.
    pub language: String,
    /// A translations file adding to or overriding the bundled one.
    pub translations: Option<PathBuf>,
    /// A fugleramme-style art folder; `None` draws generated birds.
    pub art_dir: Option<PathBuf>,
}

/// A setting that could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    variable: &'static str,
    problem: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.variable, self.problem)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Reads the settings from the process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] naming the first variable that is missing or malformed.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    /// Reads the settings through `lookup`, which returns a variable's value if set.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] naming the first variable that is missing or malformed.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let birdnet_url: String = lookup("BIRDNET_URL").ok_or(ConfigError {
            variable: "BIRDNET_URL",
            problem: String::from("not set; use your BirdNET-Go address, or `mock` to try it out"),
        })?;
        let source: Source = if birdnet_url.trim() == "mock" {
            Source::Mock
        } else {
            Source::Live(birdnet_url)
        };
        Ok(Self {
            source,
            listen: lookup("LISTEN").unwrap_or_else(|| String::from("0.0.0.0:8080")),
            panel: parsed(&lookup, "PANEL", PanelSize::Inches13Point3)?,
            rotation: parsed(&lookup, "ROTATION", Rotation::Degrees0)?,
            poll_minutes: parsed(&lookup, "POLL_MINUTES", 15)?,
            quiet_hours: optional(&lookup, "QUIET_HOURS")?,
            window_hours: parsed(&lookup, "WINDOW_HOURS", 24)?,
            utc_offset_minutes: parsed(&lookup, "UTC_OFFSET_MINUTES", 0)?,
            refresh: Duration::from_secs(parsed(&lookup, "REFRESH_SECONDS", 60)?),
            heading: lookup("HEADING").filter(|heading| !heading.trim().is_empty()),
            layout: parsed(&lookup, "LAYOUT", Layout::Collage)?,
            max_birds: parsed(&lookup, "MAX_BIRDS", 0)?,
            names: parsed(&lookup, "NAMES", true)?,
            language: lookup("LANGUAGE").unwrap_or_else(|| String::from("en")),
            translations: path(&lookup, "TRANSLATIONS"),
            art_dir: path(&lookup, "ART_DIR"),
        })
    }
}

fn path(lookup: &impl Fn(&str) -> Option<String>, variable: &str) -> Option<PathBuf> {
    lookup(variable)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn parsed<T: FromStr>(
    lookup: &impl Fn(&str) -> Option<String>,
    variable: &'static str,
    default: T,
) -> Result<T, ConfigError>
where
    T::Err: fmt::Display,
{
    match lookup(variable) {
        None => Ok(default),
        Some(text) => text.trim().parse().map_err(|error: T::Err| ConfigError {
            variable,
            problem: format!("cannot read {text:?}: {error}"),
        }),
    }
}

/// Like [`parsed`], for a setting that may be left out or empty.
fn optional<T: FromStr>(
    lookup: &impl Fn(&str) -> Option<String>,
    variable: &'static str,
) -> Result<Option<T>, ConfigError>
where
    T::Err: fmt::Display,
{
    match lookup(variable).filter(|value| !value.trim().is_empty()) {
        None => Ok(None),
        Some(text) => text
            .trim()
            .parse()
            .map(Some)
            .map_err(|error: T::Err| ConfigError {
                variable,
                problem: format!("cannot read {text:?}: {error}"),
            }),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    fn config(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        Config::from_lookup(|name| map.get(name).cloned())
    }

    #[test]
    fn defaults() {
        let config = config(&[("BIRDNET_URL", "http://birdnet:8080")]).expect("valid");
        assert_eq!(
            config.source,
            Source::Live(String::from("http://birdnet:8080"))
        );
        assert_eq!(config.poll_minutes, 15);
        assert_eq!(config.quiet_hours, None);
        assert_eq!(config.rotation, Rotation::Degrees0);
        assert_eq!(config.panel, PanelSize::Inches13Point3);
        assert_eq!(config.layout, Layout::Collage);
        assert_eq!(config.max_birds, 0);
        assert_eq!(config.heading, None);
        assert!(config.names);
        assert_eq!(config.language, "en");
        assert_eq!(config.art_dir, None);
    }

    #[test]
    fn quiet_hours() {
        let night = config(&[("BIRDNET_URL", "mock"), ("QUIET_HOURS", "22-6")]).expect("valid");
        assert_eq!(night.quiet_hours, "22-6".parse().ok());
        let empty = config(&[("BIRDNET_URL", "mock"), ("QUIET_HOURS", " ")]).expect("valid");
        assert_eq!(empty.quiet_hours, None);
        let bad = config(&[("BIRDNET_URL", "mock"), ("QUIET_HOURS", "night")]);
        assert_eq!(bad.map_err(|error| error.variable), Err("QUIET_HOURS"));
    }

    #[test]
    fn layout_choice() {
        let list = config(&[("BIRDNET_URL", "mock"), ("LAYOUT", "List")]).expect("valid");
        assert_eq!(list.layout, Layout::List);
        let bad = config(&[("BIRDNET_URL", "mock"), ("LAYOUT", "grid")]);
        assert_eq!(bad.map_err(|error| error.variable), Err("LAYOUT"));
    }

    #[test]
    fn art_language_and_names() {
        let config = config(&[
            ("BIRDNET_URL", "mock"),
            ("ART_DIR", "/art/classic"),
            ("LANGUAGE", "sv"),
            ("NAMES", "false"),
            ("HEADING", "Fåglar"),
        ])
        .expect("valid");
        assert_eq!(config.art_dir, Some(PathBuf::from("/art/classic")));
        assert_eq!(config.language, "sv");
        assert!(!config.names);
        assert_eq!(config.heading.as_deref(), Some("Fåglar"));
    }

    #[test]
    fn mock_source() {
        assert_eq!(
            config(&[("BIRDNET_URL", "mock")]).expect("valid").source,
            Source::Mock
        );
    }

    #[test]
    fn missing_url_and_bad_values_are_named() {
        assert_eq!(
            config(&[]).map_err(|error| error.variable),
            Err("BIRDNET_URL")
        );
        let bad = config(&[("BIRDNET_URL", "mock"), ("ROTATION", "45")]);
        assert_eq!(bad.map_err(|error| error.variable), Err("ROTATION"));
    }
}
