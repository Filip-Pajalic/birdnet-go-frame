//! Draws a picture with transparency, scaled into its block.

use alloc::vec::Vec;
use core::ops::Range;

use art_pack::{ArtImage, Rgba};
use frame_core::Rgb;
use frame_layout::ImageBlock;

use crate::canvas::Canvas;

/// Draws `picture` over `canvas` in `block`, averaging the source pixels under each target
/// pixel so shrunken birds stay smooth.
pub(crate) fn draw_image(canvas: &mut Canvas, block: &ImageBlock, picture: &ArtImage) {
    let (Ok(width), Ok(height)) = (usize::try_from(block.width), usize::try_from(block.height))
    else {
        return;
    };
    let source = picture.resolution();
    if width == 0 || height == 0 || source.width == 0 || source.height == 0 {
        return;
    }
    let columns: Vec<Range<usize>> = (0..width).map(|x| span(x, width, source.width)).collect();
    for y in 0..height {
        let rows: Range<usize> = span(y, height, source.height);
        for (x, columns) in columns.iter().enumerate() {
            let mut premultiplied: [u32; 3] = [0; 3];
            let mut alpha: u32 = 0;
            let mut count: u32 = 0;
            for source_y in rows.clone() {
                for source_x in columns.clone() {
                    let column: usize = if block.mirrored {
                        source.width - 1 - source_x
                    } else {
                        source_x
                    };
                    let pixel: Rgba = picture.get(column, source_y);
                    let weight: u32 = u32::from(pixel.alpha);
                    premultiplied[0] += u32::from(pixel.red) * weight;
                    premultiplied[1] += u32::from(pixel.green) * weight;
                    premultiplied[2] += u32::from(pixel.blue) * weight;
                    alpha += weight;
                    count += 1;
                }
            }
            if alpha == 0 {
                continue;
            }
            let channel = |total: u32| u8::try_from(total / alpha).unwrap_or(u8::MAX);
            let colour = Rgb::new(
                channel(premultiplied[0]),
                channel(premultiplied[1]),
                channel(premultiplied[2]),
            );
            let coverage: u8 = u8::try_from(alpha / count.max(1)).unwrap_or(u8::MAX);
            canvas.blend(
                block.x + i32::try_from(x).unwrap_or(i32::MAX),
                block.y + i32::try_from(y).unwrap_or(i32::MAX),
                colour,
                coverage,
            );
        }
    }
}

/// Returns the source pixels under target pixel `index` of `target` when `source` pixels are
/// stretched over them: at least one.
fn span(index: usize, target: usize, source: usize) -> Range<usize> {
    let start: usize = index * source / target;
    let end: usize = ((index + 1) * source)
        .div_ceil(target)
        .clamp(start + 1, source);
    start.min(source - 1)..end
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use frame_core::Resolution;

    use super::*;

    fn half_red() -> ArtImage {
        // Left column red, right column transparent.
        ArtImage::new(
            Resolution::new(2, 1),
            vec![Rgba::new(255, 0, 0, 255), Rgba::TRANSPARENT],
        )
        .expect("two pixels")
    }

    #[test]
    fn scales_up_and_keeps_transparency() {
        let mut canvas = Canvas::new(Resolution::new(4, 2), Rgb::WHITE);
        let block = ImageBlock {
            art: 0,
            x: 0,
            y: 0,
            width: 4,
            height: 2,
            mirrored: false,
        };
        draw_image(&mut canvas, &block, &half_red());
        let red = Rgb::new(255, 0, 0);
        assert_eq!(canvas.pixels()[0..4], [red, red, Rgb::WHITE, Rgb::WHITE]);
    }

    #[test]
    fn mirrors_and_averages_when_shrinking() {
        let mut canvas = Canvas::new(Resolution::new(2, 1), Rgb::WHITE);
        let block = ImageBlock {
            art: 0,
            x: 1,
            y: 0,
            width: 1,
            height: 1,
            mirrored: true,
        };
        draw_image(&mut canvas, &block, &half_red());
        // Half covered by red: red blended half way over white.
        assert_eq!(canvas.pixels()[1], Rgb::new(255, 128, 128));
        assert_eq!(canvas.pixels()[0], Rgb::WHITE);
    }

    #[test]
    fn spans_cover_the_source() {
        assert_eq!(span(0, 3, 10), 0..4);
        assert_eq!(span(2, 3, 10), 6..10);
        assert_eq!(span(5, 10, 2), 1..2);
    }
}
