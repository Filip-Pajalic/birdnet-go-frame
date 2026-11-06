//! What the story page tells about today.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use bird_almanac::{DayStory, Singer};
use birdnet_client::CivilDate;
use birdnet_model::{SpeciesHours, SpeciesSummary};
use frame_core::Ink;
use frame_layout::{ChartMark, ChorusRow, CornerSprays, StoryOptions};
use open_meteo::Weather;
use page_text::PageText;

/// Most species given a row of their own on the chorus chart, besides birds new for the year.
pub const CHART_SPECIES: usize = 8;

/// Today's birds and weather, as the story page tells them.
#[derive(Clone, Debug, PartialEq)]
pub struct DayFacts {
    /// Today's species, most heard first.
    pub species: Vec<SpeciesSummary>,
    /// Their hourly counts, from [`DayFacts::chart_species`].
    pub hours: Vec<SpeciesHours>,
    /// Today's weather, if it is known.
    pub weather: Option<Weather>,
    /// Species new for the year today, drawn in red.
    pub new_today: Vec<SpeciesSummary>,
    /// The story worked out from the above.
    pub story: DayStory,
}

impl DayFacts {
    /// Gathers the facts for `date`: `species` heard today (most heard first), `hours` for
    /// [`Self::chart_species`] of them, the weather and the birds new for the year.
    #[must_use]
    pub fn new(
        date: CivilDate,
        species: Vec<SpeciesSummary>,
        hours: Vec<SpeciesHours>,
        weather: Option<Weather>,
        new_today: Vec<SpeciesSummary>,
    ) -> Self {
        let story: DayStory = DayStory::new(date, &species, &hours);
        Self {
            species,
            hours,
            weather,
            new_today,
            story,
        }
    }

    /// Returns the scientific names whose hourly counts the chart needs: the most heard
    /// [`CHART_SPECIES`] and any of `new_today`.
    #[must_use]
    pub fn chart_species(species: &[SpeciesSummary], new_today: &[SpeciesSummary]) -> Vec<String> {
        let mut names: Vec<String> = species
            .iter()
            .take(CHART_SPECIES)
            .map(|row| row.scientific_name.to_string())
            .collect();
        for row in new_today {
            if !species
                .iter()
                .take(CHART_SPECIES)
                .any(|held| held.scientific_name == row.scientific_name)
            {
                names.push(row.scientific_name.to_string());
            }
        }
        names
    }

    /// Returns the story page's settings in `words`, with `footer` under it.
    #[must_use]
    pub fn story_options(&self, words: &PageText, footer: &str) -> StoryOptions {
        let singer = |template: &str, singer: &Singer| {
            template
                .replace("{time}", &singer.time)
                .replace("{name}", &label(&singer.species))
        };
        let mut lines: Vec<String> = Vec::new();
        if let Some(first) = &self.story.first {
            lines.push(singer(&words.first_song, first));
        }
        if let Some(last) = &self.story.last {
            lines.push(singer(&words.last_song, last));
        }
        if let Some(hour) = self.story.peak_hour {
            lines.push(
                words
                    .chorus_peak
                    .replace("{time}", &alloc::format!("{hour:02}:00")),
            );
        }
        let mut marks: Vec<ChartMark> = Vec::new();
        if let Some(weather) = &self.weather {
            let sky: String = words.word(weather.sky.key()).unwrap_or_default();
            let degrees = |value: f32| alloc::format!("{value:.0}");
            lines.push(
                words
                    .weather
                    .replace("{sky}", &sky)
                    .replace("{now}", &degrees(weather.temperature_c))
                    .replace("{low}", &degrees(weather.low_c))
                    .replace("{high}", &degrees(weather.high_c)),
            );
            for (time, label) in [
                (&weather.sunrise, &words.sunrise),
                (&weather.sunset, &words.sunset),
            ] {
                if let Some(minute) = minutes(time) {
                    marks.push(ChartMark {
                        minute,
                        label: alloc::format!("{label} {time}"),
                    });
                }
            }
        }
        let mut rows: Vec<ChorusRow> = Vec::with_capacity(self.hours.len() + 1);
        rows.push(ChorusRow {
            label: words.whole_chorus.clone(),
            hours: self.story.chorus,
            ink: Ink::Black,
            heavy: true,
        });
        // Birds new for the year first, so a small page that cannot fit every row keeps them.
        let mut ordered: Vec<&SpeciesHours> = self.hours.iter().collect();
        ordered.sort_by_key(|counts| {
            !self
                .new_today
                .iter()
                .any(|row| row.scientific_name == counts.scientific_name)
        });
        for counts in ordered {
            let is_new: bool = self
                .new_today
                .iter()
                .any(|row| row.scientific_name == counts.scientific_name);
            let named: String = self
                .species
                .iter()
                .chain(&self.new_today)
                .find(|row| row.scientific_name == counts.scientific_name)
                .map_or_else(|| counts.scientific_name.to_string(), label);
            rows.push(ChorusRow {
                label: named,
                hours: counts.hours,
                // Black hatching like the rest of the page; red, the rubric, for a new bird.
                ink: if is_new { Ink::Red } else { Ink::Black },
                heavy: false,
            });
        }
        StoryOptions {
            heading: words.story_heading.clone(),
            footer: String::from(footer),
            sprays: CornerSprays::default(),
            lines,
            rows,
            marks,
            empty: words.nothing_heard.clone(),
        }
    }
}

/// The species' common name, or its scientific one when BirdNET-Go gave none.
pub(crate) fn label(row: &SpeciesSummary) -> String {
    if row.common_name.is_empty() {
        row.scientific_name.to_string()
    } else {
        row.common_name.clone()
    }
}

/// Minutes after midnight of `HH:MM`.
fn minutes(clock: &str) -> Option<u32> {
    let (hours, minutes) = clock.split_once(':')?;
    let (hours, minutes): (u32, u32) = (hours.parse().ok()?, minutes.parse().ok()?);
    (hours < 24 && minutes < 60).then_some(hours * 60 + minutes)
}

#[cfg(test)]
mod test {
    use birdnet_client::{BirdnetClient, DateWindow};
    use birdnet_mock::MockTransport;

    use super::*;

    #[test]
    fn tells_the_mock_gardens_day() {
        // 2026-11-06T09:00:00Z.
        let now: i64 = 1_793_955_600;
        let mut client = BirdnetClient::new("mock", MockTransport::new(move || now));
        let date: CivilDate = DateWindow::local_date(now, 0);
        let species: Vec<SpeciesSummary> = client
            .species_summary(DateWindow::days_back(now, 0, 1))
            .expect("mock");
        let names: Vec<String> = DayFacts::chart_species(&species, &[]);
        let asked: Vec<&str> = names.iter().map(String::as_str).collect();
        let hours: Vec<SpeciesHours> = client.hourly(date, &asked).expect("mock");
        let facts = DayFacts::new(date, species, hours, None, Vec::new());
        let options: StoryOptions = facts.story_options(&PageText::english(), "Updated 09:00");
        assert_eq!(options.rows.len(), CHART_SPECIES + 1);
        assert_eq!(options.rows[0].label, "All birds");
        assert!(
            options.lines[0].starts_with("First song 0"),
            "{:?}",
            options.lines
        );
        assert!(
            options
                .lines
                .iter()
                .any(|line| line.starts_with("The chorus peaked at 0"))
        );
        assert_eq!(minutes("07:19"), Some(439));
    }
}
