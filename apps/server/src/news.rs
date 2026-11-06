//! Everything the pages show, fetched from BirdNET-Go and the weather service.

use std::time::Instant;

use bird_almanac::{Almanac, Arrival};
use birdnet_client::{BirdnetClient, CivilDate, ClientError, DateWindow, HttpTransport};
use birdnet_model::{SpeciesHours, SpeciesSummary};
use frame_artist::{DayFacts, YearFacts};
use open_meteo::{OpenMeteo, Weather};

use crate::config::{Config, Place};

/// What the pages beyond today's need, fetched less often than today's birds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Extras {
    /// The last seven days.
    pub week: Vec<SpeciesSummary>,
    /// Species new for the year today.
    pub new_today: Vec<SpeciesSummary>,
    /// The day's story.
    pub day: Option<DayFacts>,
    /// The year so far.
    pub year: Option<YearFacts>,
    /// Counts fetches, so a page can tell new extras from old.
    pub generation: u64,
}

/// Asks BirdNET-Go and the weather service for the extras, now and then.
pub struct News<W> {
    weather: Option<(OpenMeteo<W>, Place)>,
    last: Option<Instant>,
    extras: Extras,
}

impl<W: HttpTransport> News<W> {
    /// Creates the fetcher, with weather for `place` from `weather` when both are given.
    pub fn new(weather: Option<(OpenMeteo<W>, Place)>) -> Self {
        Self {
            weather,
            last: None,
            extras: Extras::default(),
        }
    }

    /// Returns the extras, fetching them again first when `config`'s almanac refresh has
    /// passed or `changed` says today's birds changed. A failed fetch keeps the last extras.
    pub fn extras<T: HttpTransport>(
        &mut self,
        client: &mut BirdnetClient<T>,
        config: &Config,
        now: i64,
        changed: bool,
    ) -> &Extras
    where
        T::Error: std::fmt::Display,
        W::Error: std::fmt::Display,
    {
        let due: bool = self
            .last
            .is_none_or(|last| last.elapsed() >= config.almanac_refresh);
        if due || changed {
            match self.fetch(client, config, now) {
                Ok(extras) => {
                    self.extras = Extras {
                        generation: self.extras.generation + 1,
                        ..extras
                    };
                }
                Err(error) => eprintln!("server: the year and week pages: {error}"),
            }
            self.last = Some(Instant::now());
        }
        &self.extras
    }

    fn fetch<T: HttpTransport>(
        &mut self,
        client: &mut BirdnetClient<T>,
        config: &Config,
        now: i64,
    ) -> Result<Extras, ClientError<T::Error>>
    where
        W::Error: std::fmt::Display,
    {
        let offset: i32 = config.utc_offset_minutes;
        let today: CivilDate = DateWindow::local_date(now, offset);
        let week_window: DateWindow = DateWindow::days_back(now, offset, 7);
        let week: Vec<SpeciesSummary> = client.species_summary(week_window)?;
        let year: Vec<SpeciesSummary> =
            client.species_summary(DateWindow::year_to_date(now, offset))?;
        let last_year: Vec<SpeciesSummary> = client.species_summary(week_window.year_earlier())?;
        let today_only: Vec<SpeciesSummary> =
            client.species_summary(DateWindow::days_back(now, offset, 1))?;
        let (recent, before) = YearFacts::arrival_windows(today);
        let arrivals: Vec<Arrival> = Arrival::returned(
            &client.species_summary(recent)?,
            &client.species_summary(before)?,
        );
        let almanac = Almanac::new(today, year);
        let new_today: Vec<SpeciesSummary> = almanac.new_today();
        let charted: Vec<String> = DayFacts::chart_species(&today_only, &new_today);
        let asked: Vec<&str> = charted.iter().map(String::as_str).collect();
        let hours: Vec<SpeciesHours> = client.hourly(today, &asked)?;
        let weather: Option<Weather> = self.weather.as_mut().and_then(|(service, place)| {
            service
                .today(place.latitude, place.longitude)
                .inspect_err(|error| eprintln!("server: weather: {error}"))
                .ok()
        });
        Ok(Extras {
            day: Some(DayFacts::new(
                today,
                today_only,
                hours,
                weather,
                new_today.clone(),
            )),
            year: Some(YearFacts::new(almanac, arrivals, &week, &last_year)),
            week,
            new_today,
            generation: 0,
        })
    }
}

/// Returns whether a bird new for the year was heard within `minutes` of `now`, so frames
/// should wake sooner. `local_minutes` is the local time of day, in minutes.
pub fn alert_active(extras: &Extras, local_minutes: u32, minutes: u32) -> bool {
    extras.new_today.iter().any(|row| {
        let Some(heard) = row
            .last_heard
            .as_deref()
            .and_then(|stamp| stamp.get(11..16))
        else {
            return false;
        };
        let Some((hours, minute)) = heard.split_once(':') else {
            return false;
        };
        let (Ok(hours), Ok(minute)) = (hours.parse::<u32>(), minute.parse::<u32>()) else {
            return false;
        };
        let heard: u32 = hours * 60 + minute;
        heard <= local_minutes && local_minutes - heard <= minutes
    })
}

#[cfg(test)]
mod test {
    use birdnet_model::ScientificName;

    use super::*;

    #[test]
    fn the_alert_lasts_a_while_after_the_bird_was_heard() {
        let extras = Extras {
            new_today: vec![SpeciesSummary {
                scientific_name: ScientificName::new("Bombycilla garrulus"),
                common_name: String::from("Bohemian Waxwing"),
                count: 3,
                first_heard: Some(String::from("2026-11-06T09:05:00+01:00")),
                last_heard: Some(String::from("2026-11-06T09:55:00+01:00")),
                max_confidence: None,
            }],
            ..Extras::default()
        };
        assert!(alert_active(&extras, 10 * 60, 60));
        assert!(!alert_active(&extras, 11 * 60, 60));
        assert!(!alert_active(&Extras::default(), 10 * 60, 60));
    }
}
