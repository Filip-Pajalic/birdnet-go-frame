//! Reading translations files.

use alloc::string::String;
use alloc::vec::Vec;

use lite_json::{Json, JsonError};

use crate::page_text::PageText;

/// The translations shipped with the crate.
const BUNDLED: &[u8] = include_bytes!("../data/translations.json");

/// Every language's words, as read from one or more translations files.
///
/// A file is one JSON object keyed by language code (`en`, `sv`, `pt-BR`), each holding an
/// object of key to text; see [`PageText`] for the keys. Unknown keys and values that are not
/// text are ignored, so a file can be partial.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Translations {
    languages: Vec<(String, Vec<(String, String)>)>,
}

impl Translations {
    /// Returns the bundled translations.
    #[must_use]
    #[expect(
        clippy::expect_used,
        clippy::missing_panics_doc,
        reason = "cannot fail: the bundled file is checked by tests"
    )]
    pub fn bundled() -> Self {
        Self::parse(BUNDLED).expect("the bundled translations are valid JSON")
    }

    /// Reads a translations file.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError`] when the file is not JSON. A file that is JSON but not shaped as
    /// expected reads as having no translations.
    pub fn parse(bytes: &[u8]) -> Result<Self, JsonError> {
        let document: Json = Json::parse(bytes)?;
        let languages: Vec<(String, Vec<(String, String)>)> = document
            .as_object()
            .unwrap_or_default()
            .iter()
            .filter_map(|(code, words)| {
                let words: Vec<(String, String)> = words
                    .as_object()?
                    .iter()
                    .filter_map(|(key, text)| match text {
                        Json::String(text) => Some((key.clone(), text.clone())),
                        _ => None,
                    })
                    .collect();
                Some((normalise(code), words))
            })
            .collect();
        Ok(Self { languages })
    }

    /// Adds `other`'s words, which win where both have one.
    pub fn merge(&mut self, other: Self) {
        for (code, words) in other.languages {
            match self.languages.iter_mut().find(|(known, _)| *known == code) {
                Some((_, existing)) => {
                    for (key, text) in words {
                        match existing.iter_mut().find(|(known, _)| *known == key) {
                            Some((_, old)) => *old = text,
                            None => existing.push((key, text)),
                        }
                    }
                }
                None => self.languages.push((code, words)),
            }
        }
    }

    /// Returns the language codes present.
    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.languages.iter().map(|(code, _)| code.as_str())
    }

    /// Returns the words for `language` (`sv`, `sv-SE`, `nb_NO`), falling back key by key to
    /// its main language (`sv` for `sv-SE`), then to English.
    #[must_use]
    pub fn page_text(&self, language: &str) -> PageText {
        let wanted: String = normalise(language);
        let main: &str = wanted.split('-').next().unwrap_or("");
        // Norwegian without a written standard means Bokmål, as most software assumes.
        let main: &str = if main == "no" { "nb" } else { main };
        let mut text = PageText::english();
        for code in ["en", main, wanted.as_str()] {
            let Some((_, words)) = self.languages.iter().find(|(known, _)| known == code) else {
                continue;
            };
            if code == main {
                text.language = String::from(main);
            }
            for (key, word) in words {
                if let Some(slot) = text.slot(key) {
                    slot.clone_from(word);
                }
            }
        }
        text
    }
}

/// Lower case, with `-` between parts: `pt_BR` becomes `pt-br`.
fn normalise(code: &str) -> String {
    code.trim().to_ascii_lowercase().replace('_', "-")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bundled_languages_are_complete() {
        let translations = Translations::bundled();
        assert!(translations.languages().count() >= 12);
        let english = PageText::english();
        for code in translations.languages() {
            let words = translations.page_text(code);
            if code != "en" {
                assert_ne!(words.heading, english.heading, "{code}");
                assert_ne!(words.nothing_heard, english.nothing_heard, "{code}");
                assert_ne!(words.more, english.more, "{code}");
                assert_ne!(words.updated, english.updated, "{code}");
                assert_ne!(words.unreachable, english.unreachable, "{code}");
            }
            assert!(words.more.contains("{count}"), "{code}");
            assert!(words.updated.contains("{time}"), "{code}");
        }
    }

    #[test]
    fn falls_back_by_region_then_to_english() {
        let mut translations = Translations::bundled();
        let extra = Translations::parse(
            r#"{"sv_FI": {"heading": "Hört på gården"}, "xx": {"heading": 3}}"#.as_bytes(),
        )
        .expect("valid JSON");
        translations.merge(extra);
        let finland = translations.page_text("sv-FI");
        assert_eq!(finland.heading, "Hört på gården");
        assert_eq!(finland.nothing_heard, "Inga fåglar hörda än");
        assert_eq!(finland.language, "sv");
        assert_eq!(translations.page_text("no").language, "nb");
        assert_eq!(translations.page_text("zz").language, "en");
        assert_eq!(translations.page_text("no").heading, "Hørt i hagen");
        assert_eq!(translations.page_text("xx").heading, "Heard in the garden");
        assert_eq!(translations.page_text("").heading, "Heard in the garden");
    }

    #[test]
    fn merging_overrides_single_words() {
        let mut translations = Translations::bundled();
        translations.merge(
            Translations::parse(br#"{"en": {"heading": "Our feeder"}}"#).expect("valid JSON"),
        );
        let english = translations.page_text("en");
        assert_eq!(english.heading, "Our feeder");
        assert_eq!(english.more, "and {count} more");
        assert!(Translations::parse(b"{nope").is_err());
    }
}
