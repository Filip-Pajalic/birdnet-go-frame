//! The render loop: ask BirdNET-Go, redraw when the birds change, publish the page.

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use birdnet_client::{BirdnetClient, ClientError, DateWindow, HttpTransport};
use birdnet_model::SpeciesSummary;
use frame_core::{IndexedImage, e6};
use frame_http::{PageStore, RenderedPage};
use frame_layout::{PageLayout, TextList, TextListOptions};
use frame_raster::{Canvas, Renderer};

use crate::config::Config;

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
pub fn run<T: HttpTransport>(
    mut client: BirdnetClient<T>,
    config: &Config,
    renderer: &Renderer,
    store: &PageStore,
) {
    let mut shown: Option<Vec<String>> = None;
    loop {
        let now: i64 = now_unix_seconds();
        let window = DateWindow::hours_back(now, config.utc_offset_minutes, config.window_hours);
        match client.species_summary(window) {
            Ok(species) => {
                let names: Vec<String> = species
                    .iter()
                    .map(|row| row.scientific_name.to_string())
                    .collect();
                if shown.as_ref() != Some(&names) {
                    let footer: String =
                        format!("Updated {}", clock(now, config.utc_offset_minutes));
                    publish(&species, &footer, config, renderer, store);
                    eprintln!("server: drew {} species", species.len());
                    shown = Some(names);
                }
            }
            Err(error) => {
                eprintln!("server: {error}");
                if shown.is_none() && store.get().is_none() {
                    publish_notice(&error, config, renderer, store);
                }
            }
        }
        thread::sleep(config.refresh);
    }
}

fn publish(
    species: &[SpeciesSummary],
    footer: &str,
    config: &Config,
    renderer: &Renderer,
    store: &PageStore,
) {
    let options = TextListOptions {
        heading: config.heading.clone(),
        footer: footer.to_owned(),
        ..TextListOptions::default()
    };
    let layout: PageLayout =
        TextList::layout(species, e6::page_resolution(config.rotation), &options);
    let preview: Canvas = renderer.render(&layout);
    let page: IndexedImage = frame_raster::dither(&preview);
    match RenderedPage::new(&page, &preview, config.rotation, config.poll_minutes) {
        Ok(rendered) => store.set(rendered),
        Err(error) => eprintln!("server: {error}"),
    }
}

fn publish_notice<E: std::fmt::Display>(
    error: &ClientError<E>,
    config: &Config,
    renderer: &Renderer,
    store: &PageStore,
) {
    let options = TextListOptions {
        heading: String::from("Cannot reach BirdNET-Go"),
        footer: format!("{error}"),
        ..TextListOptions::default()
    };
    let layout: PageLayout = TextList::layout(&[], e6::page_resolution(config.rotation), &options);
    let preview: Canvas = renderer.render(&layout);
    let page: IndexedImage = frame_raster::dither(&preview);
    if let Ok(rendered) = RenderedPage::new(&page, &preview, config.rotation, config.poll_minutes) {
        store.set(rendered);
    }
}

/// Formats the local wall-clock time as `HH:MM`.
fn clock(now_unix_seconds: i64, utc_offset_minutes: i32) -> String {
    let local: i64 = now_unix_seconds + i64::from(utc_offset_minutes) * 60;
    let minutes_today: i64 = local.rem_euclid(86_400) / 60;
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
