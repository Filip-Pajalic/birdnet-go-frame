//! birdnet-go-frame's server.
//!
//! Reads BirdNET-Go's API, draws the birds heard onto pages for Spectra 6 panels (13.3" by
//! default, 7.3" on request) and screens of any size, and serves them at `/panel` (for the
//! ESP32), `/frame.png` and `/preview.png`. Each frame may have its own page (`frame=`) and
//! show the week, the rarest birds, the day's story or the year so far (`view=`). Configured with
//! environment variables; see the README. `BIRDNET_URL=mock` runs against a built-in
//! simulated station, so it works out of the box.

mod config;
mod frames;
mod home_assistant;
mod news;
mod poller;
mod quiet_hours;

use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

use art_pack::{ArtSource, NoArt};
use art_webp::ArtFolder;
use birdnet_client::{BirdnetClient, CivilDate, DateWindow};
use birdnet_mock::MockTransport;
use frame_artist::{Artist, PageStyle, PageView};
use frame_http::{FrameLog, FrameServer, PageStore, Target};
use frame_raster::Renderer;
use open_meteo::OpenMeteo;
use page_text::{PageText, Translations};
use transport_lan::LanTransport;

use crate::config::{Config, Place, Source};
use crate::frames::FrameSettings;
use crate::news::News;
use crate::poller::SharedArtist;

/// Where the simulated garden is, for its weather when `LATITUDE` is not set.
const MOCK_PLACE: Place = Place {
    latitude: 59.33,
    longitude: 18.07,
};

/// The face species names are set in: EB Garamond Italic, SIL OFL 1.1 (see assets/fonts).
const FONT: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");

fn main() -> ExitCode {
    let config: Config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("birdnet-go-frame: {error}");
            return ExitCode::FAILURE;
        }
    };
    let renderer: Renderer = match Renderer::new(FONT) {
        Ok(renderer) => renderer,
        Err(error) => {
            eprintln!("birdnet-go-frame: {error}");
            return ExitCode::FAILURE;
        }
    };
    let default_target = Target::Panel {
        panel: config.panel,
        rotation: config.rotation,
    };
    let store: Arc<PageStore> = Arc::new(PageStore::new(default_target));
    let log: Arc<FrameLog> = Arc::new(FrameLog::new());
    let config: Arc<Config> = Arc::new(config);
    let routes: Arc<Config> = Arc::clone(&config);
    let server: FrameServer = match FrameServer::bind(&config.listen, Arc::clone(&store)) {
        Ok(server) => {
            let server = server
                .with_frame_log(Arc::clone(&log))
                .with_router(move |query| route(&routes, query));
            let server = match &config.firmware_dir {
                Some(folder) => server.with_firmware(folder.clone()),
                None => server,
            };
            match config.quiet_hours {
                Some(quiet) => {
                    let offset_seconds: i64 = i64::from(config.utc_offset_minutes) * 60;
                    server.with_sleep_plan(move |poll_seconds| {
                        let local: i64 = poller::now_unix_seconds() + offset_seconds;
                        let of_day: u32 = u32::try_from(local.rem_euclid(86_400)).unwrap_or(0);
                        quiet.sleep_seconds(of_day, poll_seconds)
                    })
                }
                None => server,
            }
        }
        Err(error) => {
            eprintln!(
                "birdnet-go-frame: cannot listen on {}: {error}",
                config.listen
            );
            return ExitCode::FAILURE;
        }
    };
    eprintln!(
        "birdnet-go-frame: serving on http://{}/ from {:?}",
        config.listen, config.source
    );

    let art: Box<dyn ArtSource + Send> = match &config.art_dir {
        Some(folder) => match ArtFolder::open(folder) {
            Ok(art) => {
                eprintln!(
                    "birdnet-go-frame: {} plates in {}",
                    art.len(),
                    folder.display()
                );
                Box::new(art.with_paper(renderer.paper()))
            }
            Err(error) => {
                eprintln!(
                    "birdnet-go-frame: cannot read art in {}: {error}",
                    folder.display()
                );
                return ExitCode::FAILURE;
            }
        },
        // Without plates the server still runs: collages are drawn as lists.
        None => {
            eprintln!(
                "birdnet-go-frame: no ART_DIR; collages are drawn as lists until fugleramme's \
                 plates are fetched with scripts/fetch-art.sh"
            );
            Box::new(NoArt)
        }
    };
    let words: PageText = match page_words(&config) {
        Ok(words) => words,
        Err(error) => {
            eprintln!("birdnet-go-frame: {error}");
            return ExitCode::FAILURE;
        }
    };
    let style = PageStyle {
        layout: config.layout,
        max_birds: config.max_birds,
        names: config.names,
        heading: config.heading.clone(),
        words,
        facts: config.facts,
        border: config.border,
        southern: config.place.is_some_and(|place| place.latitude < 0.0),
    };
    let poller_store: Arc<PageStore> = Arc::clone(&store);
    let artist: SharedArtist<Box<dyn ArtSource + Send>> =
        Arc::new(Mutex::new(Artist::new(renderer, art, style)));
    if let Some(mqtt) = config.mqtt.clone() {
        let log: Arc<FrameLog> = Arc::clone(&log);
        thread::spawn(move || home_assistant::run(&mqtt, &log));
    }
    let poller_config: Arc<Config> = Arc::clone(&config);
    thread::spawn(move || {
        let config: Arc<Config> = poller_config;
        match config.source.clone() {
            Source::Live(url) => {
                let mut client = BirdnetClient::new(&url, LanTransport::default());
                if let Some(login) = &config.login {
                    client = client.with_login(&login.username, &login.password);
                }
                let weather = config.place.map(|place| {
                    (
                        OpenMeteo::new(&config.weather_url, LanTransport::default()),
                        place,
                    )
                });
                let clock = poller::now_unix_seconds;
                poller::run(
                    client,
                    News::new(weather),
                    clock,
                    &config,
                    &artist,
                    &poller_store,
                );
            }
            Source::Mock => {
                let offset: i32 = config.utc_offset_minutes;
                // MOCK_DATE moves the simulated garden to another day, keeping the time of day.
                let shift: i64 = config.mock_date.map_or(0, |date| {
                    let today: CivilDate =
                        DateWindow::local_date(poller::now_unix_seconds(), offset);
                    (date.days_since_epoch() - today.days_since_epoch()) * 86_400
                });
                let clock = move || poller::now_unix_seconds() + shift;
                let transport = MockTransport::with_utc_offset(clock, offset);
                let client = BirdnetClient::new("mock", transport);
                // The simulated garden has weather of its own, wherever it is said to be.
                let place: Place = config.place.unwrap_or(MOCK_PLACE);
                let sky = MockTransport::with_utc_offset(clock, offset);
                let weather = Some((OpenMeteo::new("mock", sky), place));
                poller::run(
                    client,
                    News::new(weather),
                    clock,
                    &config,
                    &artist,
                    &poller_store,
                );
            }
        }
    });

    server.run();
    ExitCode::SUCCESS
}

/// Reads a page request's query: the frame (`frame=`) decides the default panel, rotation and
/// view, which `panel=`, `rotation=` and `view=` may change. An unknown frame gets the default
/// frame's settings and an unknown view the frame's own, so a frame never gets a 400 for them.
fn route(config: &Config, query: &str) -> Result<(Target, String), String> {
    let name: &str = frame_http::frame_name(frame_http::query_value(query, "frame"));
    let frame: FrameSettings = config.frame(name);
    let view: PageView = frame_http::query_value(query, "view")
        .and_then(|view| view.parse().ok())
        .unwrap_or(frame.view);
    let default = Target::Panel {
        panel: frame.panel,
        rotation: frame.rotation,
    };
    let target: Target = Target::from_query(query, default)?;
    Ok((target, poller::view_key(&frame.name, view)))
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
