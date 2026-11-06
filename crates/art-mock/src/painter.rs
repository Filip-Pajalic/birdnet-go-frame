//! Turns layers into a picture with transparency.

use alloc::vec::Vec;

use art_pack::{ArtImage, Rgba};
use frame_core::{Resolution, Rgb};

use crate::layer::{Edge, Layer};
use crate::point::Point;

/// The outline colour: near-black, which dithers to clean black ink.
const OUTLINE: Rgb = Rgb::new(34, 32, 30);

/// Samples per pixel along each axis; 3 gives 9 samples and smooth edges.
const SAMPLES: usize = 3;

/// Paints `layers` so the whole bird's longest side is `size_px` pixels, outlines included,
/// with `outline` drawing units of outline. Returns the picture cropped to the bird.
#[expect(
    clippy::expect_used,
    reason = "cannot fail: one pixel is pushed per position"
)]
pub(crate) fn paint(layers: &[Layer], outline: f32, size_px: usize) -> ArtImage {
    let (min, max): (Point, Point) = layers
        .iter()
        .filter(|layer| layer.edge == Edge::Silhouette)
        .map(|layer| layer.shape.bounds(outline))
        .fold(
            (
                Point::new(f32::MAX, f32::MAX),
                Point::new(f32::MIN, f32::MIN),
            ),
            |(low, high), (min, max)| {
                (
                    Point::new(low.x.min(min.x), low.y.min(min.y)),
                    Point::new(high.x.max(max.x), high.y.max(max.y)),
                )
            },
        );
    let span: f32 = (max.x - min.x).max(max.y - min.y).max(f32::EPSILON);
    let units_per_pixel: f32 = span / size_px.max(1) as f32;
    let width: usize = ((max.x - min.x) / units_per_pixel) as usize + 2;
    let height: usize = ((max.y - min.y) / units_per_pixel) as usize + 2;

    let boxes: Vec<(Point, Point)> = layers
        .iter()
        .map(|layer| layer.shape.bounds(outline))
        .collect();
    let mut pixels: Vec<Rgba> = Vec::with_capacity(width * height);
    for row in 0..height {
        for column in 0..width {
            let mut sum: [u32; 3] = [0; 3];
            let mut covered: u32 = 0;
            for sample_y in 0..SAMPLES {
                for sample_x in 0..SAMPLES {
                    let point = Point::new(
                        min.x
                            + (column as f32 + (sample_x as f32 + 0.5) / SAMPLES as f32)
                                * units_per_pixel,
                        min.y
                            + (row as f32 + (sample_y as f32 + 0.5) / SAMPLES as f32)
                                * units_per_pixel,
                    );
                    if let Some(colour) = colour_at(layers, &boxes, outline, point) {
                        sum[0] += u32::from(colour.red);
                        sum[1] += u32::from(colour.green);
                        sum[2] += u32::from(colour.blue);
                        covered += 1;
                    }
                }
            }
            pixels.push(average(sum, covered));
        }
    }
    let picture =
        ArtImage::new(Resolution::new(width, height), pixels).expect("one pixel per position");
    crop(&picture)
}

/// Returns the colour of the top-most layer at `point`, or `None` outside the bird. `boxes`
/// holds each layer's bounds, outline included, to skip layers far from the point.
fn colour_at(
    layers: &[Layer],
    boxes: &[(Point, Point)],
    outline: f32,
    point: Point,
) -> Option<Rgb> {
    let near = |(min, max): &(Point, Point)| {
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    };
    let mut colour: Option<Rgb> = None;
    if layers.iter().zip(boxes).any(|(layer, bounds)| {
        layer.edge == Edge::Silhouette && near(bounds) && layer.shape.contains(point, outline)
    }) {
        colour = Some(OUTLINE);
    }
    for (layer, bounds) in layers.iter().zip(boxes) {
        if !near(bounds) {
            continue;
        }
        if layer
            .clip
            .as_ref()
            .is_some_and(|clip| !clip.contains(point, 0.0))
        {
            continue;
        }
        if layer.shape.contains(point, 0.0) {
            colour = Some(layer.colour);
        } else if layer.edge == Edge::Own && layer.shape.contains(point, outline * 0.7) {
            colour = Some(OUTLINE);
        }
    }
    colour
}

fn average(sum: [u32; 3], covered: u32) -> Rgba {
    if covered == 0 {
        return Rgba::TRANSPARENT;
    }
    let channel = |total: u32| -> u8 { u8::try_from(total / covered).unwrap_or(u8::MAX) };
    let all: u32 = u32::try_from(SAMPLES * SAMPLES).unwrap_or(1);
    Rgba::new(
        channel(sum[0]),
        channel(sum[1]),
        channel(sum[2]),
        u8::try_from(covered * 255 / all).unwrap_or(u8::MAX),
    )
}

/// Returns `picture` without its fully transparent border.
fn crop(picture: &ArtImage) -> ArtImage {
    let Resolution { width, height } = picture.resolution();
    let opaque = |x: usize, y: usize| picture.get(x, y).alpha > 0;
    let rows: Vec<usize> = (0..height)
        .filter(|&y| (0..width).any(|x| opaque(x, y)))
        .collect();
    let columns: Vec<usize> = (0..width)
        .filter(|&x| (0..height).any(|y| opaque(x, y)))
        .collect();
    let (Some(&top), Some(&bottom), Some(&left), Some(&right)) =
        (rows.first(), rows.last(), columns.first(), columns.last())
    else {
        return picture.clone();
    };
    let mut pixels: Vec<Rgba> = Vec::with_capacity((right - left + 1) * (bottom - top + 1));
    for y in top..=bottom {
        for x in left..=right {
            pixels.push(picture.get(x, y));
        }
    }
    ArtImage::new(Resolution::new(right - left + 1, bottom - top + 1), pixels)
        .unwrap_or_else(|| picture.clone())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shape::Shape;

    #[test]
    fn paints_a_cropped_outlined_disc() {
        let layers = [
            Layer::new(Shape::circle(Point::new(0.0, 0.0), 1.0), Rgb::WHITE)
                .edged(Edge::Silhouette),
        ];
        let picture = paint(&layers, 0.1, 100);
        let Resolution { width, height } = picture.resolution();
        assert!(
            (99..=102).contains(&width) && width == height,
            "{width}x{height}"
        );
        assert_eq!(
            picture.get(width / 2, height / 2),
            Rgba::new(255, 255, 255, 255)
        );
        assert_eq!(picture.get(width / 2, 1).rgb(), OUTLINE);
        assert_eq!(picture.get(0, 0), Rgba::TRANSPARENT);
    }
}
