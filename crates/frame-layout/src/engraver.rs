//! Drawing ornaments the way the plates were engraved: hairline outlines, hatched colour.

use alloc::vec::Vec;
use core::f32::consts::{PI, TAU};

use frame_core::Ink;

use crate::pen_mark::PenMark;
use crate::point::Point;

/// Segments a curve is drawn with.
const CURVE_STEPS: usize = 14;

/// Collects [`PenMark`]s for an ornament, with line widths in units of a 1200-pixel page.
pub(crate) struct Engraver {
    pub(crate) marks: Vec<PenMark>,
    /// Pixels per unit.
    pub(crate) unit: f32,
}

impl Engraver {
    /// Creates an engraver for a page whose short side is `short_side` pixels.
    pub(crate) fn new(short_side: f32) -> Self {
        Self {
            marks: Vec::new(),
            unit: short_side / 1200.0,
        }
    }

    /// A hairline's width: at least a pixel, so it survives a small page.
    pub(crate) fn hair(&self, units: f32) -> f32 {
        (units * self.unit).max(1.0)
    }

    /// A straight line `units` wide.
    pub(crate) fn line(&mut self, from: Point, to: Point, units: f32, ink: Ink) {
        let width: f32 = self.hair(units);
        self.marks.push(PenMark::Line {
            from,
            to,
            width,
            ink,
        });
    }

    /// A filled dot of `radius` pixels.
    pub(crate) fn dot(&mut self, centre: Point, radius: f32, ink: Ink) {
        self.marks.push(PenMark::Dot {
            centre,
            radius: radius.max(0.6),
            ink,
        });
    }

    /// A quadratic curve from `from` to `to`, pulled towards `control`, tapering from
    /// `units` wide to `end_units`.
    pub(crate) fn curve(
        &mut self,
        from: Point,
        control: Point,
        to: Point,
        (units, end_units): (f32, f32),
        ink: Ink,
    ) {
        let mut previous: Point = from;
        for step in 1..=CURVE_STEPS {
            let t: f32 = step as f32 / CURVE_STEPS as f32;
            let point: Point = quadratic(from, control, to, t);
            self.line(previous, point, units + (end_units - units) * t, ink);
            previous = point;
        }
    }

    /// A circle's outline.
    pub(crate) fn ring(&mut self, centre: Point, radius: f32, units: f32, ink: Ink) {
        self.ellipse(centre, (radius, radius), 0.0, units, ink);
    }

    /// An ellipse's outline with half-axes `radii`, turned `angle` radians.
    pub(crate) fn ellipse(
        &mut self,
        centre: Point,
        (across, along): (f32, f32),
        angle: f32,
        units: f32,
        ink: Ink,
    ) {
        let (axis, normal): (Point, Point) = axes(angle);
        let point_at = |turn: f32| -> Point {
            centre + axis * (along * libm::cosf(turn)) + normal * (across * libm::sinf(turn))
        };
        let steps: usize = 24;
        for step in 0..steps {
            let a: f32 = TAU * step as f32 / steps as f32;
            let b: f32 = TAU * (step + 1) as f32 / steps as f32;
            self.line(point_at(a), point_at(b), units, ink);
        }
    }

    /// An egg or berry: an ellipse hatched across in `ink` and outlined in black.
    pub(crate) fn hatched_ellipse(
        &mut self,
        centre: Point,
        (across, along): (f32, f32),
        angle: f32,
        ink: Ink,
    ) {
        let (axis, normal): (Point, Point) = axes(angle);
        let lines: usize = (along * 2.0 / (3.0 * self.unit.max(0.5))).clamp(3.0, 40.0) as usize;
        for index in 1..lines {
            let t: f32 = index as f32 / lines as f32 * 2.0 - 1.0;
            let half: f32 = across * libm::sqrtf((1.0 - t * t).max(0.0)) * 0.9;
            let middle: Point = centre + axis * (along * t);
            self.line(middle - normal * half, middle + normal * half, 1.0, ink);
        }
        self.ellipse(centre, (across, along), angle, 1.2, Ink::Black);
    }

    /// A leaf or petal from `base` pointing at `angle` radians: two curved edges in black
    /// hairline, a midrib, and slanted hatching in `ink` between them. `width` is about its
    /// widest span.
    pub(crate) fn leaf(&mut self, base: Point, angle: f32, length: f32, width: f32, ink: Ink) {
        self.blade(base, angle, length, width, ink, Ink::Black);
    }

    /// Like [`leaf`](Self::leaf), outlined in `outline` instead of black.
    pub(crate) fn blade(
        &mut self,
        base: Point,
        angle: f32,
        length: f32,
        width: f32,
        ink: Ink,
        outline: Ink,
    ) {
        let (axis, normal): (Point, Point) = axes(angle);
        let tip: Point = base + axis * length;
        let middle: Point = base + axis * (length * 0.45);
        // A quadratic edge's widest point is half its control's offset.
        let left: Point = middle + normal * width;
        let right: Point = middle - normal * width;
        let hatches: usize = (length / (3.2 * self.unit.max(0.5))).clamp(3.0, 30.0) as usize;
        for index in 1..hatches {
            let t: f32 = index as f32 / hatches as f32;
            let rib: Point = base.lerp(tip, (t - 0.08).max(0.0));
            for control in [left, right] {
                let edge: Point = quadratic(base, control, tip, t);
                self.line(rib, rib.lerp(edge, 0.88), 0.9, ink);
            }
        }
        self.curve(base, left, tip, (1.3, 0.8), outline);
        self.curve(base, right, tip, (1.3, 0.8), outline);
        self.line(base, base.lerp(tip, 0.85), 0.9, outline);
    }

    /// A holly leaf: spiny edges outlined in black, hatched in green.
    pub(crate) fn holly(&mut self, base: Point, angle: f32, length: f32, width: f32) {
        let (axis, normal): (Point, Point) = axes(angle);
        let half = |t: f32| -> f32 { width * 0.5 * libm::powf(libm::sinf(PI * t), 0.7) };
        let spines: usize = 4;
        for side in [1.0_f32, -1.0] {
            let mut previous: Point = base;
            for index in 0..spines {
                let start: f32 = index as f32 / spines as f32;
                let end: f32 = (index + 1) as f32 / spines as f32;
                let notch: f32 = (start + end) / 2.0;
                let inner: Point =
                    base + axis * (length * notch) + normal * (side * half(notch) * 0.6);
                let spine: Point =
                    base + axis * (length * end) + normal * (side * half(end) * 1.25);
                self.line(previous, inner, 1.3, Ink::Black);
                self.line(inner, spine, 1.3, Ink::Black);
                previous = spine;
            }
            self.line(previous, base + axis * length, 1.3, Ink::Black);
        }
        let hatches: usize = (length / (3.0 * self.unit.max(0.5))).clamp(4.0, 30.0) as usize;
        for index in 1..hatches {
            let t: f32 = index as f32 / hatches as f32;
            let rib: Point = base + axis * (length * t);
            let reach: f32 = half(t) * 0.75;
            self.line(rib - normal * reach, rib + normal * reach, 0.9, Ink::Green);
        }
        self.line(base, base + axis * length, 1.0, Ink::Black);
    }

    /// A five-petalled flower: petals in `ink` outlined in `outline`, a yellow heart and black
    /// stamens.
    pub(crate) fn flower(&mut self, centre: Point, radius: f32, turn: f32, ink: Ink, outline: Ink) {
        for petal in 0..5 {
            let angle: f32 = turn + TAU * petal as f32 / 5.0;
            self.blade(centre, angle, radius, radius * 0.95, ink, outline);
        }
        self.dot(centre, radius * 0.28, Ink::Yellow);
        for stamen in 0..7 {
            let angle: f32 = turn + TAU * stamen as f32 / 7.0;
            self.dot(
                centre + Point::at_angle(angle) * (radius * 0.3),
                radius * 0.06,
                Ink::Black,
            );
        }
    }

    /// A daisy: many narrow white petals outlined in black around a yellow heart.
    pub(crate) fn daisy(&mut self, centre: Point, radius: f32, turn: f32) {
        for petal in 0..12 {
            let angle: f32 = turn + TAU * petal as f32 / 12.0;
            let (axis, normal): (Point, Point) = axes(angle);
            let base: Point = centre + axis * (radius * 0.25);
            let tip: Point = centre + axis * radius;
            let side: Point = normal * (radius * 0.12);
            self.curve(
                base,
                base.lerp(tip, 0.6) + side,
                tip,
                (1.0, 1.0),
                Ink::Black,
            );
            self.curve(
                base,
                base.lerp(tip, 0.6) - side,
                tip,
                (1.0, 1.0),
                Ink::Black,
            );
        }
        self.dot(centre, radius * 0.3, Ink::Yellow);
        self.ring(centre, radius * 0.3, 0.9, Ink::Black);
    }

    /// A cornflower: ragged blue florets fanned around a dark heart.
    pub(crate) fn cornflower(&mut self, centre: Point, radius: f32, turn: f32) {
        for floret in 0..8 {
            let angle: f32 = turn + TAU * floret as f32 / 8.0;
            let (axis, normal): (Point, Point) = axes(angle);
            let base: Point = centre + axis * (radius * 0.2);
            for prong in [-1.0_f32, 0.0, 1.0] {
                let tip: Point = centre + axis * radius + normal * (prong * radius * 0.18);
                self.line(base, tip, 1.6, Ink::Blue);
            }
        }
        self.dot(centre, radius * 0.22, Ink::Blue);
        self.dot(centre, radius * 0.1, Ink::Black);
    }

    /// A six-armed snow crystal in `ink`, its arms branched twice.
    pub(crate) fn snowflake(&mut self, centre: Point, radius: f32, turn: f32, ink: Ink) {
        for arm in 0..6 {
            let angle: f32 = turn + TAU * arm as f32 / 6.0;
            let tip: Point = centre + Point::at_angle(angle) * radius;
            self.line(centre, tip, 1.5, ink);
            for (along, length) in [(0.4_f32, 0.32_f32), (0.68, 0.22)] {
                let root: Point = centre.lerp(tip, along);
                for side in [-1.0_f32, 1.0] {
                    let branch: Point = Point::at_angle(angle + side * PI / 3.0);
                    self.line(root, root + branch * (radius * length), 1.1, ink);
                }
            }
        }
        self.ring(centre, radius * 0.14, 1.0, ink);
    }

    /// Frost's fern: a stem from `root` at `angle` with fronds that fern again, `depth` times.
    /// The stem's own fronds grow on `side` only (1 clockwise, -1 anticlockwise, 0 both
    /// alternately), so a fern along the page's edge grows inwards; theirs grow on both.
    pub(crate) fn ice_fern(
        &mut self,
        root: Point,
        angle: f32,
        length: f32,
        (depth, side): (u8, f32),
        ink: Ink,
    ) {
        let tip: Point = root + Point::at_angle(angle) * length;
        self.line(root, tip, 0.8 + f32::from(depth) * 0.45, ink);
        if depth == 0 || length < 4.0 * self.unit {
            return;
        }
        let fronds: usize = 6;
        for index in 1..fronds {
            let t: f32 = index as f32 / fronds as f32;
            let alternate: f32 = if index % 2 == 0 { 1.0 } else { -1.0 };
            let (turn, reach): (f32, f32) = if side == 0.0 {
                (alternate, 0.42)
            } else {
                (side, 0.3 + 0.12 * alternate)
            };
            let at: Point = root.lerp(tip, t);
            let frond: f32 = length * reach * (1.1 - t);
            self.ice_fern(at, angle + turn * 0.95, frond, (depth - 1, 0.0), ink);
        }
    }

    /// A feather hanging from `base` at `angle`: a black quill and barbs in `ink`.
    pub(crate) fn feather(&mut self, base: Point, angle: f32, length: f32, ink: Ink) {
        let (axis, normal): (Point, Point) = axes(angle);
        let tip: Point = base + axis * length;
        let barbs: usize = (length / (2.6 * self.unit.max(0.5))).clamp(4.0, 40.0) as usize;
        for index in 2..barbs {
            let t: f32 = index as f32 / barbs as f32;
            let width: f32 = length * 0.24 * libm::sinf(PI * libm::powf(t, 0.7));
            let root: Point = base.lerp(tip, t);
            for side in [1.0_f32, -1.0] {
                let end: Point = root + normal * (side * width) - axis * (width * 0.5);
                self.line(root, end, 1.1, ink);
            }
        }
        self.line(base, tip, 1.4, Ink::Black);
    }
}

/// Returns the unit vector at `angle` and the one a quarter turn from it.
fn axes(angle: f32) -> (Point, Point) {
    let axis: Point = Point::at_angle(angle);
    (axis, axis.normal())
}

/// The point `t` along the quadratic Bézier curve `from`, `control`, `to`.
pub(crate) fn quadratic(from: Point, control: Point, to: Point, t: f32) -> Point {
    let a: Point = from.lerp(control, t);
    let b: Point = control.lerp(to, t);
    a.lerp(b, t)
}
