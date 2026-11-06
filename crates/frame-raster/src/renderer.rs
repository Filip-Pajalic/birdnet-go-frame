//! Draws text blocks with a TrueType font.

use alloc::vec::Vec;

use art_pack::ArtImage;
use fontdue::{Font, FontSettings, Metrics};
use frame_core::{Ink, Rgb};
use frame_layout::{Align, PageLayout, TextBlock, TextStyle};

use crate::canvas::Canvas;
use crate::draw_image::draw_image;
use crate::font_error::FontError;

/// Glyph coverage, of 255, from which a pixel is inked. Lower than half so the italic's
/// hairlines survive at small sizes instead of breaking letters apart.
const INK_COVERAGE: u8 = 64;

/// Draws [`PageLayout`]s with one font, on one paper colour.
pub struct Renderer {
    font: Font,
    paper: Rgb,
}

impl Renderer {
    /// Creates a renderer from a TrueType or OpenType font file.
    ///
    /// # Errors
    ///
    /// Returns [`FontError`] when the data is not a font fontdue can read.
    pub fn new(font_data: &[u8]) -> Result<Self, FontError> {
        let font: Font =
            Font::from_bytes(font_data, FontSettings::default()).map_err(FontError::new)?;
        Ok(Self {
            font,
            paper: Self::PAPER,
        })
    }

    /// The paper pages are drawn on by default: fugleramme's panel paper, a cream the six-ink
    /// dither renders cleanly.
    pub const PAPER: Rgb = Rgb::new(233, 225, 202);

    /// Returns the renderer drawing on `paper` instead of [`Self::PAPER`].
    #[must_use]
    pub fn with_paper(mut self, paper: Rgb) -> Self {
        self.paper = paper;
        self
    }

    /// Returns the paper pages are drawn on.
    #[must_use]
    pub fn paper(&self) -> Rgb {
        self.paper
    }

    /// Draws `layout` on a fresh paper-coloured canvas: its pictures first, taken from `art` by
    /// [`frame_layout::ImageBlock::art`], then its shapes, then its text. Pictures missing from `art` are
    /// skipped.
    #[must_use]
    pub fn render(&self, layout: &PageLayout, art: &[ArtImage]) -> Canvas {
        let mut canvas = Canvas::new(layout.resolution, self.paper);
        for block in &layout.images {
            if let Some(picture) = art.get(block.art) {
                draw_image(&mut canvas, block, picture);
            }
        }
        for shape in &layout.shapes {
            let colour: Rgb = shape.ink.dither_colour();
            for y in shape.y..shape.y.saturating_add(shape.height) {
                for x in shape.x..shape.x.saturating_add(shape.width) {
                    canvas.blend(x, y, colour, u8::MAX);
                }
            }
        }
        for block in &layout.text {
            self.draw(&mut canvas, block);
        }
        canvas
    }

    /// Returns the width of `text` at `size_px`, in pixels.
    #[must_use]
    pub fn measure(&self, text: &str, size_px: f32) -> f32 {
        let mut width: f32 = 0.0;
        for letter in text.chars() {
            width += self.font.metrics(letter, size_px).advance_width;
        }
        width
    }

    fn draw(&self, canvas: &mut Canvas, block: &TextBlock) {
        let natural_width: f32 = self.measure(&block.text, block.size_px);
        let size_px: f32 = match block.max_width {
            Some(limit) if natural_width > limit as f32 && natural_width > 0.0 => {
                block.size_px * limit as f32 / natural_width
            }
            _ => block.size_px,
        };
        let width: f32 = self.measure(&block.text, size_px);
        let left: f32 = match block.align {
            Align::Left => block.x as f32,
            Align::Right => block.x as f32 - width,
            Align::Centre => block.x as f32 - width / 2.0,
        };
        let colour: Rgb = ink_for(block.style).dither_colour();
        let mut pen_x: f32 = left;
        for letter in block.text.chars() {
            let (metrics, coverage): (Metrics, Vec<u8>) = self.font.rasterize(letter, size_px);
            let glyph_left: i32 = (pen_x + 0.5) as i32 + metrics.xmin;
            let glyph_top: i32 =
                block.baseline_y - i32::try_from(metrics.height).unwrap_or(0) - metrics.ymin;
            for (row_index, row) in coverage.chunks(metrics.width.max(1)).enumerate() {
                for (column_index, &alpha) in row.iter().enumerate() {
                    let x: i32 = glyph_left + i32::try_from(column_index).unwrap_or(0);
                    let y: i32 = glyph_top + i32::try_from(row_index).unwrap_or(0);
                    // Hard edges: anti-aliased grey would dither into coloured specks on
                    // a six-ink panel.
                    let ink_coverage: u8 = if alpha >= INK_COVERAGE { u8::MAX } else { 0 };
                    canvas.blend(x, y, colour, ink_coverage);
                }
            }
            pen_x += metrics.advance_width;
        }
    }
}

fn ink_for(style: TextStyle) -> Ink {
    match style {
        TextStyle::Heading | TextStyle::Name | TextStyle::Footer => Ink::Black,
        TextStyle::Detail => Ink::Red,
    }
}
