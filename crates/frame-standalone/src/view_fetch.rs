//! What a standalone frame asks BirdNET-Go for each page, and what it makes of the answers.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use bird_almanac::{Almanac, Arrival};
use birdnet_client::{CivilDate, DateWindow};
use birdnet_model::{SpeciesHours, SpeciesSummary};
use frame_artist::{DayFacts, PageView, ViewData, YearFacts};

/// One species summary a page needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Summary {
    /// The settings' window of hours: today's birds.
    Window,
    /// The last seven days.
    Week,
    /// The year so far.
    Year,
    /// The same seven days a year earlier.
    WeekLastYear,
    /// Since local midnight, for the story.
    Day,
    /// The last weeks, to find arrivals.
    Recent,
    /// The weeks before those.
    Before,
    /// The whole of last year, for the bird of the day while this year has no birds yet.
    LastYear,
}

/// The requests for one page and, once answered, the facts it is drawn from. Without a server
/// the frame asks only for what the page it shows needs: today's page is one request, the
/// season's five. The story's hourly counts depend on the day's birds, so they are asked for
/// after the summaries ([`hourly_paths`](Self::hourly_paths)). Weather and the new-bird
/// alert need a server and are left out. A page that turns out to have no birds asks for
/// more afterwards ([`next_fallback`](Self::next_fallback)), to find the bird of the day.
///
/// ```
/// use frame_artist::PageView;
/// use frame_standalone::{Summary, ViewFetch};
///
/// // 2026-11-06 07:00 UTC, in UTC+1, with a 24-hour window.
/// let fetch = ViewFetch::new(PageView::Rarest, 1_793_948_400, 60, 24);
/// let asked: Vec<Summary> = fetch.summaries().map(|(summary, _)| summary).collect();
/// assert_eq!(asked, [Summary::Window, Summary::Year]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ViewFetch {
    view: PageView,
    today: CivilDate,
    year: DateWindow,
    last_year: DateWindow,
    windows: Vec<(Summary, DateWindow)>,
    answers: Vec<(Summary, Vec<SpeciesSummary>)>,
    hours: Vec<SpeciesHours>,
}

impl ViewFetch {
    /// Plans `view` at unix time `now` in a garden `utc_offset_minutes` east of UTC, today's
    /// birds being those of the last `window_hours`.
    #[must_use]
    pub fn new(view: PageView, now: i64, utc_offset_minutes: i32, window_hours: u32) -> Self {
        let today: CivilDate = DateWindow::local_date(now, utc_offset_minutes);
        let week: DateWindow = DateWindow::days_back(now, utc_offset_minutes, 7);
        let (recent, before) = YearFacts::arrival_windows(today);
        let window_of = |summary: Summary| -> DateWindow {
            match summary {
                Summary::Window => DateWindow::hours_back(now, utc_offset_minutes, window_hours),
                Summary::Week => week,
                Summary::Year => DateWindow::year_to_date(now, utc_offset_minutes),
                Summary::WeekLastYear => week.year_earlier(),
                Summary::Day => DateWindow::days_back(now, utc_offset_minutes, 1),
                Summary::Recent => recent,
                Summary::Before => before,
                Summary::LastYear => DateWindow::previous_year(now, utc_offset_minutes),
            }
        };
        let needed: &[Summary] = match view {
            PageView::Today => &[Summary::Window],
            PageView::Week => &[Summary::Week],
            PageView::Rarest => &[Summary::Window, Summary::Year],
            PageView::Story => &[Summary::Window, Summary::Day],
            PageView::Season => &[
                Summary::Year,
                Summary::Week,
                Summary::WeekLastYear,
                Summary::Recent,
                Summary::Before,
            ],
        };
        Self {
            view,
            today,
            year: window_of(Summary::Year),
            last_year: window_of(Summary::LastYear),
            windows: needed
                .iter()
                .map(|&summary| (summary, window_of(summary)))
                .collect(),
            answers: Vec::new(),
            hours: Vec::new(),
        }
    }

    /// Returns the page being fetched.
    #[must_use]
    pub fn view(&self) -> PageView {
        self.view
    }

    /// Returns each summary the page needs and its path on BirdNET-Go, from `/api/v2`.
    pub fn summaries(&self) -> impl Iterator<Item = (Summary, String)> + '_ {
        self.windows
            .iter()
            .map(|(summary, window)| (*summary, window.species_summary_path()))
    }

    /// Keeps BirdNET-Go's answer to `summary`.
    pub fn answer(&mut self, summary: Summary, species: Vec<SpeciesSummary>) {
        self.answers.retain(|(kept, _)| *kept != summary);
        self.answers.push((summary, species));
    }

    /// Returns the next summary to ask for, and its path from `/api/v2`, once the page's own
    /// answers are in and it has no birds: the year so far, then last year while this year has
    /// none. The page then shows the bird of the day from them. `None` when the page has birds
    /// or nothing more can be asked; answer each with [`answer`](Self::answer) before calling
    /// again.
    pub fn next_fallback(&mut self) -> Option<(Summary, String)> {
        let own: Summary = match self.view {
            PageView::Today | PageView::Rarest => Summary::Window,
            PageView::Week => Summary::Week,
            PageView::Story => Summary::Day,
            PageView::Season => Summary::Year,
        };
        if !self.list(own).is_empty() {
            return None;
        }
        let asked = |summary: Summary| self.windows.iter().any(|(kept, _)| *kept == summary);
        let (summary, window): (Summary, DateWindow) = if !asked(Summary::Year) {
            (Summary::Year, self.year)
        } else if self.list(Summary::Year).is_empty() && !asked(Summary::LastYear) {
            (Summary::LastYear, self.last_year)
        } else {
            return None;
        };
        self.windows.push((summary, window));
        Some((summary, window.species_summary_path()))
    }

    /// Returns the paths of the story's hourly counts, from `/api/v2`, once the day's summary
    /// is in; none for other pages.
    #[must_use]
    pub fn hourly_paths(&self) -> Vec<String> {
        if self.view != PageView::Story {
            return Vec::new();
        }
        let names: Vec<String> = DayFacts::chart_species(self.list(Summary::Day), &[]);
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        names
            .chunks(birdnet_client::HOURLY_BATCH)
            .map(|batch| format!("/api/v2{}", birdnet_client::hourly_path(self.today, batch)))
            .collect()
    }

    /// Keeps hourly counts from an answer to one of [`hourly_paths`](Self::hourly_paths).
    pub fn answer_hours(&mut self, hours: Vec<SpeciesHours>) {
        self.hours.extend(hours);
    }

    /// Returns the answers to every summary, in the order asked, for telling whether the page
    /// changed.
    #[must_use]
    pub fn lists(&self) -> Vec<&[SpeciesSummary]> {
        self.windows
            .iter()
            .map(|(summary, _)| self.list(*summary))
            .collect()
    }

    /// Works out the story's and the season's facts from the answers.
    #[must_use]
    pub fn facts(&self) -> ViewFacts {
        let day: Option<DayFacts> = (self.view == PageView::Story).then(|| {
            DayFacts::new(
                self.today,
                self.list(Summary::Day).to_vec(),
                self.hours.clone(),
                None,
                Vec::new(),
            )
        });
        let year: Option<YearFacts> = match self.view {
            PageView::Rarest | PageView::Season => {
                let almanac = Almanac::new(self.today, self.list(Summary::Year).to_vec());
                let arrivals: Vec<Arrival> =
                    Arrival::returned(self.list(Summary::Recent), self.list(Summary::Before));
                Some(YearFacts::new(
                    almanac,
                    arrivals,
                    self.list(Summary::Week),
                    self.list(Summary::WeekLastYear),
                ))
            }
            _ => None,
        };
        ViewFacts { day, year }
    }

    /// Returns the pages' data, with `facts` from [`facts`](Self::facts) and `footer` under the
    /// list, story and season pages.
    #[must_use]
    pub fn data<'a>(&'a self, facts: &'a ViewFacts, footer: &'a str) -> ViewData<'a> {
        ViewData {
            today: self.list(Summary::Window),
            week: self.list(Summary::Week),
            new_today: &[],
            day: facts.day.as_ref(),
            year: facts.year.as_ref(),
            known: self.known(),
            footer,
        }
    }

    /// This year's birds, or last year's while this year has none.
    fn known(&self) -> &[SpeciesSummary] {
        match self.list(Summary::Year) {
            [] => self.list(Summary::LastYear),
            year => year,
        }
    }

    fn list(&self, summary: Summary) -> &[SpeciesSummary] {
        self.answers
            .iter()
            .find(|(kept, _)| *kept == summary)
            .map_or(&[], |(_, species)| species.as_slice())
    }
}

/// The story's and the season's facts, worked out from a [`ViewFetch`]'s answers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewFacts {
    /// The day's story, for the story page.
    pub day: Option<DayFacts>,
    /// The year so far, for the rarest and season pages.
    pub year: Option<YearFacts>,
}

#[cfg(test)]
mod test {
    use super::*;

    const NOW: i64 = 1_793_948_400;

    fn birds(json: &[u8]) -> Vec<SpeciesSummary> {
        SpeciesSummary::parse_list(json).expect("valid JSON")
    }

    #[test]
    fn the_buttons_reach_every_page_the_artist_draws() {
        let stepped: Vec<&str> = panel_wake::PageView::ALL
            .iter()
            .map(|view| view.as_str())
            .collect();
        let drawn: Vec<&str> = PageView::ALL.iter().map(|view| view.name()).collect();
        assert_eq!(stepped, drawn);
        for name in drawn {
            let view: PageView = name.parse().expect("a view");
            assert!(ViewFetch::new(view, NOW, 60, 24).summaries().count() > 0);
        }
    }

    #[test]
    fn asks_each_page_for_what_it_shows() {
        let asked = |view: PageView| -> Vec<Summary> {
            ViewFetch::new(view, NOW, 60, 24)
                .summaries()
                .map(|(summary, _)| summary)
                .collect()
        };
        assert_eq!(asked(PageView::Today), [Summary::Window]);
        assert_eq!(asked(PageView::Week), [Summary::Week]);
        assert_eq!(asked(PageView::Story), [Summary::Window, Summary::Day]);
        assert_eq!(asked(PageView::Season).len(), 5);
        let fetch = ViewFetch::new(PageView::Week, NOW, 60, 24);
        let (_, path) = fetch.summaries().next().expect("one");
        assert_eq!(
            path,
            "/api/v2/analytics/species/summary?start_date=2026-10-31&end_date=2026-11-06"
        );
        assert!(fetch.hourly_paths().is_empty());
    }

    #[test]
    fn the_story_asks_for_its_birds_hours_then_tells_the_day() {
        let mut fetch = ViewFetch::new(PageView::Story, NOW, 60, 24);
        let day = birds(
            br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":9,
                  "first_heard":"2026-11-06T06:40:00+01:00"}]"#,
        );
        fetch.answer(Summary::Day, day.clone());
        fetch.answer(Summary::Window, day);
        assert_eq!(
            fetch.hourly_paths(),
            ["/api/v2/analytics/time/hourly/batch?date=2026-11-06&species=Parus%20major"]
        );
        let facts: ViewFacts = fetch.facts();
        let story: &DayFacts = facts.day.as_ref().expect("a story");
        assert_eq!(story.species.len(), 1);
        assert!(facts.year.is_none());
        let data: ViewData<'_> = fetch.data(&facts, "Updated 08:00");
        assert_eq!(data.today.len(), 1);
        assert!(data.day.is_some());
    }

    #[test]
    fn the_season_gathers_the_year() {
        let mut fetch = ViewFetch::new(PageView::Season, NOW, 60, 24);
        let year = birds(
            br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":900,
                  "first_heard":"2026-01-02T08:00:00+01:00"}]"#,
        );
        fetch.answer(Summary::Year, year.clone());
        fetch.answer(Summary::Week, year);
        let facts: ViewFacts = fetch.facts();
        let season: &YearFacts = facts.year.as_ref().expect("a year");
        assert_eq!(season.almanac.year_list().len(), 1);
        assert_eq!(season.year_on_year.now, 1);
        assert_eq!(fetch.lists().len(), 5);
    }

    #[test]
    fn a_quiet_page_asks_for_the_year_then_last_year() {
        let tit =
            birds(br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":9}]"#);
        let mut busy = ViewFetch::new(PageView::Today, NOW, 60, 24);
        busy.answer(Summary::Window, tit.clone());
        assert!(busy.next_fallback().is_none());

        let mut quiet = ViewFetch::new(PageView::Today, NOW, 60, 24);
        quiet.answer(Summary::Window, Vec::new());
        let (summary, path) = quiet.next_fallback().expect("the year");
        assert_eq!(summary, Summary::Year);
        assert_eq!(
            path,
            "/api/v2/analytics/species/summary?start_date=2026-01-01&end_date=2026-11-06"
        );
        quiet.answer(Summary::Year, Vec::new());
        let (summary, path) = quiet.next_fallback().expect("last year");
        assert_eq!(summary, Summary::LastYear);
        assert_eq!(
            path,
            "/api/v2/analytics/species/summary?start_date=2025-01-01&end_date=2025-12-31"
        );
        quiet.answer(Summary::LastYear, tit);
        assert!(quiet.next_fallback().is_none());
        let facts: ViewFacts = quiet.facts();
        assert_eq!(quiet.data(&facts, "").known.len(), 1);
        assert_eq!(quiet.lists().len(), 3);

        let mut rarest = ViewFetch::new(PageView::Rarest, NOW, 60, 24);
        rarest.answer(Summary::Window, Vec::new());
        rarest.answer(
            Summary::Year,
            birds(br#"[{"scientific_name":"Parus major","count":1}]"#),
        );
        assert!(rarest.next_fallback().is_none());
        let facts: ViewFacts = rarest.facts();
        assert_eq!(rarest.data(&facts, "").known.len(), 1);
    }
}
