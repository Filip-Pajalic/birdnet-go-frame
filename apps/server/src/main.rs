//! birdnet-go-frame's server.
//!
//! Reads BirdNET-Go's API, draws the birds heard onto a page for a 13.3" Spectra 6 panel, and
//! serves it at `/frame.e6` (for the ESP32), `/frame.png` and `/preview.png`. Configured with
//! environment variables; see the README. `BIRDNET_URL=mock` runs against a built-in
//! simulated station, so it works out of the box.

mod artist;
mod config;
mod poller;

use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

use art_mock::MockArt;
use birdnet_client::BirdnetClient;
use birdnet_mock::MockTransport;
use frame_http::{FrameServer, PageStore};
use frame_raster::Renderer;
use transport_lan::LanTransport;

use crate::artist::Artist;
use crate::config::{Config, Source};

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
    let store: Arc<PageStore> = Arc::new(PageStore::new());
    let server: FrameServer = match FrameServer::bind(&config.listen, Arc::clone(&store)) {
        Ok(server) => server,
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

    let poller_store: Arc<PageStore> = Arc::clone(&store);
    // Generated pictures until real art is supported; see docs/explanation/image-pipeline.md.
    let mut artist = Artist::new(
        renderer,
        MockArt::default(),
        config.layout,
        config.max_birds,
    );
    thread::spawn(move || match config.source.clone() {
        Source::Live(url) => {
            let client = BirdnetClient::new(&url, LanTransport::default());
            poller::run(client, &config, &mut artist, &poller_store);
        }
        Source::Mock => {
            let transport =
                MockTransport::with_utc_offset(poller::now_unix_seconds, config.utc_offset_minutes);
            let client = BirdnetClient::new("mock", transport);
            poller::run(client, &config, &mut artist, &poller_store);
        }
    });

    server.run();
    ExitCode::SUCCESS
}
