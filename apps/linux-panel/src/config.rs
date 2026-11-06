//! Settings, read from environment variables. The page settings have the server's names.

use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fmt};

use frame_artist::{Border, Layout};
use frame_core::{PanelSize, Rotation};

/// Where detections come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A real BirdNET-Go at this base URL.
    Live(String),
    /// The built-in simulated station (`BIRDNET_URL=mock`).
    Mock,
}

/// Where pages go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    /// The panel, on these pins.
    Glass(Wiring),
    /// A file that each new page's panel bytes replace (`PANEL_OUT`), for trying it out with
    /// no panel attached.
    File(PathBuf),
}

/// How the panel is wired to the Pi: line numbers on one GPIO chip, and the SPI device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wiring {
    /// The SPI device, `/dev/spidev0.0` by default.
    pub spi_device: PathBuf,
    /// The bus clock in hertz.
    pub spi_hz: u32,
    /// The GPIO chip, `/dev/gpiochip0` by default.
    pub gpio_chip: PathBuf,
    /// Chip select of the panel's (first) controller.
    pub cs0: u32,
    /// Chip select of the 13.3" panel's second controller.
    pub cs1: u32,
    /// Data/command.
    pub dc: u32,
    /// Reset.
    pub rst: u32,
    /// BUSY.
    pub busy: u32,
    /// The panel's power switch, if the board has one.
    pub power: Option<u32>,
}

/// Every setting. Not `Debug`, so the password never reaches a log.
pub struct Config {
    /// BirdNET-Go, or the simulated station.
    pub source: Source,
    /// BirdNET-Go's password, for its private mode, with the user name.
    pub login: Option<(String, String)>,
    /// Which panel is attached.
    pub panel: PanelSize,
    /// How the panel hangs.
    pub rotation: Rotation,
    /// Minutes between asking BirdNET-Go for news; the panel refreshes at most this often.
    pub poll_minutes: u16,
    /// How far back detections count.
    pub window_hours: u32,
    /// The garden's time zone, minutes east of UTC.
    pub utc_offset_minutes: i32,
    /// A heading to print over the birds.
    pub heading: Option<String>,
    /// Which page to draw.
    pub layout: Layout,
    /// Most birds on a page; 0 for every bird heard.
    pub max_birds: usize,
    /// Whether the collage writes each bird's name under it.
    pub names: bool,
    /// Whether the page ends with a line about its main bird.
    pub facts: bool,
    /// The border round the page: `season`, `off` or a theme.
    pub border: Border,
    /// Language code for the page's own words, e.g. `sv`.
    pub language: String,
    /// A translations file adding to or overriding the bundled one.
    pub translations: Option<PathBuf>,
    /// A fugleramme-style art folder; `None` has no pictures, so collages are drawn as lists.
    pub art_dir: Option<PathBuf>,
    /// Where pages go.
    pub output: Output,
}

/// A setting that could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
pub struct ConfigError {
    /// The variable at fault.
    pub variable: &'static str,
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
        let birdnet_url: String = lookup("BIRDNET_URL").ok_or_else(|| ConfigError {
            variable: "BIRDNET_URL",
            problem: String::from("not set; use your BirdNET-Go address, or `mock` to try it out"),
        })?;
        let source: Source = if birdnet_url.trim() == "mock" {
            Source::Mock
        } else {
            Source::Live(birdnet_url)
        };
        let panel: PanelSize = parsed(&lookup, "PANEL", PanelSize::Inches13Point3)?;
        let output: Output = match path(&lookup, "PANEL_OUT") {
            Some(file) => Output::File(file),
            None => Output::Glass(wiring(&lookup, panel)?),
        };
        Ok(Self {
            source,
            login: lookup("BIRDNET_PASSWORD")
                .filter(|password| !password.is_empty())
                .map(|password| {
                    let username: String = lookup("BIRDNET_USERNAME")
                        .filter(|name| !name.trim().is_empty())
                        .unwrap_or_else(|| String::from("admin"));
                    (username, password)
                }),
            panel,
            rotation: parsed(&lookup, "ROTATION", Rotation::Degrees0)?,
            poll_minutes: parsed(&lookup, "POLL_MINUTES", 15)?,
            window_hours: parsed(&lookup, "WINDOW_HOURS", 24)?,
            utc_offset_minutes: parsed(&lookup, "UTC_OFFSET_MINUTES", 0)?,
            heading: lookup("HEADING").filter(|heading| !heading.trim().is_empty()),
            layout: parsed(&lookup, "LAYOUT", Layout::Collage)?,
            max_birds: parsed(&lookup, "MAX_BIRDS", 0)?,
            names: parsed(&lookup, "NAMES", true)?,
            facts: parsed(&lookup, "FACTS", true)?,
            border: parsed(&lookup, "BORDER", Border::Season)?,
            language: lookup("LANGUAGE").unwrap_or_else(|| String::from("en")),
            translations: path(&lookup, "TRANSLATIONS"),
            art_dir: path(&lookup, "ART_DIR"),
            output,
        })
    }
}

/// Reads the wiring. The defaults are the pins of Waveshare's e-Paper HAT+ boards, by BCM
/// number on `/dev/gpiochip0`: RST 17, DC 25, CS 8 (and 7 for the 13.3" panel's second
/// controller), BUSY 24 and power 18.
fn wiring(
    lookup: &impl Fn(&str) -> Option<String>,
    panel: PanelSize,
) -> Result<Wiring, ConfigError> {
    let default_hz: u32 = match panel {
        PanelSize::Inches13Point3 => 10_000_000,
        PanelSize::Inches7Point3 => 4_000_000,
    };
    let power: Option<u32> = match lookup("PIN_POWER").map(|text| text.trim().to_owned()) {
        Some(text) if text == "none" || text.is_empty() => None,
        Some(_) => Some(parsed(lookup, "PIN_POWER", 18)?),
        None => Some(18),
    };
    Ok(Wiring {
        spi_device: path(lookup, "SPI_DEVICE").unwrap_or_else(|| PathBuf::from("/dev/spidev0.0")),
        spi_hz: parsed(lookup, "SPI_HZ", default_hz)?,
        gpio_chip: path(lookup, "GPIO_CHIP").unwrap_or_else(|| PathBuf::from("/dev/gpiochip0")),
        cs0: parsed(lookup, "PIN_CS0", 8)?,
        cs1: parsed(lookup, "PIN_CS1", 7)?,
        dc: parsed(lookup, "PIN_DC", 25)?,
        rst: parsed(lookup, "PIN_RST", 17)?,
        busy: parsed(lookup, "PIN_BUSY", 24)?,
        power,
    })
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
    fn defaults_drive_a_waveshare_hat() {
        let config: Config = config(&[("BIRDNET_URL", "http://birdnet:8080")]).expect("valid");
        assert_eq!(
            config.source,
            Source::Live(String::from("http://birdnet:8080"))
        );
        assert_eq!(config.panel, PanelSize::Inches13Point3);
        assert_eq!(config.poll_minutes, 15);
        assert_eq!(config.layout, Layout::Collage);
        let Output::Glass(wiring) = config.output else {
            panic!("drives the panel by default");
        };
        assert_eq!(wiring.spi_device, PathBuf::from("/dev/spidev0.0"));
        assert_eq!(wiring.spi_hz, 10_000_000);
        assert_eq!(
            (
                wiring.cs0,
                wiring.cs1,
                wiring.dc,
                wiring.rst,
                wiring.busy,
                wiring.power
            ),
            (8, 7, 25, 17, 24, Some(18))
        );
    }

    #[test]
    fn small_panel_file_output_and_no_power_pin() {
        let small: Config = config(&[
            ("BIRDNET_URL", "mock"),
            ("PANEL", "7.3"),
            ("PIN_POWER", "none"),
        ])
        .expect("valid");
        let Output::Glass(wiring) = small.output else {
            panic!("drives the panel");
        };
        assert_eq!((wiring.spi_hz, wiring.power), (4_000_000, None));
        let file: Config =
            config(&[("BIRDNET_URL", "mock"), ("PANEL_OUT", "/tmp/page.bin")]).expect("valid");
        assert_eq!(file.output, Output::File(PathBuf::from("/tmp/page.bin")));
        assert_eq!(file.source, Source::Mock);
    }

    #[test]
    fn bad_values_are_named() {
        assert_eq!(
            config(&[]).err().map(|error| error.variable),
            Some("BIRDNET_URL")
        );
        let bad = config(&[("BIRDNET_URL", "mock"), ("PIN_BUSY", "x")]);
        assert_eq!(bad.err().map(|error| error.variable), Some("PIN_BUSY"));
        let private: Config =
            config(&[("BIRDNET_URL", "mock"), ("BIRDNET_PASSWORD", "pw")]).expect("valid");
        assert_eq!(
            private.login,
            Some((String::from("admin"), String::from("pw")))
        );
    }
}
