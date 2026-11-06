//! Draws pages: picks the layout, finds each bird's picture and size, renders.

use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use art_pack::{ArtImage, ArtSource};
use bird_facts::{BirdFacts, Fact};
use bird_sizes::BirdSizes;
use birdnet_client::CivilDate;
use birdnet_model::{ScientificName, SpeciesSummary};
use frame_core::{Resolution, Rgb};
use frame_layout::{
    BorderTheme, Collage, CollageBird, CollageOptions, PageLayout, SeasonBorder, SeasonPage,
    Silhouette, StoryPage, TextList, TextListOptions, size_weights,
};
use frame_raster::{Canvas, Renderer};
use page_text::PageText;

use crate::day_facts::DayFacts;
use crate::fun_fact;
use crate::layout::Layout;
use crate::page_style::PageStyle;
use crate::page_view::PageView;
use crate::view_data::ViewData;
use crate::year_facts::YearFacts;

/// How much bigger a featured bird is drawn than its weight alone would make it.
const FEATURED_BOOST: f32 = 2.2;
/// Most birds on the rarest page's collage, unless the style sets fewer.
const RAREST_BIRDS: usize = 6;

/// A species' picture and its outline, worked out once a day.
struct Picture {
    art: ArtImage,
    silhouette: Silhouette,
}

/// Draws pages with one font, one source of pictures and one style.
pub struct Artist<A: ArtSource> {
    renderer: Renderer,
    art: A,
    pictures: BTreeMap<String, Option<Picture>>,
    day: Option<i64>,
    sizes: BirdSizes<'static>,
    facts: BirdFacts<'static>,
    style: PageStyle,
}

impl<A: ArtSource> Artist<A> {
    /// Creates an artist that draws with `renderer`'s font and paper, `art`'s pictures and
    /// `style`.
    #[must_use]
    pub fn new(renderer: Renderer, art: A, style: PageStyle) -> Self {
        Self {
            renderer,
            art,
            pictures: BTreeMap::new(),
            day: None,
            sizes: BirdSizes::bundled(),
            facts: BirdFacts::bundled(),
            style,
        }
    }

    /// Returns the paper pages are drawn on.
    #[must_use]
    pub fn paper(&self) -> Rgb {
        self.renderer.paper()
    }

    /// Returns the page's words.
    #[must_use]
    pub fn words(&self) -> &PageText {
        &self.style.words
    }

    /// Returns how pages are set out.
    #[must_use]
    pub fn style(&self) -> &PageStyle {
        &self.style
    }

    /// Sets pages out with `style` from now on, e.g. another frame's. Pictures stay cached.
    pub fn set_style(&mut self, style: PageStyle) {
        self.style = style;
    }

    /// Starts day `day` (days since 1970, in the garden's time zone): art sources may pick
    /// another plate for a species, so cached pictures are forgotten. The day also picks the
    /// fact told about a bird and the border; before the first day there is no border.
    pub fn start_day(&mut self, day: i64) {
        if self.day != Some(day) {
            self.pictures.clear();
            self.day = Some(day);
        }
    }

    /// Draws `species` (most heard first) on a page of `resolution`. `footer` goes under the
    /// list page; the collage, like fugleramme's, is birds only. A collage for which the art
    /// source has no picture of any of the birds is drawn as the list.
    pub fn draw(
        &mut self,
        species: &[SpeciesSummary],
        footer: &str,
        resolution: Resolution,
    ) -> Canvas {
        self.draw_featured(species, None, None, footer, resolution)
    }

    /// Like [`draw`](Self::draw), with `heading` instead of the style's and `featured` (one
    /// of `species`, e.g. a bird new for the year) put first: drawn biggest on the collage,
    /// and kept on it whatever the style's most birds.
    pub fn draw_featured(
        &mut self,
        species: &[SpeciesSummary],
        featured: Option<&ScientificName>,
        heading: Option<&str>,
        footer: &str,
        resolution: Resolution,
    ) -> Canvas {
        let mut ordered: Vec<SpeciesSummary> = species.to_vec();
        if let Some(featured) = featured {
            if let Some(index) = ordered
                .iter()
                .position(|row| row.scientific_name == *featured)
            {
                let row: SpeciesSummary = ordered.remove(index);
                ordered.insert(0, row);
            }
        }
        let featured: Option<&ScientificName> = featured.filter(|name| {
            ordered
                .first()
                .is_some_and(|row| row.scientific_name == **name)
        });
        let shown: &[SpeciesSummary] = match self.style.max_birds {
            0 => &ordered,
            limit => &ordered[..ordered.len().min(limit)],
        };
        match self.style.layout {
            Layout::List => {
                let heading: String = heading
                    .map(String::from)
                    .or_else(|| self.style.heading.clone())
                    .unwrap_or_else(|| self.style.words.heading.clone());
                let fact: String = self.fact_line(ordered.first());
                self.draw_list(&ordered, &heading, &fact, footer, resolution)
            }
            Layout::Collage => {
                let heading: String = heading
                    .map(String::from)
                    .or_else(|| self.style.heading.clone())
                    .unwrap_or_default();
                let shown: Vec<SpeciesSummary> = shown.to_vec();
                for row in &shown {
                    self.ensure_picture(&row.scientific_name.to_string());
                }
                // Without a single plate, e.g. before the art is installed, the collage would be
                // empty: the birds are listed instead.
                let any_pictured: bool = shown.iter().any(|row| {
                    self.pictures
                        .get(&row.scientific_name.to_string())
                        .is_some_and(Option::is_some)
                });
                let fact: String = self.fact_line(ordered.first());
                if shown.is_empty() || any_pictured {
                    self.draw_collage(&shown, featured.is_some(), &heading, &fact, resolution)
                } else {
                    let heading: String = if heading.is_empty() {
                        self.style.words.heading.clone()
                    } else {
                        heading
                    };
                    self.draw_list(&ordered, &heading, &fact, footer, resolution)
                }
            }
        }
    }

    /// Draws the day's story: who sang first and last, the weather, and a chart of when each
    /// bird sang.
    #[must_use]
    pub fn draw_story(&self, day: &DayFacts, footer: &str, resolution: Resolution) -> Canvas {
        let mut layout: PageLayout =
            StoryPage::layout(resolution, &day.story_options(&self.style.words, footer));
        self.decorate(&mut layout);
        self.renderer.render(&layout, &[])
    }

    /// Draws the year so far: how many species, the latest arrivals, this week against the
    /// same week last year, and the year list.
    #[must_use]
    pub fn draw_season(&self, year: &YearFacts, footer: &str, resolution: Resolution) -> Canvas {
        let mut layout: PageLayout =
            SeasonPage::layout(resolution, &year.season_options(&self.style.words, footer));
        self.decorate(&mut layout);
        self.renderer.render(&layout, &[])
    }

    /// Draws `view` of `data` in the current style: the one place the server, the Raspberry Pi
    /// app and the standalone firmware decide what each page shows.
    ///
    /// Today's and the week's pages put a bird new for the year first; the rarest page shows
    /// today's birds rarest first, at most six; the story and season pages fall back to
    /// today's birds while their facts are missing.
    pub fn draw_view(
        &mut self,
        view: PageView,
        data: &ViewData<'_>,
        resolution: Resolution,
    ) -> Canvas {
        let words: PageText = self.style.words.clone();
        let new_bird: Option<&SpeciesSummary> = data.new_today.first();
        let new_heading: Option<String> =
            new_bird.map(|row| words.new_for_year.replace("{name}", &label(row)));
        let featured: Option<&ScientificName> = new_bird.map(|row| &row.scientific_name);
        match view {
            PageView::Today => self.draw_featured(
                data.today,
                featured,
                new_heading.as_deref(),
                data.footer,
                resolution,
            ),
            PageView::Week => self.draw_featured(
                data.week,
                featured,
                Some(&words.heading_week),
                data.footer,
                resolution,
            ),
            PageView::Rarest => {
                let rarest: Vec<SpeciesSummary> = match data.year {
                    Some(year) => year.almanac.rarest(data.today),
                    None => data.today.iter().rev().cloned().collect(),
                };
                let most: usize = match self.style.max_birds {
                    0 => RAREST_BIRDS,
                    limit => limit.min(RAREST_BIRDS),
                };
                let shown: &[SpeciesSummary] = &rarest[..rarest.len().min(most)];
                let rarest_name: Option<ScientificName> =
                    shown.first().map(|row| row.scientific_name.clone());
                self.draw_featured(
                    shown,
                    rarest_name.as_ref(),
                    Some(&words.heading_rarest),
                    data.footer,
                    resolution,
                )
            }
            PageView::Story => match data.day {
                Some(day) => self.draw_story(day, data.footer, resolution),
                None => self.draw(data.today, data.footer, resolution),
            },
            PageView::Season => match data.year {
                Some(year) => self.draw_season(year, data.footer, resolution),
                None => self.draw(data.today, data.footer, resolution),
            },
        }
    }

    /// Draws the page saying BirdNET-Go cannot be reached, with `problem` underneath.
    pub fn draw_unreachable(&self, problem: &str, resolution: Resolution) -> Canvas {
        let heading: String = self.style.words.unreachable.clone();
        self.draw_list(&[], &heading, "", problem, resolution)
    }

    /// Returns the fact line about `bird` for the current day, or nothing when the style has
    /// no facts, there is no bird or nothing is known about it.
    #[must_use]
    pub fn fact_line(&self, bird: Option<&SpeciesSummary>) -> String {
        let (Some(bird), Some(day), true) = (bird, self.day, self.style.facts) else {
            return String::new();
        };
        let name: String = bird.scientific_name.to_string();
        let fact: Option<Fact> = self.facts.fact_of_the_day(&name, day);
        fact.map_or_else(String::new, |fact| {
            fun_fact::fun_fact(&label(bird), fact, &self.style.words)
        })
    }

    /// Returns the border the current day and style give, if any.
    #[must_use]
    pub fn border_theme(&self) -> Option<BorderTheme> {
        let date: CivilDate = CivilDate::from_days_since_epoch(self.day?);
        self.style
            .border
            .theme(date.year, date.month, date.day, self.style.southern)
    }

    /// Adds the day's border to `layout`.
    fn decorate(&self, layout: &mut PageLayout) {
        if let Some(theme) = self.border_theme() {
            layout.ornament = SeasonBorder::layout(theme, layout.resolution);
        }
    }

    fn draw_list(
        &self,
        species: &[SpeciesSummary],
        heading: &str,
        fact: &str,
        footer: &str,
        resolution: Resolution,
    ) -> Canvas {
        let options = TextListOptions {
            heading: heading.to_owned(),
            footer: footer.to_owned(),
            fact: fact.to_owned(),
            max_species: match self.style.max_birds {
                0 => 24,
                limit => limit.max(24),
            },
            empty: self.style.words.nothing_heard.clone(),
            more: self.style.words.more.clone(),
        };
        let mut layout: PageLayout = TextList::layout(species, resolution, &options);
        self.decorate(&mut layout);
        self.renderer.render(&layout, &[])
    }

    /// Draws the collage of `species`, the first drawn bigger when `featured`, with `fact`
    /// at the foot.
    fn draw_collage(
        &mut self,
        species: &[SpeciesSummary],
        featured: bool,
        heading: &str,
        fact: &str,
        resolution: Resolution,
    ) -> Canvas {
        for row in species {
            self.ensure_picture(&row.scientific_name.to_string());
        }
        // Species without a picture are left off: with names under the birds, there is nothing
        // to draw for them.
        let pictured: Vec<(&SpeciesSummary, &Picture)> = species
            .iter()
            .filter_map(|row| {
                let picture: &Picture = self
                    .pictures
                    .get(&row.scientific_name.to_string())?
                    .as_ref()?;
                Some((row, picture))
            })
            .collect();
        let masses: Vec<f32> = pictured
            .iter()
            .map(|(row, _)| self.sizes.mass_grams(&row.scientific_name.to_string()))
            .collect();
        let mut weights: Vec<f32> = size_weights(&masses);
        let first_pictured: bool = pictured
            .first()
            .zip(species.first())
            .is_some_and(|((row, _), first)| row.scientific_name == first.scientific_name);
        if let Some(weight) = weights.first_mut().filter(|_| featured && first_pictured) {
            *weight *= FEATURED_BOOST;
        }
        let mut art: Vec<ArtImage> = Vec::with_capacity(pictured.len());
        let mut birds: Vec<CollageBird> = Vec::with_capacity(pictured.len());
        for ((row, picture), weight) in pictured.into_iter().zip(weights) {
            let name: String = row.scientific_name.to_string();
            let label: String = label(row);
            birds.push(CollageBird {
                label_width_em: self.renderer.measure(&label, 1.0),
                label,
                weight,
                span_ratio: picture.art.span_ratio(),
                silhouette: picture.silhouette.clone(),
                mirrored: mirrored(&name),
            });
            art.push(picture.art.clone());
        }
        let options = CollageOptions {
            heading: heading.to_owned(),
            fact: fact.to_owned(),
            empty: self.style.words.nothing_heard.clone(),
            names: self.style.names,
            ..CollageOptions::default()
        };
        let mut layout: PageLayout = Collage::layout(&birds, resolution, &options);
        self.decorate(&mut layout);
        self.renderer.render(&layout, &art)
    }

    /// Fetches the species' picture into the cache, if it is not there yet.
    fn ensure_picture(&mut self, scientific_name: &str) {
        if self.pictures.contains_key(scientific_name) {
            return;
        }
        let picture: Option<Picture> = self.art.art_for(scientific_name).and_then(|art| {
            let alpha: Vec<u8> = art.pixels().iter().map(|pixel| pixel.alpha).collect();
            let silhouette: Silhouette = Silhouette::new(art.resolution(), &alpha)?;
            Some(Picture { art, silhouette })
        });
        self.pictures.insert(scientific_name.to_owned(), picture);
    }
}

/// The species' common name, or its scientific one when BirdNET-Go gave none.
fn label(row: &SpeciesSummary) -> String {
    if row.common_name.is_empty() {
        row.scientific_name.to_string()
    } else {
        row.common_name.clone()
    }
}

/// Whether to draw a species mirrored: decided by its name alone, for variety that does not
/// change whenever another bird arrives. FNV-1a, stable across runs.
fn mirrored(scientific_name: &str) -> bool {
    let hash: u64 = scientific_name
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    hash & 0x80 == 0
}

#[cfg(test)]
mod test {
    use art_pack::{NoArt, OutlineArt};
    use frame_layout::Border;

    use super::*;

    const FONT: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");

    fn artist<A: ArtSource>(art: A, layout: Layout) -> Artist<A> {
        let renderer: Renderer = Renderer::new(FONT).expect("the bundled font loads");
        let style = PageStyle {
            layout,
            facts: false,
            border: Border::Off,
            ..PageStyle::default()
        };
        Artist::new(renderer, art, style)
    }

    fn birds() -> Vec<SpeciesSummary> {
        SpeciesSummary::parse_list(
            br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":3},
                {"scientific_name":"Pica pica","common_name":"Magpie","count":1}]"#,
        )
        .expect("valid JSON")
    }

    #[test]
    fn lists_the_birds_when_there_is_no_art() {
        let size = Resolution::new(800, 480);
        let collage: Canvas = artist(NoArt, Layout::Collage).draw(&birds(), "Updated 07:30", size);
        let list: Canvas = artist(NoArt, Layout::List).draw(&birds(), "Updated 07:30", size);
        assert_eq!(collage, list);
        let outlines: Canvas =
            artist(OutlineArt::new(120), Layout::Collage).draw(&birds(), "Updated 07:30", size);
        assert_ne!(outlines, list);
    }

    #[test]
    fn mirroring_is_stable_and_mixed() {
        let names: [&str; 6] = [
            "Parus major",
            "Erithacus rubecula",
            "Pica pica",
            "Turdus merula",
            "Columba palumbus",
            "Passer domesticus",
        ];
        let flipped: usize = names.iter().filter(|name| mirrored(name)).count();
        assert!(flipped > 0 && flipped < names.len());
        assert_eq!(mirrored("Parus major"), mirrored("Parus major"));
    }
}
