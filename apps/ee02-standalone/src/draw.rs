//! Drawing the page on the chip.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use art_pack::{ArtBundle, ArtSource, NoArt};
use esp_println::println;
use frame_artist::{Artist, PageStyle, ViewData};
use frame_core::{PageSizeError, PanelSize, Resolution};
use frame_raster::{Canvas, FontError, Renderer};
use frame_standalone::{StandaloneSettings, ViewFacts, ViewFetch};
use page_text::Translations;

/// The page's typeface, as the server's.
const FONT: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");
/// The art bundle `FRAME_ART` named at build time, or nothing (see `build.rs`).
const ART: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/art.artb"));
/// Seconds in a day.
const SECONDS_PER_DAY: i64 = 86_400;

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

/// Draws the page `fetched` holds, as BirdNET-Go told it at unix time `now`, as the panel's
/// bytes for `panel`.
///
/// # Errors
///
/// Returns [`DrawError`] when the font does not load or the page does not fit the panel.
pub fn page(
    settings: &StandaloneSettings,
    panel: PanelSize,
    now: i64,
    fetched: &ViewFetch,
) -> Result<Vec<u8>, DrawError> {
    let renderer: Renderer = Renderer::new(FONT).map_err(DrawError::Font)?;
    let style = PageStyle {
        layout: settings.layout,
        max_birds: settings.max_birds,
        names: settings.names,
        heading: None,
        words: Translations::bundled().page_text(&settings.language),
        facts: settings.facts,
        border: settings.border,
        southern: false,
    };
    let mut artist: Artist<Box<dyn ArtSource>> = Artist::new(renderer, art(), style);
    artist.start_day(local_day(now, settings.utc_offset_minutes));
    let footer: String = artist
        .words()
        .updated(&clock(now, settings.utc_offset_minutes));
    let size: Resolution = frame_standalone::drawing_resolution(panel, settings.rotation);
    let facts: ViewFacts = fetched.facts();
    let data: ViewData<'_> = fetched.data(&facts, &footer);
    let page: Canvas = artist.draw_view(fetched.view(), &data, size);
    frame_artist::panel_bytes(page, artist.paper(), panel, settings.rotation)
        .map_err(DrawError::Size)
}

/// The bundled art, or none when the build had none or it does not read: then collages are
/// drawn as lists.
fn art() -> Box<dyn ArtSource> {
    if ART.is_empty() {
        println!("standalone: no art bundled; drawing lists");
        return Box::new(NoArt);
    }
    match ArtBundle::parse(ART) {
        Ok(bundle) => {
            println!("standalone: {} species in the art bundle", bundle.len());
            Box::new(bundle)
        }
        Err(error) => {
            println!("standalone: the art bundle: {error}; drawing lists");
            Box::new(NoArt)
        }
    }
}

/// Returns the garden's time of day at unix time `now`, e.g. `07:05`.
fn clock(now: i64, utc_offset_minutes: i32) -> String {
    let local: i64 = now + i64::from(utc_offset_minutes) * 60;
    let minutes_today: i64 = local.rem_euclid(SECONDS_PER_DAY) / 60;
    format!("{:02}:{:02}", minutes_today / 60, minutes_today % 60)
}
