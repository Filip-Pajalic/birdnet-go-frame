//! What the pages are drawn from.

use birdnet_model::SpeciesSummary;

use crate::day_facts::DayFacts;
use crate::year_facts::YearFacts;

/// Everything [`crate::Artist::draw_view`] may need, borrowed. A page whose facts are
/// missing, such as the story before the hourly counts arrive, is drawn as today's birds.
#[derive(Clone, Copy, Debug)]
pub struct ViewData<'a> {
    /// The birds heard in the window, most heard first.
    pub today: &'a [SpeciesSummary],
    /// The birds heard in the last seven days, most heard first.
    pub week: &'a [SpeciesSummary],
    /// Species heard for the first time this year today, newest first.
    pub new_today: &'a [SpeciesSummary],
    /// The day's story.
    pub day: Option<&'a DayFacts>,
    /// The year so far.
    pub year: Option<&'a YearFacts>,
    /// The line at the foot of the list, story and season pages, e.g. when they were drawn.
    pub footer: &'a str,
}
