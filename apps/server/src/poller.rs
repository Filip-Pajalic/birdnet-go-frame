//! The render loop: ask BirdNET-Go, and publish new pages when the birds change.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use art_pack::ArtSource;
use birdnet_client::{BirdnetClient, DateWindow, HttpTransport};
use birdnet_model::SpeciesSummary;
use frame_artist::Artist;
use frame_core::{IndexedImage, Rgb};
use frame_http::{PageStore, RenderedPage, Target};
use frame_raster::Canvas;

use crate::config::Config;

const SECONDS_PER_DAY: i64 = 86_400;

/// Returns the current unix time in seconds.
pub fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

/// The artist, shared by the poller and the HTTP threads that draw pages on request.
pub type SharedArtist<A> = Arc<Mutex<Artist<A>>>;

/// Polls `client` forever, publishing new pages to `store` whenever the set of species heard
/// changes. Keeps the last good pages up while BirdNET-Go is unreachable.
pub fn run<T: HttpTransport, A: ArtSource + Send + 'static>(
    mut client: BirdnetClient<T>,
    config: &Config,
    artist: &SharedArtist<A>,
    store: &PageStore,
) {
    let mut shown: Option<(i64, Vec<String>)> = None;
    loop {
        let now: i64 = now_unix_seconds();
        let window = DateWindow::hours_back(now, config.utc_offset_minutes, config.window_hours);
        match client.species_summary(window) {
            Ok(species) => {
                let names: Vec<String> = species
                    .iter()
                    .map(|row| row.scientific_name.to_string())
                    .collect();
                // A new day may dress a bird in another plate, so it redraws too.
                let day: i64 = now.div_euclid(SECONDS_PER_DAY);
                let wanted: (i64, Vec<String>) = (day, names);
                if shown.as_ref() != Some(&wanted) {
                    let time: String = clock(now, config.utc_offset_minutes);
                    let Ok(footer) = artist.lock().map(|mut artist| {
                        artist.start_day(day);
                        artist.words().updated(&time)
                    }) else {
                        return;
                    };
                    let count: usize = species.len();
                    let page = Page::Birds {
                        species: Arc::new(species),
                        footer,
                    };
                    publish(page, artist, config.poll_minutes, store);
                    eprintln!("server: drew {count} species");
                    shown = Some(wanted);
                }
            }
            Err(error) => {
                eprintln!("server: {error}");
                if shown.is_none() && !store.is_ready() {
                    let page = Page::Unreachable(format!("{error}"));
                    publish(page, artist, config.poll_minutes, store);
                }
            }
        }
        thread::sleep(config.refresh);
    }
}

/// What the published pages show.
#[derive(Clone)]
enum Page {
    /// The birds heard, most heard first, and the list page's foot line.
    Birds {
        species: Arc<Vec<SpeciesSummary>>,
        footer: String,
    },
    /// BirdNET-Go cannot be reached, and why.
    Unreachable(String),
}

/// Publishes `page` for every target, and draws the default one straight away so the panel's
/// next wake is answered at once.
fn publish<A: ArtSource + Send + 'static>(
    page: Page,
    artist: &SharedArtist<A>,
    poll_minutes: u8,
    store: &PageStore,
) {
    let artist: SharedArtist<A> = Arc::clone(artist);
    store.publish(Box::new(move |target: Target| {
        let (preview, paper): (Canvas, Rgb) = {
            let mut artist = artist.lock().ok()?;
            let preview: Canvas = match &page {
                Page::Birds { species, footer } => {
                    artist.draw(species, footer, target.resolution())
                }
                Page::Unreachable(problem) => artist.draw_unreachable(problem, target.resolution()),
            };
            (preview, artist.paper())
        };
        let inks: IndexedImage = frame_raster::dither(&frame_raster::pop(&preview, paper));
        RenderedPage::for_target(target, &inks, &preview, poll_minutes)
            .inspect_err(|error| eprintln!("server: {error}"))
            .ok()
    }));
    if store.get().is_none() {
        eprintln!("server: the page could not be drawn");
    }
}

/// Formats the local wall-clock time as `HH:MM`.
fn clock(now_unix_seconds: i64, utc_offset_minutes: i32) -> String {
    let local: i64 = now_unix_seconds + i64::from(utc_offset_minutes) * 60;
    let minutes_today: i64 = local.rem_euclid(SECONDS_PER_DAY) / 60;
    format!("{:02}:{:02}", minutes_today / 60, minutes_today % 60)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_applies_offset() {
        // 2026-11-06T06:30:00Z at UTC+1.
        assert_eq!(clock(1_793_946_600, 60), "07:30");
    }
}
