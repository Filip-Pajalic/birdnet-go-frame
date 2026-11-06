//! birdnet-go-frame's server.
//!
//! Reads BirdNET-Go's API, draws the birds heard onto pages for Spectra 6 panels (13.3" by
//! default, 7.3" on request) and screens of any size, and serves them at `/panel` (for the
//! ESP32), `/frame.png` and `/preview.png`. Configured with
//! environment variables; see the README. `BIRDNET_URL=mock` runs against a built-in
//! simulated station, so it works out of the box.

mod artist;
mod config;
mod poller;
mod quiet_hours;

use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

use art_mock::MockArt;
use art_pack::ArtSource;
use art_webp::ArtFolder;
use birdnet_client::BirdnetClient;
use birdnet_mock::MockTransport;
use frame_http::{FrameServer, PageStore, Target};
use frame_raster::Renderer;
use page_text::{PageText, Translations};
use transport_lan::LanTransport;

use crate::artist::{Artist, PageStyle};
use crate::config::{Config, Source};
use crate::poller::SharedArtist;

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
    let server: FrameServer = match FrameServer::bind(&config.listen, Arc::clone(&store)) {
        Ok(server) => match config.quiet_hours {
            Some(quiet) => {
                let offset_seconds: i64 = i64::from(config.utc_offset_minutes) * 60;
                server.with_sleep_plan(move |poll_seconds| {
                    let local: i64 = poller::now_unix_seconds() + offset_seconds;
                    let of_day: u32 = u32::try_from(local.rem_euclid(86_400)).unwrap_or(0);
                    quiet.sleep_seconds(of_day, poll_seconds)
                })
            }
            None => server,
        },
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
        // Generated birds, so the server works with nothing else installed.
        None => Box::new(MockArt::default()),
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
    };
    let poller_store: Arc<PageStore> = Arc::clone(&store);
    let artist: SharedArtist<Box<dyn ArtSource + Send>> =
        Arc::new(Mutex::new(Artist::new(renderer, art, style)));
    thread::spawn(move || match config.source.clone() {
        Source::Live(url) => {
            let mut client = BirdnetClient::new(&url, LanTransport::default());
            if let Some(login) = &config.login {
                client = client.with_login(&login.username, &login.password);
            }
            poller::run(client, &config, &artist, &poller_store);
        }
        Source::Mock => {
            let transport =
                MockTransport::with_utc_offset(poller::now_unix_seconds, config.utc_offset_minutes);
            let client = BirdnetClient::new("mock", transport);
            poller::run(client, &config, &artist, &poller_store);
        }
    });

    server.run();
    ExitCode::SUCCESS
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
