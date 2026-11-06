//! Drawing the page on the chip.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use art_mock::MockArt;
use art_pack::{ArtBundle, ArtSource};
use birdnet_model::SpeciesSummary;
use esp_println::println;
use frame_artist::{Artist, PageStyle};
use frame_core::{PageSizeError, PanelSize, Resolution};
use frame_raster::{Canvas, FontError, Renderer};
use frame_standalone::StandaloneSettings;
use page_text::Translations;

/// The page's typeface, as the server's.
const FONT: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");
/// The art bundle `FRAME_ART` named at build time, or nothing (see `build.rs`).
const ART: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/art.artb"));
/// Seconds in a day.
const SECONDS_PER_DAY: i64 = 86_400;
/// The side of a generated bird, in pixels, when no art was bundled.
const MOCK_SIDE: usize = 200;

/// Why no page was drawn.
#[derive(Debug)]
pub enum DrawError {
    /// The font did not load.
    Font(FontError),
    /// The page came out a size the panel does not take.
    Size(PageSizeError),
}

impl fmt::Display for DrawError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Font(error) => write!(formatter, "the font: {error}"),
            Self::Size(error) => write!(formatter, "the page: {error}"),
        }
    }
}

/// Returns the day (since 1970) in the garden's time zone at unix time `now`.
pub fn local_day(now: i64, utc_offset_minutes: i32) -> i64 {
    (now + i64::from(utc_offset_minutes) * 60).div_euclid(SECONDS_PER_DAY)
}

/// Draws `species`, heard by unix time `now`, as the panel's bytes for `panel`.
///
/// # Errors
///
/// Returns [`DrawError`] when the font does not load or the page does not fit the panel.
pub fn page(
    settings: &StandaloneSettings,
    panel: PanelSize,
    now: i64,
    species: &[SpeciesSummary],
) -> Result<Vec<u8>, DrawError> {
    let renderer: Renderer = Renderer::new(FONT).map_err(DrawError::Font)?;
    let style = PageStyle {
        layout: settings.layout,
        max_birds: settings.max_birds,
        names: settings.names,
        heading: None,
        words: Translations::bundled().page_text(&settings.language),
    };
    let mut artist: Artist<Box<dyn ArtSource>> = Artist::new(renderer, art(), style);
    artist.start_day(local_day(now, settings.utc_offset_minutes));
    let footer: String = artist
        .words()
        .updated(&clock(now, settings.utc_offset_minutes));
    let size: Resolution = frame_standalone::drawing_resolution(panel, settings.rotation);
    let page: Canvas = artist.draw(species, &footer, size);
    frame_artist::panel_bytes(page, artist.paper(), panel, settings.rotation)
        .map_err(DrawError::Size)
}

/// The bundled art, or generated birds when the build had none or it does not read.
fn art() -> Box<dyn ArtSource> {
    if ART.is_empty() {
        println!("standalone: no art bundled; drawing generated birds");
        return Box::new(MockArt::new(MOCK_SIDE));
    }
    match ArtBundle::parse(ART) {
        Ok(bundle) => {
            println!("standalone: {} species in the art bundle", bundle.len());
            Box::new(bundle)
        }
        Err(error) => {
            println!("standalone: the art bundle: {error}; drawing generated birds");
            Box::new(MockArt::new(MOCK_SIDE))
        }
    }
}

/// Returns the garden's time of day at unix time `now`, e.g. `07:05`.
fn clock(now: i64, utc_offset_minutes: i32) -> String {
    let local: i64 = now + i64::from(utc_offset_minutes) * 60;
    let minutes_today: i64 = local.rem_euclid(SECONDS_PER_DAY) / 60;
    format!("{:02}:{:02}", minutes_today / 60, minutes_today % 60)
}
