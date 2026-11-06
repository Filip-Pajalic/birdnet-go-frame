//! Settings, read from environment variables.

use std::str::FromStr;
use std::time::Duration;
use std::{env, fmt};

use frame_core::Rotation;

/// Where detections come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A real BirdNET-Go at this base URL.
    Live(String),
    /// The built-in simulated station (`BIRDNET_URL=mock`).
    Mock,
}

/// Server settings.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Detection source.
    pub source: Source,
    /// Address to listen on.
    pub listen: String,
    /// How the panel hangs.
    pub rotation: Rotation,
    /// Minutes the panel sleeps between checks, sent in the page header.
    pub poll_minutes: u8,
    /// How far back detections count.
    pub window_hours: u32,
    /// The garden's time zone, minutes east of UTC.
    pub utc_offset_minutes: i32,
    /// How often the server asks BirdNET-Go for news.
    pub refresh: Duration,
    /// Page heading.
    pub heading: String,
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
            rotation: parsed(&lookup, "ROTATION", Rotation::Degrees0)?,
            poll_minutes: parsed(&lookup, "POLL_MINUTES", 15)?,
            window_hours: parsed(&lookup, "WINDOW_HOURS", 24)?,
            utc_offset_minutes: parsed(&lookup, "UTC_OFFSET_MINUTES", 0)?,
            refresh: Duration::from_secs(parsed(&lookup, "REFRESH_SECONDS", 60)?),
            heading: lookup("HEADING").unwrap_or_else(|| String::from("Heard in the garden")),
        })
    }
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
        assert_eq!(config.rotation, Rotation::Degrees0);
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
