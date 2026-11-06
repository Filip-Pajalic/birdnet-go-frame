//! One language's words for the page.

use alloc::string::{String, ToString};

/// The words on the page in one language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageText {
    /// The language the words are in, e.g. `sv`: the main part of the code asked for when
    /// there is a translation for it, else `en`. Not a translation key.
    pub language: String,
    /// The heading of the list page (`heading`).
    pub heading: String,
    /// Shown when no bird has been heard (`nothing_heard`).
    pub nothing_heard: String,
    /// Counts species left off a page (`more`); `{count}` stands for the number.
    pub more: String,
    /// When the page was drawn (`updated`); `{time}` stands for the clock time.
    pub updated: String,
    /// Heading of the page shown when BirdNET-Go cannot be reached (`unreachable`).
    pub unreachable: String,
    /// The heading of the week page (`heading_week`).
    pub heading_week: String,
    /// The heading of the rarest page (`heading_rarest`).
    pub heading_rarest: String,
    /// The heading of a page with no birds of its own, drawing one bird heard before
    /// (`bird_of_the_day`).
    pub bird_of_the_day: String,
    /// The heading when a bird is new for the year; `{name}` stands for it (`new_for_year`).
    pub new_for_year: String,
    /// The heading of the day's story (`story_heading`).
    pub story_heading: String,
    /// The first bird of the day; `{time}` and `{name}` (`first_song`).
    pub first_song: String,
    /// The latest bird so far; `{time}` and `{name}` (`last_song`).
    pub last_song: String,
    /// The busiest hour; `{time}` (`chorus_peak`).
    pub chorus_peak: String,
    /// The chart row of every bird together (`whole_chorus`).
    pub whole_chorus: String,
    /// The weather line; `{sky}`, `{now}`, `{low}` and `{high}` in degrees (`weather`).
    pub weather: String,
    /// Sunrise, marked on the chart (`sunrise`).
    pub sunrise: String,
    /// Sunset, marked on the chart (`sunset`).
    pub sunset: String,
    /// The heading of the season page (`season_heading`).
    pub season_heading: String,
    /// How many species this year; `{count}` (`year_count`).
    pub year_count: String,
    /// The title of the arrivals column (`arrivals`).
    pub arrivals: String,
    /// Shown when nothing has arrived lately (`no_arrivals`).
    pub no_arrivals: String,
    /// The title of the year-on-year column (`last_year`).
    pub last_year: String,
    /// This week against last year's; `{now}` and `{then}` (`now_then`).
    pub now_then: String,
    /// A bird heard now but not last year (`gained`).
    pub gained: String,
    /// A bird heard last year but not now (`missing`).
    pub missing: String,
    /// The title of the year list (`year_list`).
    pub year_list: String,
    /// Month abbreviations, January first, separated by spaces (`months`).
    pub months: String,
    /// Clear sky (`sky_clear`).
    pub sky_clear: String,
    /// Partly cloudy (`sky_partly_cloudy`).
    pub sky_partly_cloudy: String,
    /// Overcast (`sky_overcast`).
    pub sky_overcast: String,
    /// Fog (`sky_fog`).
    pub sky_fog: String,
    /// Drizzle (`sky_drizzle`).
    pub sky_drizzle: String,
    /// Rain (`sky_rain`).
    pub sky_rain: String,
    /// Snow (`sky_snow`).
    pub sky_snow: String,
    /// Thunder (`sky_thunder`).
    pub sky_thunder: String,
    /// Weather the page cannot name (`sky_unknown`).
    pub sky_unknown: String,
    /// A fun fact about the page's main bird; `{name}` and `{fact}` (`fact_line`).
    pub fact_line: String,
    /// Its weight; `{weight}`, e.g. `18 g`, and `{count}` sugar cubes of 4 g (`fact_weight`).
    pub fact_weight: String,
    /// Its weight, when about one sugar cube's; `{weight}` (`fact_weight_one`).
    pub fact_weight_one: String,
    /// Its wingspan; `{cm}` (`fact_wingspan`).
    pub fact_wingspan: String,
    /// Days its eggs take to hatch; `{days}` (`fact_incubation`).
    pub fact_incubation: String,
    /// The oldest one known; `{years}` (`fact_oldest`).
    pub fact_oldest: String,
}

impl PageText {
    /// Returns the English words, which every other language falls back to.
    #[must_use]
    pub fn english() -> Self {
        Self {
            language: String::from("en"),
            heading: String::from("Heard in the garden"),
            nothing_heard: String::from("No birds heard yet"),
            more: String::from("and {count} more"),
            updated: String::from("Updated {time}"),
            unreachable: String::from("Cannot reach BirdNET-Go"),
            heading_week: String::from("Heard this week"),
            heading_rarest: String::from("Rarest today"),
            bird_of_the_day: String::from("Bird of the day"),
            new_for_year: String::from("New this year: {name}"),
            story_heading: String::from("The day so far"),
            first_song: String::from("First song {time}, {name}"),
            last_song: String::from("Latest {time}, {name}"),
            chorus_peak: String::from("The chorus peaked at {time}"),
            whole_chorus: String::from("All birds"),
            weather: String::from("{sky}, {now}° (low {low}°, high {high}°)"),
            sunrise: String::from("Sunrise"),
            sunset: String::from("Sunset"),
            season_heading: String::from("The year so far"),
            year_count: String::from("{count} species this year"),
            arrivals: String::from("Just arrived"),
            no_arrivals: String::from("Nothing new lately"),
            last_year: String::from("This week last year"),
            now_then: String::from("{now} species now, {then} then"),
            gained: String::from("new"),
            missing: String::from("missing"),
            year_list: String::from("Year list"),
            months: String::from("Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec"),
            sky_clear: String::from("Clear"),
            sky_partly_cloudy: String::from("Partly cloudy"),
            sky_overcast: String::from("Overcast"),
            sky_fog: String::from("Fog"),
            sky_drizzle: String::from("Drizzle"),
            sky_rain: String::from("Rain"),
            sky_snow: String::from("Snow"),
            sky_thunder: String::from("Thunder"),
            sky_unknown: String::from("Weather"),
            fact_line: String::from("{name}: {fact}"),
            fact_weight: String::from("weighs about {weight}, as much as {count} sugar cubes"),
            fact_weight_one: String::from("weighs about {weight}, as much as a sugar cube"),
            fact_wingspan: String::from("{cm} cm from wingtip to wingtip"),
            fact_incubation: String::from("the eggs hatch after about {days} days"),
            fact_oldest: String::from("the oldest one known lived {years} years"),
        }
    }

    /// Returns the `more` line for `count` species.
    #[must_use]
    pub fn more(&self, count: usize) -> String {
        self.more.replace("{count}", &count.to_string())
    }

    /// Returns the `updated` line for clock time `time`, e.g. `07:30`.
    #[must_use]
    pub fn updated(&self, time: &str) -> String {
        self.updated.replace("{time}", time)
    }

    /// Returns the word for `key`, e.g. a sky's [`sky_rain`](Self::sky_rain), or `None` for a
    /// key the page does not use.
    #[must_use]
    pub fn word(&self, key: &str) -> Option<String> {
        self.clone().slot(key).map(|word| word.clone())
    }

    /// Returns month `month` (1 to 12) from [`months`](Self::months), or its number when the
    /// translation has fewer names.
    #[must_use]
    pub fn month(&self, month: u8) -> String {
        self.months
            .split_whitespace()
            .nth(usize::from(month.max(1)) - 1)
            .map_or_else(|| month.to_string(), String::from)
    }

    /// Returns the word for `key`, for filling in from a translations file.
    pub(crate) fn slot(&mut self, key: &str) -> Option<&mut String> {
        match key {
            "heading" => Some(&mut self.heading),
            "nothing_heard" => Some(&mut self.nothing_heard),
            "more" => Some(&mut self.more),
            "updated" => Some(&mut self.updated),
            "unreachable" => Some(&mut self.unreachable),
            "heading_week" => Some(&mut self.heading_week),
            "heading_rarest" => Some(&mut self.heading_rarest),
            "bird_of_the_day" => Some(&mut self.bird_of_the_day),
            "new_for_year" => Some(&mut self.new_for_year),
            "story_heading" => Some(&mut self.story_heading),
            "first_song" => Some(&mut self.first_song),
            "last_song" => Some(&mut self.last_song),
            "chorus_peak" => Some(&mut self.chorus_peak),
            "whole_chorus" => Some(&mut self.whole_chorus),
            "weather" => Some(&mut self.weather),
            "sunrise" => Some(&mut self.sunrise),
            "sunset" => Some(&mut self.sunset),
            "season_heading" => Some(&mut self.season_heading),
            "year_count" => Some(&mut self.year_count),
            "arrivals" => Some(&mut self.arrivals),
            "no_arrivals" => Some(&mut self.no_arrivals),
            "last_year" => Some(&mut self.last_year),
            "now_then" => Some(&mut self.now_then),
            "gained" => Some(&mut self.gained),
            "missing" => Some(&mut self.missing),
            "year_list" => Some(&mut self.year_list),
            "months" => Some(&mut self.months),
            "sky_clear" => Some(&mut self.sky_clear),
            "sky_partly_cloudy" => Some(&mut self.sky_partly_cloudy),
            "sky_overcast" => Some(&mut self.sky_overcast),
            "sky_fog" => Some(&mut self.sky_fog),
            "sky_drizzle" => Some(&mut self.sky_drizzle),
            "sky_rain" => Some(&mut self.sky_rain),
            "sky_snow" => Some(&mut self.sky_snow),
            "sky_thunder" => Some(&mut self.sky_thunder),
            "sky_unknown" => Some(&mut self.sky_unknown),
            "fact_line" => Some(&mut self.fact_line),
            "fact_weight" => Some(&mut self.fact_weight),
            "fact_weight_one" => Some(&mut self.fact_weight_one),
            "fact_wingspan" => Some(&mut self.fact_wingspan),
            "fact_incubation" => Some(&mut self.fact_incubation),
            "fact_oldest" => Some(&mut self.fact_oldest),
            _ => None,
        }
    }
}

impl Default for PageText {
    fn default() -> Self {
        Self::english()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fills_in_placeholders() {
        let words = PageText::english();
        assert_eq!(words.more(3), "and 3 more");
        assert_eq!(words.updated("21:05"), "Updated 21:05");
        assert_eq!(words.word("sky_rain").as_deref(), Some("Rain"));
        assert_eq!(words.word("nope"), None);
        assert_eq!(
            (words.month(11).as_str(), words.month(13).as_str()),
            ("Nov", "13")
        );
    }
}
