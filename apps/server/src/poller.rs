//! The render loop: ask BirdNET-Go, redraw when the birds change, publish the page.

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use art_pack::ArtSource;
use birdnet_client::{BirdnetClient, ClientError, DateWindow, HttpTransport};
use frame_core::{IndexedImage, Rgb, e6};
use frame_http::{PageStore, RenderedPage};
use frame_raster::Canvas;

use crate::artist::Artist;
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

/// Polls `client` forever, publishing a new page to `store` whenever the set of species heard
/// changes. Keeps the last good page up while BirdNET-Go is unreachable.
pub fn run<T: HttpTransport, A: ArtSource>(
    mut client: BirdnetClient<T>,
    config: &Config,
    artist: &mut Artist<A>,
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
                    artist.start_day(day);
                    let footer: String = artist
                        .words()
                        .updated(&clock(now, config.utc_offset_minutes));
                    let preview: Canvas =
                        artist.draw(&species, &footer, e6::page_resolution(config.rotation));
                    publish(&preview, artist.paper(), config, store);
                    eprintln!("server: drew {} species", species.len());
                    shown = Some(wanted);
                }
            }
            Err(error) => {
                eprintln!("server: {error}");
                if shown.is_none() && store.get().is_none() {
                    publish_notice(&error, config, artist, store);
                }
            }
        }
        thread::sleep(config.refresh);
    }
}

fn publish(preview: &Canvas, paper: Rgb, config: &Config, store: &PageStore) {
    let page: IndexedImage = frame_raster::dither(&frame_raster::pop(preview, paper));
    match RenderedPage::new(&page, preview, config.rotation, config.poll_minutes) {
        Ok(rendered) => store.set(rendered),
        Err(error) => eprintln!("server: {error}"),
    }
}

fn publish_notice<E: std::fmt::Display, A: ArtSource>(
    error: &ClientError<E>,
    config: &Config,
    artist: &Artist<A>,
    store: &PageStore,
) {
    let problem: String = format!("{error}");
    let preview: Canvas = artist.draw_unreachable(&problem, e6::page_resolution(config.rotation));
    publish(&preview, artist.paper(), config, store);
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
