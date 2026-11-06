//! What the season page tells about the year.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use bird_almanac::{Almanac, Arrival, YearOnYear};
use birdnet_client::CivilDate;
use birdnet_model::SpeciesSummary;
use frame_layout::{SeasonColumn, SeasonOptions};
use page_text::PageText;

use crate::day_facts::label;

/// How far back an arrival counts as recent, in days.
pub const ARRIVAL_DAYS: u32 = 21;

/// The year so far, and this week against the same week last year.
#[derive(Clone, Debug, PartialEq)]
pub struct YearFacts {
    /// The year so far.
    pub almanac: Almanac,
    /// This week and the same week last year.
    pub year_on_year: YearOnYear,
}

impl YearFacts {
    /// Gathers the facts from the year's almanac and the summaries of the last seven days and
    /// the same seven days a year earlier.
    #[must_use]
    pub fn new(almanac: Almanac, week: &[SpeciesSummary], last_year: &[SpeciesSummary]) -> Self {
        Self {
            almanac,
            year_on_year: YearOnYear::new(week, last_year),
        }
    }

    /// Returns the season page's settings in `words`, with `footer` under it.
    #[must_use]
    pub fn season_options(&self, words: &PageText, footer: &str) -> SeasonOptions {
        let date = |date: CivilDate| format!("{} {}", date.day, words.month(date.month));
        let arrivals: Vec<Arrival> = self.almanac.arrivals(ARRIVAL_DAYS);
        let mut left: Vec<(String, String)> = arrivals
            .iter()
            .map(|arrival| (label(&arrival.species), date(arrival.date)))
            .collect();
        if left.is_empty() {
            left.push((words.no_arrivals.clone(), String::new()));
        }
        let compared: &YearOnYear = &self.year_on_year;
        let mut right: Vec<(String, String)> = alloc::vec![(
            words
                .now_then
                .replace("{now}", &format!("{}", compared.now))
                .replace("{then}", &format!("{}", compared.then)),
            String::new(),
        )];
        right.extend(
            compared
                .gained
                .iter()
                .take(3)
                .map(|row| (label(row), words.gained.clone())),
        );
        right.extend(
            compared
                .missing
                .iter()
                .take(3)
                .map(|row| (label(row), words.missing.clone())),
        );
        // The year list in the order the birds were first heard, like a birder's notebook.
        let mut list: Vec<&SpeciesSummary> = self.almanac.year_list().iter().collect();
        list.sort_by(|left, right| {
            left.first_heard
                .cmp(&right.first_heard)
                .then_with(|| left.common_name.cmp(&right.common_name))
        });
        SeasonOptions {
            heading: words.season_heading.clone(),
            footer: String::from(footer),
            headline: words
                .year_count
                .replace("{count}", &format!("{}", self.almanac.year_list().len())),
            left: SeasonColumn {
                title: words.arrivals.clone(),
                lines: left,
            },
            right: SeasonColumn {
                title: words.last_year.clone(),
                lines: right,
            },
            list_title: words.year_list.clone(),
            list: list.into_iter().map(label).collect(),
            more: words.more.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use birdnet_client::{BirdnetClient, DateWindow};
    use birdnet_mock::MockTransport;

    use super::*;

    #[test]
    fn sums_up_the_mock_gardens_year() {
        // 2026-11-06T09:00:00Z, when the winter thrushes have just come.
        let now: i64 = 1_793_955_600;
        let mut client = BirdnetClient::new("mock", MockTransport::new(move || now));
        let year: Vec<SpeciesSummary> = client
            .species_summary(DateWindow::year_to_date(now, 0))
            .expect("mock");
        let week_window: DateWindow = DateWindow::days_back(now, 0, 7);
        let week: Vec<SpeciesSummary> = client.species_summary(week_window).expect("mock");
        let last_year: Vec<SpeciesSummary> = client
            .species_summary(week_window.year_earlier())
            .expect("mock");
        let facts = YearFacts::new(
            Almanac::new(DateWindow::local_date(now, 0), year),
            &week,
            &last_year,
        );
        let options: SeasonOptions = facts.season_options(&PageText::english(), "");
        assert!(options.headline.ends_with("species this year"));
        assert_eq!(options.list.len(), facts.almanac.year_list().len());
        assert!(options.list[0].len() > 2);
        assert!(options.right.lines[0].0.contains("species now"));
    }
}
