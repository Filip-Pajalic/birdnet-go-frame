//! One language's words for the page.

use alloc::string::{String, ToString};

/// The words on the page in one language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageText {
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
}

impl PageText {
    /// Returns the English words, which every other language falls back to.
    #[must_use]
    pub fn english() -> Self {
        Self {
            heading: String::from("Heard in the garden"),
            nothing_heard: String::from("No birds heard yet"),
            more: String::from("and {count} more"),
            updated: String::from("Updated {time}"),
            unreachable: String::from("Cannot reach BirdNET-Go"),
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

    /// Returns the word for `key`, for filling in from a translations file.
    pub(crate) fn slot(&mut self, key: &str) -> Option<&mut String> {
        match key {
            "heading" => Some(&mut self.heading),
            "nothing_heard" => Some(&mut self.nothing_heard),
            "more" => Some(&mut self.more),
            "updated" => Some(&mut self.updated),
            "unreachable" => Some(&mut self.unreachable),
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
    }
}
