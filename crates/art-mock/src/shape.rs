//! Filled shapes the birds are made of.

use alloc::vec::Vec;

use crate::point::Point;

/// A filled region in drawing units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Shape {
    /// An ellipse turned about its centre; see [`Shape::ellipse`].
    Ellipse {
        centre: Point,
        radius_x: f32,
        radius_y: f32,
        /// Sine and cosine of the turn, computed once.
        sine: f32,
        cosine: f32,
    },
    /// A polygon, its corners in order.
    Polygon(Vec<Point>),
    /// A line with round ends.
    Capsule { from: Point, to: Point, radius: f32 },
    /// An upright rectangle.
    Rectangle { min: Point, max: Point },
}

impl Shape {
    /// An ellipse turned by `angle` radians, clockwise on screen.
    pub(crate) fn ellipse(centre: Point, radius_x: f32, radius_y: f32, angle: f32) -> Self {
        Self::Ellipse {
            centre,
            radius_x,
            radius_y,
            sine: libm::sinf(angle),
            cosine: libm::cosf(angle),
        }
    }

    pub(crate) fn circle(centre: Point, radius: f32) -> Self {
        Self::ellipse(centre, radius, radius, 0.0)
    }

    /// Returns whether `point` lies inside the shape grown outwards by `grow` units.
    pub(crate) fn contains(&self, point: Point, grow: f32) -> bool {
        match self {
            Self::Ellipse {
                centre,
                radius_x,
                radius_y,
                sine,
                cosine,
            } => {
                let offset: Point = point.minus(*centre);
                // Turned back by the ellipse's angle.
                let local = Point::new(
                    offset.x * cosine + offset.y * sine,
                    offset.y * cosine - offset.x * sine,
                );
                let x: f32 = local.x / (radius_x + grow);
                let y: f32 = local.y / (radius_y + grow);
                x * x + y * y <= 1.0
            }
            Self::Polygon(corners) => {
                inside_polygon(corners, point)
                    || (grow > 0.0 && distance_to_outline(corners, point) <= grow)
            }
            Self::Capsule { from, to, radius } => {
                distance_to_segment(point, *from, *to) <= radius + grow
            }
            Self::Rectangle { min, max } => {
                point.x >= min.x - grow
                    && point.x <= max.x + grow
                    && point.y >= min.y - grow
                    && point.y <= max.y + grow
            }
        }
    }

    /// Returns a box that holds the shape grown by `grow`.
    pub(crate) fn bounds(&self, grow: f32) -> (Point, Point) {
        let (min, max): (Point, Point) = match self {
            Self::Ellipse {
                centre,
                radius_x,
                radius_y,
                ..
            } => {
                let reach: f32 = radius_x.max(*radius_y);
                (
                    Point::new(centre.x - reach, centre.y - reach),
                    Point::new(centre.x + reach, centre.y + reach),
                )
            }
            Self::Polygon(corners) => corners.iter().fold(
                (
                    Point::new(f32::MAX, f32::MAX),
                    Point::new(f32::MIN, f32::MIN),
                ),
                |(min, max), corner| {
                    (
                        Point::new(min.x.min(corner.x), min.y.min(corner.y)),
                        Point::new(max.x.max(corner.x), max.y.max(corner.y)),
                    )
                },
            ),
            Self::Capsule { from, to, radius } => (
                Point::new(from.x.min(to.x) - radius, from.y.min(to.y) - radius),
                Point::new(from.x.max(to.x) + radius, from.y.max(to.y) + radius),
            ),
            Self::Rectangle { min, max } => (*min, *max),
        };
        (
            Point::new(min.x - grow, min.y - grow),
            Point::new(max.x + grow, max.y + grow),
        )
    }
}

fn inside_polygon(corners: &[Point], point: Point) -> bool {
    let mut inside: bool = false;
    let mut previous: Point = match corners.last() {
        Some(corner) => *corner,
        None => return false,
    };
    for &corner in corners {
        if (corner.y > point.y) != (previous.y > point.y) {
            let crossing: f32 =
                corner.x + (point.y - corner.y) / (previous.y - corner.y) * (previous.x - corner.x);
            if point.x < crossing {
                inside = !inside;
            }
        }
        previous = corner;
    }
    inside
}

fn distance_to_outline(corners: &[Point], point: Point) -> f32 {
    let mut nearest: f32 = f32::MAX;
    let mut previous: Point = match corners.last() {
        Some(corner) => *corner,
        None => return nearest,
    };
    for &corner in corners {
        nearest = nearest.min(distance_to_segment(point, previous, corner));
        previous = corner;
    }
    nearest
}

fn distance_to_segment(point: Point, from: Point, to: Point) -> f32 {
    let along: Point = to.minus(from);
    let length_squared: f32 = along.dot(along);
    let fraction: f32 = if length_squared > 0.0 {
        (point.minus(from).dot(along) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    point.minus(from.plus(along.scaled(fraction))).length()
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::*;

    #[test]
    fn turned_ellipse() {
        let shape = Shape::ellipse(Point::new(1.0, 1.0), 0.5, 0.1, core::f32::consts::FRAC_PI_2);
        assert!(shape.contains(Point::new(1.0, 1.4), 0.0));
        assert!(!shape.contains(Point::new(1.4, 1.0), 0.0));
        assert!(shape.contains(Point::new(1.15, 1.0), 0.1));
    }

    #[test]
    fn polygon_and_its_outline() {
        let triangle = Shape::Polygon(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ]);
        assert!(triangle.contains(Point::new(0.2, 0.2), 0.0));
        assert!(!triangle.contains(Point::new(0.8, 0.8), 0.0));
        assert!(triangle.contains(Point::new(0.5, -0.05), 0.1));
    }

    #[test]
    fn capsule_and_rectangle() {
        let leg = Shape::Capsule {
            from: Point::new(0.0, 0.0),
            to: Point::new(0.0, 1.0),
            radius: 0.1,
        };
        assert!(leg.contains(Point::new(0.05, 1.05), 0.0));
        assert!(!leg.contains(Point::new(0.2, 0.5), 0.0));
        let band = Shape::Rectangle {
            min: Point::new(0.0, 0.0),
            max: Point::new(1.0, 0.2),
        };
        assert!(band.contains(Point::new(0.5, 0.1), 0.0));
        assert!(!band.contains(Point::new(0.5, 0.3), 0.0));
    }
}
