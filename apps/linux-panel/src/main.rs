//! birdnet-go-frame on a Raspberry Pi, with the panel on its SPI bus and no server.
//!
//! Every `POLL_MINUTES` it asks BirdNET-Go for the birds heard, and when they make a different
//! page it draws the page as the server would and shows it on the panel. Configured with the
//! server's environment variables, plus the panel's wiring; see
//! `docs/how-to/run-on-raspberry-pi.md`. `PANEL_OUT=file` writes each page's panel bytes to a
//! file instead, on any computer, and `--once` draws one page and exits.

mod board;
// Only the Linux board writes through it; its tests run everywhere.
#[cfg_attr(
    not(target_os = "linux"),
    allow(dead_code, reason = "only the Linux board uses it")
)]
mod chunked_bus;
mod config;

use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, thread};

use art_pack::{ArtSource, NoArt};
use art_webp::ArtFolder;
use birdnet_client::{BirdnetClient, ClientError, DateWindow, HttpTransport};
use birdnet_mock::MockTransport;
use birdnet_model::SpeciesSummary;
use frame_artist::{Artist, PageStyle};
use frame_core::{PanelSize, Resolution, panel_bytes};
use frame_raster::{Canvas, Renderer};
use frame_standalone::Fingerprint;
use page_text::{PageText, Translations};
use panel_wake::PageView;
use transport_lan::LanTransport;

use crate::board::Board;
use crate::config::{Config, Output, Source};

/// The face species names are set in: EB Garamond Italic, SIL OFL 1.1 (see assets/fonts).
const FONT: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");
const SECONDS_PER_DAY: i64 = 86_400;

/// What is on the glass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    /// Nothing drawn yet.
    Nothing,
    /// The page saying BirdNET-Go cannot be reached.
    Unreachable,
    /// The birds' page with this fingerprint.
    Birds(Fingerprint),
}

/// Where pages go: the panel, or a file.
enum Sink {
    Glass(Box<Board>),
    File(std::path::PathBuf),
}

impl Sink {
    /// Shows `page`, the panel's bytes.
    fn show(&mut self, page: &[u8]) -> Result<(), String> {
        match self {
            Self::Glass(board) => {
                let milliseconds: u32 = board.show(page)?;
                eprintln!("linux-panel: refreshed in {milliseconds} ms");
                Ok(())
            }
            Self::File(path) => {
                fs::write(&*path, page).map_err(|error| format!("{}: {error}", path.display()))
            }
        }
    }
}

fn main() -> ExitCode {
    let once: bool = env::args().skip(1).any(|arg| arg == "--once");
    match run(once) {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("linux-panel: {problem}");
            ExitCode::FAILURE
        }
    }
}

/// Sets everything up from the environment and draws pages, forever or `once`.
fn run(once: bool) -> Result<(), String> {
    let config: Config = Config::from_env().map_err(|error| error.to_string())?;
    let renderer: Renderer = Renderer::new(FONT).map_err(|error| error.to_string())?;
    let sink: Sink = match &config.output {
        Output::Glass(wiring) => Sink::Glass(Box::new(Board::open(wiring, config.panel)?)),
        Output::File(path) => Sink::File(path.clone()),
    };
    let art: Box<dyn ArtSource> = match &config.art_dir {
        Some(folder) => {
            let art: ArtFolder = ArtFolder::open(folder)
                .map_err(|error| format!("cannot read art in {}: {error}", folder.display()))?;
            eprintln!("linux-panel: {} plates in {}", art.len(), folder.display());
            Box::new(art.with_paper(renderer.paper()))
        }
        None => {
            eprintln!(
                "linux-panel: no ART_DIR; collages are drawn as lists until fugleramme's plates \
                 are fetched with scripts/fetch-art.sh"
            );
            Box::new(NoArt)
        }
    };
    let style = PageStyle {
        layout: config.layout,
        max_birds: config.max_birds,
        names: config.names,
        heading: config.heading.clone(),
        words: page_words(&config)?,
        facts: config.facts,
        border: config.border,
        southern: false,
    };
    let artist: Artist<Box<dyn ArtSource>> = Artist::new(renderer, art, style);
    let mut frame = Frame {
        artist,
        sink,
        config: &config,
        shown: Shown::Nothing,
    };
    match &config.source {
        Source::Live(url) => {
            let mut client = BirdnetClient::new(url, LanTransport::default());
            if let Some((username, password)) = &config.login {
                client = client.with_login(username, password);
            }
            frame.run(client, once)
        }
        Source::Mock => {
            let transport = MockTransport::with_utc_offset(now, config.utc_offset_minutes);
            frame.run(BirdnetClient::new("mock", transport), once)
        }
    }
}

/// The loop: the artist, where pages go, and what is on the glass.
struct Frame<'c> {
    artist: Artist<Box<dyn ArtSource>>,
    sink: Sink,
    config: &'c Config,
    shown: Shown,
}

impl Frame<'_> {
    /// Asks `client` for the birds every poll and shows each new page. With `once`, returns
    /// after the first page, or with the error when none could be drawn.
    fn run<T: HttpTransport>(
        &mut self,
        mut client: BirdnetClient<T>,
        once: bool,
    ) -> Result<(), String> {
        let poll: Duration = Duration::from_secs(u64::from(self.config.poll_minutes.max(1)) * 60);
        loop {
            let result: Result<(), String> = self.step(&mut client);
            if let Err(problem) = &result {
                eprintln!("linux-panel: {problem}");
            }
            if once {
                return result;
            }
            thread::sleep(poll);
        }
    }

    /// One poll: asks for the birds and draws them if the page would change.
    fn step<T: HttpTransport>(&mut self, client: &mut BirdnetClient<T>) -> Result<(), String> {
        let now: i64 = now();
        let config: &Config = self.config;
        let window = DateWindow::hours_back(now, config.utc_offset_minutes, config.window_hours);
        let species: Vec<SpeciesSummary> = match client.species_summary(window) {
            Ok(species) => species,
            Err(error) => return self.unreachable(&error),
        };
        let day: i64 = now.div_euclid(SECONDS_PER_DAY);
        let fingerprint: Fingerprint =
            Fingerprint::of(day, PageView::Today, &[&species], config.layout);
        if self.shown == Shown::Birds(fingerprint) {
            return Ok(());
        }
        self.artist.start_day(day);
        let footer: String = self
            .artist
            .words()
            .updated(&clock(now, config.utc_offset_minutes));
        let page: Canvas = self.artist.draw(&species, &footer, self.resolution());
        self.show(page)?;
        eprintln!("linux-panel: drew {} species", species.len());
        self.shown = Shown::Birds(fingerprint);
        Ok(())
    }

    /// Shows the page saying BirdNET-Go cannot be reached, unless something is on the glass
    /// already: a page of birds a little old beats no birds.
    fn unreachable<E: std::error::Error>(&mut self, error: &ClientError<E>) -> Result<(), String> {
        if self.shown == Shown::Nothing {
            let page: Canvas = self
                .artist
                .draw_unreachable(&error.to_string(), self.resolution());
            self.show(page)?;
            self.shown = Shown::Unreachable;
        }
        Err(error.to_string())
    }

    /// Turns `page` into the panel's bytes and shows it.
    fn show(&mut self, page: Canvas) -> Result<(), String> {
        let config: &Config = self.config;
        let bytes: Vec<u8> =
            frame_artist::panel_bytes(page, self.artist.paper(), config.panel, config.rotation)
                .map_err(|error| error.to_string())?;
        self.sink.show(&bytes)
    }

    /// The page's size as it hangs.
    fn resolution(&self) -> Resolution {
        let panel: PanelSize = self.config.panel;
        panel_bytes::page_resolution(panel, self.config.rotation)
    }
}

/// Returns the page's words in the configured language, from the bundled translations and any
/// file named by `TRANSLATIONS`.
fn page_words(config: &Config) -> Result<PageText, String> {
    let mut translations: Translations = Translations::bundled();
    if let Some(path) = &config.translations {
        let bytes: Vec<u8> =
            fs::read(path).map_err(|error| format!("TRANSLATIONS: {}: {error}", path.display()))?;
        let extra: Translations = Translations::parse(&bytes)
            .map_err(|error| format!("TRANSLATIONS: {}: {error}", path.display()))?;
        translations.merge(extra);
    }
    Ok(translations.page_text(&config.language))
}

/// Returns the current unix time in seconds.
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

/// Formats the local wall-clock time as `HH:MM`.
fn clock(now_unix_seconds: i64, utc_offset_minutes: i32) -> String {
    let local: i64 = now_unix_seconds + i64::from(utc_offset_minutes) * 60;
    let minutes_today: i64 = local.rem_euclid(SECONDS_PER_DAY) / 60;
    format!("{:02}:{:02}", minutes_today / 60, minutes_today % 60)
}
