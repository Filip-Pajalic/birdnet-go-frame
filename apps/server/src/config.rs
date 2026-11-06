//! Settings, read from environment variables.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::{env, fmt};

use birdnet_client::CivilDate;
use frame_artist::{Border, Layout, PageView};
use frame_core::{PanelSize, Rotation};

use crate::frames::FrameSettings;
use crate::quiet_hours::QuietHours;

/// Where the garden is, for its weather.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    /// Degrees north.
    pub latitude: f32,
    /// Degrees east.
    pub longitude: f32,
}

/// An MQTT broker to announce frames to Home Assistant on. The password never shows in debug
/// output.
#[derive(Clone, PartialEq, Eq)]
pub struct Mqtt {
    /// `host:port`.
    pub address: String,
    /// The user name and password, if the broker asks for them.
    pub login: Option<(String, String)>,
    /// Home Assistant's discovery prefix, `homeassistant` unless changed.
    pub discovery_prefix: String,
}

impl fmt::Debug for Mqtt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Mqtt")
            .field("address", &self.address)
            .field("user", &self.login.as_ref().map(|(user, _)| user))
            .field("discovery_prefix", &self.discovery_prefix)
            .finish_non_exhaustive()
    }
}

impl FromStr for Mqtt {
    type Err = String;

    /// Reads `mqtt://user:password@host:port`; the login and port (1883) may be left out.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let rest: &str = text
            .trim()
            .strip_prefix("mqtt://")
            .ok_or_else(|| String::from("expected mqtt://host:port"))?;
        let rest: &str = rest.trim_end_matches('/');
        let (login, host): (Option<(String, String)>, &str) = match rest.rsplit_once('@') {
            Some((login, host)) => {
                let (user, password) = login.split_once(':').unwrap_or((login, ""));
                (Some((String::from(user), String::from(password))), host)
            }
            None => (None, rest),
        };
        if host.is_empty() {
            return Err(String::from("no host"));
        }
        let address: String = if host
            .rsplit_once(':')
            .is_some_and(|(_, port)| port.parse::<u16>().is_ok())
        {
            String::from(host)
        } else {
            format!("{host}:1883")
        };
        Ok(Self {
            address,
            login,
            discovery_prefix: String::from("homeassistant"),
        })
    }
}

/// Where detections come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A real BirdNET-Go at this base URL.
    Live(String),
    /// The built-in simulated station (`BIRDNET_URL=mock`).
    Mock,
}

/// BirdNET-Go's login, for its private mode. The password never shows in debug output.
#[derive(Clone, PartialEq, Eq)]
pub struct Login {
    /// The user name; BirdNET-Go's password login takes any.
    pub username: String,
    /// The password.
    pub password: String,
}

impl fmt::Debug for Login {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Login")
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// Server settings.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Detection source.
    pub source: Source,
    /// BirdNET-Go's login, when its private mode is on.
    pub login: Option<Login>,
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
    /// Whether the bird pages end with a line about their main bird (`FACTS`).
    pub facts: bool,
    /// The border round the pages (`BORDER`): `season`, `off` or a theme.
    pub border: Border,
    /// Language code for the page's own words, e.g. `sv`.
    pub language: String,
    /// A translations file adding to or overriding the bundled one.
    pub translations: Option<PathBuf>,
    /// A fugleramme-style art folder; `None` has no pictures, so collages are drawn as lists.
    pub art_dir: Option<PathBuf>,
    /// The page shown to frames that do not ask for one.
    pub view: PageView,
    /// Frames with settings of their own, chosen by `frame=`.
    pub frames: Vec<FrameSettings>,
    /// Where the garden is, for the weather on the story page; `None` for no weather.
    pub place: Option<Place>,
    /// Where Open-Meteo is.
    pub weather_url: String,
    /// Minutes the panel sleeps while a bird new for the year is about; 0 to keep
    /// [`poll_minutes`](Self::poll_minutes).
    pub alert_poll_minutes: u8,
    /// How long after a new bird was last heard the shorter sleep lasts, in minutes.
    pub alert_minutes: u32,
    /// How often the year, week and story are asked for again, besides when the birds change.
    pub almanac_refresh: Duration,
    /// Where to announce frames to Home Assistant.
    pub mqtt: Option<Mqtt>,
    /// A folder of firmware to offer frames.
    pub firmware_dir: Option<PathBuf>,
    /// The day the simulated station pretends it is (`MOCK_DATE`), for demos of other
    /// seasons; `None` for today.
    pub mock_date: Option<CivilDate>,
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
        let mut config = Self {
            source,
            login: lookup("BIRDNET_PASSWORD")
                .filter(|password| !password.is_empty())
                .map(|password| Login {
                    username: lookup("BIRDNET_USERNAME")
                        .filter(|name| !name.trim().is_empty())
                        .unwrap_or_else(|| String::from("admin")),
                    password,
                }),
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
            facts: parsed(&lookup, "FACTS", true)?,
            border: parsed(&lookup, "BORDER", Border::Season)?,
            language: lookup("LANGUAGE").unwrap_or_else(|| String::from("en")),
            translations: path(&lookup, "TRANSLATIONS"),
            art_dir: path(&lookup, "ART_DIR"),
            view: parsed(&lookup, "VIEW", PageView::Today)?,
            frames: Vec::new(),
            place: place(&lookup)?,
            weather_url: lookup("WEATHER_URL")
                .filter(|url| !url.trim().is_empty())
                .unwrap_or_else(|| String::from("http://api.open-meteo.com")),
            alert_poll_minutes: parsed(&lookup, "ALERT_POLL_MINUTES", 5)?,
            alert_minutes: parsed(&lookup, "ALERT_MINUTES", 60)?,
            almanac_refresh: Duration::from_secs(60 * parsed(&lookup, "ALMANAC_MINUTES", 30)?),
            mqtt: optional(&lookup, "MQTT_URL")?,
            firmware_dir: path(&lookup, "FIRMWARE_DIR"),
            mock_date: match lookup("MOCK_DATE").filter(|date| !date.trim().is_empty()) {
                None => None,
                Some(text) => Some(CivilDate::parse(text.trim()).ok_or(ConfigError {
                    variable: "MOCK_DATE",
                    problem: format!("cannot read {text:?}: expected YYYY-MM-DD"),
                })?),
            },
        };
        if let Some(mqtt) = &mut config.mqtt {
            if let Some(prefix) =
                lookup("MQTT_DISCOVERY_PREFIX").filter(|prefix| !prefix.trim().is_empty())
            {
                mqtt.discovery_prefix = String::from(prefix.trim());
            }
        }
        if let Some(text) = lookup("FRAMES") {
            config.frames =
                FrameSettings::parse_all(&text, &config.default_frame()).map_err(|error| {
                    ConfigError {
                        variable: "FRAMES",
                        problem: error.to_string(),
                    }
                })?;
        }
        Ok(config)
    }

    /// Returns the settings of frames that do not give a name, or give one not in `FRAMES`.
    #[must_use]
    pub fn default_frame(&self) -> FrameSettings {
        FrameSettings {
            name: String::from(frame_http::DEFAULT_FRAME),
            layout: self.layout,
            view: self.view,
            panel: self.panel,
            rotation: self.rotation,
            max_birds: self.max_birds,
            names: self.names,
            heading: self.heading.clone(),
            facts: self.facts,
            border: self.border,
        }
    }

    /// Returns the settings of the frame called `name`, or the default frame's.
    #[must_use]
    pub fn frame(&self, name: &str) -> FrameSettings {
        self.frames
            .iter()
            .find(|frame| frame.name == name)
            .cloned()
            .unwrap_or_else(|| self.default_frame())
    }
}

/// Reads `LATITUDE` and `LONGITUDE`, which go together.
fn place(lookup: &impl Fn(&str) -> Option<String>) -> Result<Option<Place>, ConfigError> {
    let latitude: Option<f32> = optional(lookup, "LATITUDE")?;
    let longitude: Option<f32> = optional(lookup, "LONGITUDE")?;
    match (latitude, longitude) {
        (Some(latitude), Some(longitude)) if latitude.abs() <= 90.0 && longitude.abs() <= 180.0 => {
            Ok(Some(Place {
                latitude,
                longitude,
            }))
        }
        (None, None) => Ok(None),
        _ => Err(ConfigError {
            variable: "LATITUDE",
            problem: String::from("set both LATITUDE and LONGITUDE, in degrees"),
        }),
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
        assert!(config.facts);
        assert_eq!(config.border, Border::Season);
        assert_eq!(config.language, "en");
        assert_eq!(config.art_dir, None);
    }

    #[test]
    fn login_for_private_mode() {
        let open = config(&[("BIRDNET_URL", "mock")]).expect("valid");
        assert_eq!(open.login, None);
        let private =
            config(&[("BIRDNET_URL", "mock"), ("BIRDNET_PASSWORD", "hunter2")]).expect("valid");
        let login: Login = private.login.expect("a login");
        assert_eq!(
            (login.username.as_str(), login.password.as_str()),
            ("admin", "hunter2")
        );
        assert!(
            !format!("{login:?}").contains("hunter2"),
            "kept out of logs"
        );
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
            ("FACTS", "false"),
            ("BORDER", "frost"),
        ])
        .expect("valid");
        assert!(!config.facts);
        assert_eq!(config.border.to_string(), "frost");
        assert_eq!(config.art_dir, Some(PathBuf::from("/art/classic")));
        assert_eq!(config.language, "sv");
        assert!(!config.names);
        assert_eq!(config.heading.as_deref(), Some("Fåglar"));
    }

    #[test]
    fn frames_views_weather_and_home_assistant() {
        let config = config(&[
            ("BIRDNET_URL", "mock"),
            ("LAYOUT", "list"),
            ("VIEW", "story"),
            ("FRAMES", "hall:layout=collage;desk:panel=7.3"),
            ("LATITUDE", "59.33"),
            ("LONGITUDE", "18.07"),
            ("MQTT_URL", "mqtt://frame:s3cret@broker.lan"),
        ])
        .expect("valid");
        assert_eq!(config.frame("hall").layout, Layout::Collage);
        assert_eq!(config.frame("desk").layout, Layout::List);
        assert_eq!(config.frame("desk").view, PageView::Story);
        assert_eq!(config.frame("attic").name, "default");
        assert_eq!(config.place.map(|place| place.latitude), Some(59.33));
        let mqtt: Mqtt = config.mqtt.expect("an MQTT broker");
        assert_eq!(mqtt.address, "broker.lan:1883");
        assert_eq!(
            mqtt.login,
            Some((String::from("frame"), String::from("s3cret")))
        );
        assert!(!format!("{mqtt:?}").contains("s3cret"), "kept out of logs");
        let half = config_error(&[("BIRDNET_URL", "mock"), ("LATITUDE", "59")]);
        assert_eq!(half, Some("LATITUDE"));
        assert_eq!(
            config_error(&[("BIRDNET_URL", "mock"), ("FRAMES", "Hall")]),
            Some("FRAMES")
        );
        assert_eq!(
            config_error(&[("BIRDNET_URL", "mock"), ("MQTT_URL", "http://x")]),
            Some("MQTT_URL")
        );
    }

    fn config_error(pairs: &[(&str, &str)]) -> Option<&'static str> {
        config(pairs).err().map(|error| error.variable)
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
