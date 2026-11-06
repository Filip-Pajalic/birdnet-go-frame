//! The line about the page's main bird.

use alloc::format;
use alloc::string::{String, ToString};

use bird_facts::Fact;
use page_text::PageText;

/// Grams in a sugar cube, the weight a bird's is told in.
const SUGAR_CUBE_GRAMS: f32 = 4.0;

/// Returns `fact` about the bird called `name` as a line in `words`' language, e.g.
/// `Great Tit: weighs about 16 g, as much as 4 sugar cubes`. A note is told as written.
#[must_use]
pub fn fun_fact(name: &str, fact: Fact, words: &PageText) -> String {
    let told: String = match fact {
        Fact::Note(note) => String::from(note),
        Fact::Weight { grams } => {
            let cubes: u32 = libm::roundf(grams / SUGAR_CUBE_GRAMS) as u32;
            let template: &str = if cubes <= 1 {
                &words.fact_weight_one
            } else {
                &words.fact_weight
            };
            template
                .replace("{weight}", &weight(grams))
                .replace("{count}", &grouped(cubes))
        }
        Fact::Wingspan { centimetres } => words
            .fact_wingspan
            .replace("{cm}", &centimetres.to_string()),
        Fact::Incubation { days } => words.fact_incubation.replace("{days}", &days.to_string()),
        Fact::Oldest { years } => words.fact_oldest.replace("{years}", &years.to_string()),
    };
    words
        .fact_line
        .replace("{name}", name)
        .replace("{fact}", &told)
}

/// A weight in grams below ten kilograms, to the nearest 100 g above one, else in kilograms.
fn weight(grams: f32) -> String {
    if grams >= 10_000.0 {
        format!("{} kg", libm::roundf(grams / 1_000.0) as u32)
    } else if grams >= 1_000.0 {
        format!("{} g", grouped(libm::roundf(grams / 100.0) as u32 * 100))
    } else {
        format!("{} g", libm::roundf(grams).max(1.0) as u32)
    }
}

/// `number` with a space between each group of three digits, as SI writes them.
fn grouped(number: u32) -> String {
    let digits: String = number.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod test {
    use page_text::Translations;

    use super::*;

    #[test]
    fn tells_each_fact_in_english_and_swedish() {
        let english = PageText::english();
        let swedish = Translations::bundled().page_text("sv");
        let tell = |fact: Fact| fun_fact("Great Tit", fact, &english);
        assert_eq!(
            tell(Fact::Weight { grams: 16.3 }),
            "Great Tit: weighs about 16 g, as much as 4 sugar cubes"
        );
        assert_eq!(
            tell(Fact::Weight { grams: 3.1 }),
            "Great Tit: weighs about 3 g, as much as a sugar cube"
        );
        assert_eq!(
            tell(Fact::Weight { grams: 10_300.0 }),
            "Great Tit: weighs about 10 kg, as much as 2 575 sugar cubes"
        );
        assert_eq!(
            tell(Fact::Weight { grams: 1_440.0 }),
            "Great Tit: weighs about 1 400 g, as much as 360 sugar cubes"
        );
        assert_eq!(
            tell(Fact::Note("its two-note song starts in January")),
            "Great Tit: its two-note song starts in January"
        );
        assert_eq!(
            fun_fact("Talgoxe", Fact::Wingspan { centimetres: 23 }, &swedish),
            "Talgoxe: 23 cm mellan vingspetsarna"
        );
        assert_eq!(
            fun_fact("Talgoxe", Fact::Oldest { years: 15 }, &swedish),
            "Talgoxe: den äldsta kända blev 15 år"
        );
    }

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(grouped(7), "7");
        assert_eq!(grouped(1_000), "1 000");
        assert_eq!(grouped(1_234_567), "1 234 567");
    }
}
