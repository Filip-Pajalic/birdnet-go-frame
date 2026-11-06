//! The render loop: ask BirdNET-Go, and publish new pages when the birds change.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use art_pack::ArtSource;
use birdnet_client::{BirdnetClient, DateWindow, HttpTransport};
use birdnet_model::SpeciesSummary;
use frame_artist::{Artist, PageStyle, PageView, ViewData};
use frame_core::{IndexedImage, Rgb};
use frame_http::{PageStore, RenderedPage, Target};
use frame_raster::Canvas;

use crate::config::Config;
use crate::frames::FrameSettings;
use crate::news::{self, Extras, News};

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

/// The store's name for one frame's page in one view: `frame/view`.
pub fn view_key(frame: &str, view: PageView) -> String {
    format!("{frame}/{view}")
}

/// Polls `client` forever, publishing new pages to `store` whenever the set of species heard
/// or the extras change. Keeps the last good pages up while BirdNET-Go is unreachable.
/// `clock` returns the unix time to ask about, the real time unless the mock pretends.
pub fn run<T: HttpTransport, W: HttpTransport, A: ArtSource + Send + 'static>(
    mut client: BirdnetClient<T>,
    mut news: News<W>,
    clock: impl Fn() -> i64,
    config: &Arc<Config>,
    artist: &SharedArtist<A>,
    store: &PageStore,
) where
    T::Error: std::fmt::Display,
    W::Error: std::fmt::Display,
{
    let mut shown: Option<(i64, Vec<String>, u64, bool)> = None;
    loop {
        let now: i64 = clock();
        let window = DateWindow::hours_back(now, config.utc_offset_minutes, config.window_hours);
        match client.species_summary(window) {
            Ok(species) => {
                let names: Vec<String> = species
                    .iter()
                    .map(|row| row.scientific_name.to_string())
                    .collect();
                // A new day may dress a bird in another plate, so it redraws too.
                let day: i64 = now.div_euclid(SECONDS_PER_DAY);
                let changed: bool = shown.as_ref().is_none_or(|(shown_day, shown_names, _, _)| {
                    *shown_day != day || *shown_names != names
                });
                let extras: Extras = news.extras(&mut client, config, now, changed).clone();
                let local: i64 = now + i64::from(config.utc_offset_minutes) * 60;
                let minute: u32 =
                    u32::try_from(local.rem_euclid(SECONDS_PER_DAY) / 60).unwrap_or(0);
                let alert: bool = config.alert_poll_minutes > 0
                    && news::alert_active(&extras, minute, config.alert_minutes);
                let wanted = (day, names, extras.generation, alert);
                if shown.as_ref() != Some(&wanted) {
                    let time: String = wall_clock(now, config.utc_offset_minutes);
                    let Ok(footer) = artist.lock().map(|mut artist| {
                        artist.start_day(day);
                        artist.words().updated(&time)
                    }) else {
                        return;
                    };
                    let count: usize = species.len();
                    let page = Page::Birds(Arc::new(Birds {
                        today: species,
                        extras,
                        footer,
                        alert,
                    }));
                    publish(page, artist, config, store);
                    eprintln!(
                        "server: drew {count} species{}",
                        if alert { ", a new bird is about" } else { "" }
                    );
                    shown = Some(wanted);
                }
            }
            Err(error) => {
                eprintln!("server: {error}");
                if shown.is_none() && !store.is_ready() {
                    let page = Page::Unreachable(format!("{error}"));
                    publish(page, artist, config, store);
                }
            }
        }
        thread::sleep(config.refresh);
    }
}

/// Everything the bird pages show.
struct Birds {
    /// The birds heard in the window, most heard first.
    today: Vec<SpeciesSummary>,
    /// The week, the year and the day's story.
    extras: Extras,
    /// The list page's foot line.
    footer: String,
    /// Whether frames should wake sooner, for a bird new for the year.
    alert: bool,
}

/// What the published pages show.
#[derive(Clone)]
enum Page {
    /// The birds heard.
    Birds(Arc<Birds>),
    /// BirdNET-Go cannot be reached, and why.
    Unreachable(String),
}

/// Publishes `page` for every target and view, and draws the default one straight away so
/// the panel's next wake is answered at once.
fn publish<A: ArtSource + Send + 'static>(
    page: Page,
    artist: &SharedArtist<A>,
    config: &Arc<Config>,
    store: &PageStore,
) {
    let artist: SharedArtist<A> = Arc::clone(artist);
    let default: String = view_key(frame_http::DEFAULT_FRAME, config.view);
    let config: Arc<Config> = Arc::clone(config);
    store.publish_views(Box::new(move |target: Target, key: &str| {
        let (name, view): (&str, &str) = key.split_once('/').unwrap_or((key, ""));
        let frame: FrameSettings = config.frame(name);
        let view: PageView = view.parse().unwrap_or(frame.view);
        let (preview, paper, alert): (Canvas, Rgb, bool) = {
            let mut artist = artist.lock().ok()?;
            let style = PageStyle {
                layout: frame.layout,
                max_birds: frame.max_birds,
                names: frame.names,
                heading: frame.heading.clone(),
                words: artist.words().clone(),
                facts: frame.facts,
                border: frame.border,
                southern: artist.style().southern,
            };
            artist.set_style(style);
            let (preview, alert): (Canvas, bool) = match &page {
                Page::Birds(birds) => (draw(&mut artist, birds, view, target), birds.alert),
                Page::Unreachable(problem) => {
                    (artist.draw_unreachable(problem, target.resolution()), false)
                }
            };
            (preview, artist.paper(), alert)
        };
        let poll_minutes: u8 = if alert {
            config.alert_poll_minutes.min(config.poll_minutes).max(1)
        } else {
            config.poll_minutes
        };
        let inks: IndexedImage = frame_raster::dither(&frame_raster::pop(&preview, paper));
        RenderedPage::for_target(target, &inks, &preview, poll_minutes)
            .inspect_err(|error| eprintln!("server: {error}"))
            .ok()
    }));
    if store.view(store.default_target(), &default).is_none() {
        eprintln!("server: the page could not be drawn");
    }
}

/// Draws `view` of `birds` for `target` in the artist's current style.
fn draw<A: ArtSource>(
    artist: &mut Artist<A>,
    birds: &Birds,
    view: PageView,
    target: Target,
) -> Canvas {
    let extras: &Extras = &birds.extras;
    let data = ViewData {
        today: &birds.today,
        week: &extras.week,
        new_today: &extras.new_today,
        day: extras.day.as_ref(),
        year: extras.year.as_ref(),
        footer: &birds.footer,
    };
    artist.draw_view(view, &data, target.resolution())
}

/// Formats the local wall-clock time as `HH:MM`.
fn wall_clock(now_unix_seconds: i64, utc_offset_minutes: i32) -> String {
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
        assert_eq!(wall_clock(1_793_946_600, 60), "07:30");
    }

    #[test]
    fn keys_name_the_frame_and_view() {
        assert_eq!(view_key("desk", PageView::Story), "desk/story");
    }
}
