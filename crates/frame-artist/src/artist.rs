//! Draws pages: picks the layout, finds each bird's picture and size, renders.

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
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
    BorderTheme, Collage, CollageBird, CollageOptions, CornerSprays, PageLayout, SeasonPage,
    Silhouette, Spray, StoryPage, TextList, TextListOptions, size_weights,
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

impl Picture {
    /// Returns `art` with its outline, or `None` when it has no opaque pixel.
    fn of(art: ArtImage) -> Option<Self> {
        let alpha: Vec<u8> = art.pixels().iter().map(|pixel| pixel.alpha).collect();
        let silhouette: Silhouette = Silhouette::new(art.resolution(), &alpha)?;
        Some(Self { art, silhouette })
    }
}

/// Where a border's plants come from, and the pictures of the current day's.
struct Border {
    /// Finds a theme's sprays by `<theme>-top` and `<theme>-bottom`, e.g. `frost-top`.
    sprays: Box<dyn ArtSource + Send>,
    /// The current border's top-left and bottom-right sprays.
    pictures: Option<(BorderTheme, Option<Picture>, Option<Picture>)>,
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
    border: Option<Border>,
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
            border: None,
        }
    }

    /// Returns the artist with `sprays`, the plants its borders are drawn from: the pictures
    /// named `<theme>-top` and `<theme>-bottom` (e.g. `frost-top`) hang from the top-left corner
    /// and rise from the bottom-right. Without them pages have no border.
    #[must_use]
    pub fn with_sprays(mut self, sprays: Box<dyn ArtSource + Send>) -> Self {
        self.border = Some(Border {
            sprays,
            pictures: None,
        });
        self.refresh_border();
        self
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
        self.refresh_border();
    }

    /// Starts day `day` (days since 1970, in the garden's time zone): art sources may pick
    /// another plate for a species, so cached pictures are forgotten. The day also picks the
    /// fact told about a bird and the border; before the first day there is no border.
    pub fn start_day(&mut self, day: i64) {
        if self.day != Some(day) {
            self.pictures.clear();
            self.day = Some(day);
            self.refresh_border();
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
        let (sprays, art): (CornerSprays, Vec<ArtImage>) = self.sprays(0);
        let mut options = day.story_options(&self.style.words, footer);
        options.sprays = sprays;
        let layout: PageLayout = StoryPage::layout(resolution, &options);
        self.renderer.render(&layout, &art)
    }

    /// Draws the year so far: how many species, the latest arrivals, this week against the
    /// same week last year, and the year list.
    #[must_use]
    pub fn draw_season(&self, year: &YearFacts, footer: &str, resolution: Resolution) -> Canvas {
        let (sprays, art): (CornerSprays, Vec<ArtImage>) = self.sprays(0);
        let mut options = year.season_options(&self.style.words, footer);
        options.sprays = sprays;
        let layout: PageLayout = SeasonPage::layout(resolution, &options);
        self.renderer.render(&layout, &art)
    }

    /// Draws `view` of `data` in the current style: the one place the server, the Raspberry Pi
    /// app and the standalone firmware decide what each page shows.
    ///
    /// Today's and the week's pages put a bird new for the year first; the rarest page shows
    /// today's birds rarest first, at most six; the story and season pages fall back to
    /// today's birds while their facts are missing. A page with no birds of its own (a quiet
    /// day, week or year so far) shows the bird of the day from `data.known` instead, when
    /// there is one.
    pub fn draw_view(
        &mut self,
        view: PageView,
        data: &ViewData<'_>,
        resolution: Resolution,
    ) -> Canvas {
        let quiet: bool = match view {
            PageView::Today | PageView::Rarest => data.today.is_empty(),
            PageView::Week => data.week.is_empty(),
            PageView::Story => data
                .day
                .map_or(data.today.is_empty(), |day| day.species.is_empty()),
            PageView::Season => data.year.map_or(data.today.is_empty(), |year| {
                year.almanac.year_list().is_empty()
            }),
        };
        if quiet {
            if let Some(page) = self.draw_bird_of_the_day(data.known, data.footer, resolution) {
                return page;
            }
        }
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

    /// Draws one of `known` big, with a fun fact about it, under the bird of the day heading:
    /// the page for a quiet spell, a picture even on list pages. The bird changes each day, preferring birds the art source
    /// has a picture of. Returns `None` when `known` is empty.
    pub fn draw_bird_of_the_day(
        &mut self,
        known: &[SpeciesSummary],
        footer: &str,
        resolution: Resolution,
    ) -> Option<Canvas> {
        let bird: SpeciesSummary = self.bird_of_the_day(known)?;
        let heading: String = self.style.words.bird_of_the_day.clone();
        // The fact is the point of the page, so it is told whatever the style says.
        let fact: String = self.fact_about(&bird);
        let birds: [SpeciesSummary; 1] = [bird];
        let pictured: bool = self
            .pictures
            .get(&birds[0].scientific_name.to_string())
            .is_some_and(Option::is_some);
        // A single row would leave a list page as bare as an empty one, so the bird is drawn
        // whatever the layout, and listed only without a picture.
        Some(if pictured {
            self.draw_collage(&birds, true, &heading, &fact, resolution)
        } else {
            self.draw_list(&birds, &heading, &fact, footer, resolution)
        })
    }

    /// Returns the day's pick of `known`: the same all day, another the next. Birds with a
    /// picture are picked over those without.
    fn bird_of_the_day(&mut self, known: &[SpeciesSummary]) -> Option<SpeciesSummary> {
        let mut candidates: Vec<&SpeciesSummary> = known.iter().collect();
        candidates.sort_by(|left, right| {
            left.scientific_name
                .as_str()
                .cmp(right.scientific_name.as_str())
        });
        candidates.dedup_by(|left, right| left.scientific_name == right.scientific_name);
        let count: usize = candidates.len();
        if count == 0 {
            return None;
        }
        // Knuth's multiplicative hash, so the days do not walk the list in name order.
        let day: u64 = u64::from_le_bytes(self.day.unwrap_or(0).to_le_bytes());
        let mixed: u64 = day.wrapping_mul(2_654_435_761) >> 7;
        let count_u64: u64 = u64::try_from(count).unwrap_or(1);
        let start: usize = usize::try_from(mixed % count_u64).unwrap_or(0);
        let mut pictured: Option<&SpeciesSummary> = None;
        for offset in 0..count {
            let row: &SpeciesSummary = candidates[(start + offset) % count];
            let name: String = row.scientific_name.to_string();
            self.ensure_picture(&name);
            if self.pictures.get(&name).is_some_and(Option::is_some) {
                pictured = Some(row);
                break;
            }
        }
        Some(pictured.unwrap_or(candidates[start]).clone())
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
        match bird {
            Some(bird) if self.style.facts => self.fact_about(bird),
            _ => String::new(),
        }
    }

    /// Returns the fact line about `bird` for the current day, whatever the style says, or
    /// nothing before the first day or when nothing is known about it.
    fn fact_about(&self, bird: &SpeciesSummary) -> String {
        let Some(day) = self.day else {
            return String::new();
        };
        let name: String = bird.scientific_name.to_string();
        let fact: Option<Fact> = self
            .facts
            .fact_of_the_day(&name, &self.style.words.language, day);
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

    /// Fetches the current border's sprays, if the day or style changed it.
    fn refresh_border(&mut self) {
        let theme: Option<BorderTheme> = self.border_theme();
        let Some(border) = self.border.as_mut() else {
            return;
        };
        let Some(theme) = theme else {
            border.pictures = None;
            return;
        };
        if border
            .pictures
            .as_ref()
            .is_some_and(|(current, _, _)| *current == theme)
        {
            return;
        }
        let mut spray = |corner: &str| {
            let name: String = alloc::format!("{}-{corner}", theme.name());
            border.sprays.art_for(&name).and_then(Picture::of)
        };
        let (top, bottom) = (spray("top"), spray("bottom"));
        border.pictures = Some((theme, top, bottom));
    }

    /// Returns the current border's sprays, their pictures numbered from `first_art` on, and
    /// the pictures to draw them with.
    fn sprays(&self, first_art: usize) -> (CornerSprays, Vec<ArtImage>) {
        let mut sprays = CornerSprays::default();
        let mut art: Vec<ArtImage> = Vec::new();
        let Some((_, top, bottom)) = self.border.as_ref().and_then(|b| b.pictures.as_ref()) else {
            return (sprays, art);
        };
        for (picture, slot) in [
            (top, &mut sprays.top_left),
            (bottom, &mut sprays.bottom_right),
        ] {
            if let Some(picture) = picture {
                *slot = Some(Spray {
                    art: first_art + art.len(),
                    silhouette: picture.silhouette.clone(),
                });
                art.push(picture.art.clone());
            }
        }
        (sprays, art)
    }

    fn draw_list(
        &self,
        species: &[SpeciesSummary],
        heading: &str,
        fact: &str,
        footer: &str,
        resolution: Resolution,
    ) -> Canvas {
        let (sprays, art): (CornerSprays, Vec<ArtImage>) = self.sprays(0);
        let options = TextListOptions {
            heading: heading.to_owned(),
            sprays,
            footer: footer.to_owned(),
            fact: fact.to_owned(),
            max_species: match self.style.max_birds {
                0 => 24,
                limit => limit.max(24),
            },
            empty: self.style.words.nothing_heard.clone(),
            more: self.style.words.more.clone(),
        };
        let layout: PageLayout = TextList::layout(species, resolution, &options);
        self.renderer.render(&layout, &art)
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
        let (sprays, spray_art): (CornerSprays, Vec<ArtImage>) = self.sprays(art.len());
        art.extend(spray_art);
        let options = CollageOptions {
            heading: heading.to_owned(),
            fact: fact.to_owned(),
            heading_width_em: self.renderer.measure(heading, 1.0),
            fact_width_em: self.renderer.measure(fact, 1.0),
            sprays,
            empty: self.style.words.nothing_heard.clone(),
            names: self.style.names,
            ..CollageOptions::default()
        };
        let layout: PageLayout = Collage::layout(&birds, resolution, &options);
        self.renderer.render(&layout, &art)
    }

    /// Fetches the species' picture into the cache, if it is not there yet.
    fn ensure_picture(&mut self, scientific_name: &str) {
        if self.pictures.contains_key(scientific_name) {
            return;
        }
        let picture: Option<Picture> = self.art.art_for(scientific_name).and_then(Picture::of);
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
    fn a_quiet_page_shows_the_bird_of_the_day() {
        let size = Resolution::new(800, 480);
        let mut drawing = artist(OutlineArt::new(120), Layout::Collage);
        drawing.start_day(20_398);
        let known: Vec<SpeciesSummary> = birds();
        let data = ViewData {
            today: &[],
            week: &[],
            new_today: &[],
            day: None,
            year: None,
            known: &known,
            footer: "Updated 07:30",
        };
        let quiet: Canvas = drawing.draw_view(PageView::Today, &data, size);
        let empty: Canvas = drawing.draw(&[], "Updated 07:30", size);
        assert_ne!(quiet, empty);
        let bird: SpeciesSummary = drawing.bird_of_the_day(&known).expect("a bird");
        let again: SpeciesSummary = drawing.bird_of_the_day(&known).expect("a bird");
        assert_eq!(bird.scientific_name, again.scientific_name);
        for view in [
            PageView::Week,
            PageView::Rarest,
            PageView::Story,
            PageView::Season,
        ] {
            assert_eq!(drawing.draw_view(view, &data, size), quiet, "{view}");
        }
        let picked: usize = (20_398..20_418)
            .filter(|&day| {
                drawing.start_day(day);
                drawing
                    .bird_of_the_day(&known)
                    .is_some_and(|row| row.scientific_name.as_str() == "Parus major")
            })
            .count();
        assert!(
            picked > 0 && picked < 20,
            "the bird changes from day to day"
        );
    }

    #[test]
    fn a_page_with_birds_or_nothing_known_is_drawn_as_before() {
        let size = Resolution::new(800, 480);
        let mut drawing = artist(OutlineArt::new(120), Layout::Collage);
        let known: Vec<SpeciesSummary> = birds();
        let today: Vec<SpeciesSummary> = birds()[..1].to_vec();
        let busy = ViewData {
            today: &today,
            week: &[],
            new_today: &[],
            day: None,
            year: None,
            known: &known,
            footer: "",
        };
        let drawn: Canvas = drawing.draw(&today, "", size);
        assert_eq!(drawing.draw_view(PageView::Today, &busy, size), drawn);
        let unknown = ViewData {
            today: &[],
            known: &[],
            ..busy
        };
        let empty: Canvas = drawing.draw(&[], "", size);
        assert_eq!(drawing.draw_view(PageView::Today, &unknown, size), empty);
        assert!(drawing.draw_bird_of_the_day(&[], "", size).is_none());
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
