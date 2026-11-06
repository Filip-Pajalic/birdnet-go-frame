//! Drawing an ornament's pen marks.

use frame_core::Rgb;
use frame_layout::{PenMark, Point};

use crate::canvas::Canvas;

/// Draws `mark` on `canvas` with hard edges: a pixel whose centre the mark covers takes its
/// ink, so no grey dithers into specks.
pub(crate) fn draw_mark(canvas: &mut Canvas, mark: &PenMark) {
    match *mark {
        PenMark::Line {
            from,
            to,
            width,
            ink,
        } => {
            let reach: f32 = (width / 2.0).max(0.5);
            fill(
                canvas,
                ink.dither_colour(),
                bounds(from, to, reach),
                |pixel| distance_to_segment(pixel, from, to) <= reach,
            );
        }
        PenMark::Dot {
            centre,
            radius,
            ink,
        } => {
            fill(
                canvas,
                ink.dither_colour(),
                bounds(centre, centre, radius),
                |pixel| pixel.distance(centre) <= radius,
            );
        }
    }
}

/// The pixel box round the segment `from`–`to` widened by `reach`: left, top, right and
/// bottom, inclusive.
fn bounds(from: Point, to: Point, reach: f32) -> (i32, i32, i32, i32) {
    (
        libm::floorf(from.x.min(to.x) - reach) as i32,
        libm::floorf(from.y.min(to.y) - reach) as i32,
        libm::ceilf(from.x.max(to.x) + reach) as i32,
        libm::ceilf(from.y.max(to.y) + reach) as i32,
    )
}

/// Inks every pixel in `bounds` whose centre `covered` says is inside.
fn fill(
    canvas: &mut Canvas,
    colour: Rgb,
    (left, top, right, bottom): (i32, i32, i32, i32),
    covered: impl Fn(Point) -> bool,
) {
    let width: i32 = i32::try_from(canvas.resolution().width).unwrap_or(i32::MAX);
    let height: i32 = i32::try_from(canvas.resolution().height).unwrap_or(i32::MAX);
    for y in top.max(0)..=bottom.min(height - 1) {
        for x in left.max(0)..=right.min(width - 1) {
            if covered(Point::new(x as f32 + 0.5, y as f32 + 0.5)) {
                canvas.blend(x, y, colour, u8::MAX);
            }
        }
    }
}

/// The distance from `point` to the segment `from`–`to`.
fn distance_to_segment(point: Point, from: Point, to: Point) -> f32 {
    let along: Point = to - from;
    let length_squared: f32 = along.x * along.x + along.y * along.y;
    if length_squared <= f32::EPSILON {
        return point.distance(from);
    }
    let offset: Point = point - from;
    let t: f32 = ((offset.x * along.x + offset.y * along.y) / length_squared).clamp(0.0, 1.0);
    point.distance(from + along * t)
}

#[cfg(test)]
mod test {
    use frame_core::{Ink, Resolution};

    use super::*;

    #[test]
    fn inks_a_line_and_a_dot_with_hard_edges() {
        let paper = Rgb::new(255, 255, 255);
        let mut canvas = Canvas::new(Resolution::new(20, 20), paper);
        draw_mark(
            &mut canvas,
            &PenMark::Line {
                from: Point::new(2.0, 5.5),
                to: Point::new(17.0, 5.5),
                width: 1.0,
                ink: Ink::Blue,
            },
        );
        draw_mark(
            &mut canvas,
            &PenMark::Dot {
                centre: Point::new(10.0, 14.0),
                radius: 3.0,
                ink: Ink::Red,
            },
        );
        let at = |x: usize, y: usize| canvas.pixels()[y * 20 + x];
        assert_eq!(at(10, 5), Ink::Blue.dither_colour());
        assert_eq!(at(10, 4), paper);
        assert_eq!(at(10, 6), paper);
        assert_eq!(at(10, 14), Ink::Red.dither_colour());
        assert_eq!(at(14, 14), paper);
        let inked: usize = canvas
            .pixels()
            .iter()
            .filter(|pixel| {
                ![paper, Ink::Blue.dither_colour(), Ink::Red.dither_colour()].contains(pixel)
            })
            .count();
        assert_eq!(inked, 0, "no blended greys");
    }
}
