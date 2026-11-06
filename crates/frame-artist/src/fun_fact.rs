//! The line about the page's main bird.

use alloc::format;
use alloc::string::{String, ToString};

use bird_facts::{Diet, Fact, Lifestyle, Migration};
use page_text::PageText;

/// Grams in a sugar cube, the weight a bird's is told in.
const SUGAR_CUBE_GRAMS: f32 = 4.0;
/// Europe's area in km², which large ranges are told in.
const EUROPE_KM2: u32 = 10_180_000;

/// Returns `fact` about the bird called `name` as a line in `words`' language, e.g.
/// `Great Tit: weighs about 16 g, as much as 4 sugar cubes`.
#[must_use]
pub fn fun_fact(name: &str, fact: Fact, words: &PageText) -> String {
    let told: String = match fact {
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
        Fact::Range { square_kilometres } => range(square_kilometres, words),
        Fact::North { degrees } => words.fact_north.replace("{degrees}", &degrees.to_string()),
        Fact::South { degrees } => words.fact_south.replace("{degrees}", &degrees.to_string()),
        Fact::Migration(Migration::Resident) => words.fact_resident.clone(),
        Fact::Migration(Migration::Partial) => words.fact_partial.clone(),
        Fact::Migration(Migration::Migrant) => words.fact_migrant.clone(),
        Fact::Diet(Diet::Anything) => words.fact_omnivore.clone(),
        Fact::Diet(diet) => words.fact_diet.replace("{food}", food(diet, words)),
        Fact::Lifestyle(Lifestyle::Aerial) => words.fact_aerial.clone(),
        Fact::Lifestyle(Lifestyle::Aquatic) => words.fact_aquatic.clone(),
        Fact::Lifestyle(Lifestyle::Terrestrial) => words.fact_terrestrial.clone(),
        Fact::Lifestyle(Lifestyle::Perching) => words.fact_perching.clone(),
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

fn range(square_kilometres: u32, words: &PageText) -> String {
    if square_kilometres >= 2 * EUROPE_KM2 {
        let times: u32 = (square_kilometres + EUROPE_KM2 / 2) / EUROPE_KM2;
        words
            .fact_range_europe
            .replace("{times}", &times.to_string())
    } else if square_kilometres >= 1_000_000 {
        let millions: u32 = (square_kilometres + 500_000) / 1_000_000;
        words
            .fact_range_millions
            .replace("{millions}", &millions.to_string())
    } else {
        // Two significant figures: a range's edge is never known to the square kilometre.
        let mut scale: u32 = 1;
        while square_kilometres / scale >= 100 {
            scale *= 10;
        }
        let rounded: u32 = (square_kilometres + scale / 2) / scale * scale;
        words.fact_range.replace("{area}", &grouped(rounded.max(1)))
    }
}

fn food(diet: Diet, words: &PageText) -> &str {
    match diet {
        Diet::Invertebrates => &words.food_invertebrates,
        Diet::Seeds => &words.food_seeds,
        Diet::Fruit => &words.food_fruit,
        Diet::Nectar => &words.food_nectar,
        Diet::Plants => &words.food_plants,
        Diet::WaterPlants => &words.food_water_plants,
        Diet::WaterAnimals => &words.food_water_animals,
        Diet::Vertebrates => &words.food_vertebrates,
        Diet::Carrion => &words.food_carrion,
        Diet::Anything => &words.fact_omnivore,
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
            tell(Fact::Range {
                square_kilometres: 32_578_000
            }),
            "Great Tit: its range is 3 times the size of Europe"
        );
        assert_eq!(
            tell(Fact::Range {
                square_kilometres: 10_572_000
            }),
            "Great Tit: its range covers 11 million km²"
        );
        assert_eq!(
            tell(Fact::Range {
                square_kilometres: 45_321
            }),
            "Great Tit: its range covers 45 000 km²"
        );
        assert_eq!(
            tell(Fact::Diet(Diet::Seeds)),
            "Great Tit: eats mostly seeds"
        );
        assert_eq!(
            tell(Fact::Diet(Diet::Anything)),
            "Great Tit: eats almost anything"
        );
        assert_eq!(
            fun_fact("Talgoxe", Fact::Wingspan { centimetres: 23 }, &swedish),
            "Talgoxe: 23 cm mellan vingspetsarna"
        );
        assert_eq!(
            fun_fact("Talgoxe", Fact::North { degrees: 71 }, &swedish),
            "Talgoxe: finns så långt norrut som 71° N"
        );
    }

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(grouped(7), "7");
        assert_eq!(grouped(1_000), "1 000");
        assert_eq!(grouped(1_234_567), "1 234 567");
    }
}
