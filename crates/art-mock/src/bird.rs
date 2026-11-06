//! Assembles a perched bird from its build and plumage.

use alloc::vec;
use alloc::vec::Vec;

use frame_core::Rgb;

use crate::build::Build;
use crate::layer::{Edge, Layer};
use crate::plumage::{Plumage, SNOW};
use crate::point::Point;
use crate::shape::Shape;

/// Outline width in drawing units.
pub(crate) const OUTLINE_WIDTH: f32 = 0.022;

const EYE: Rgb = Rgb::new(20, 18, 18);

/// Returns the layers of a bird facing right, back to front.
pub(crate) fn layers(build: &Build, plumage: &Plumage) -> Vec<Layer> {
    let tilt: f32 = -build.tilt_degrees.to_radians();
    let depth: f32 = build.body_depth;
    // Positions on the body are given level, then turned with it.
    let on_body = |x: f32, y: f32| Point::new(x, y * depth).rotated(tilt);
    let body = Shape::ellipse(Point::default(), 0.5, depth / 2.0, tilt);
    let along_body = |centre: Point, radius_x: f32, radius_y: f32, turn: f32| {
        Shape::ellipse(centre, radius_x, radius_y * depth, tilt + turn.to_radians())
    };

    let mut layers: Vec<Layer> = Vec::with_capacity(32);

    // Legs, behind everything, hanging straight down from under the body.
    let hip_y: f32 = depth * 0.34;
    for (hip_x, splay) in [(-0.02, -0.03), (0.1, 0.02)] {
        let hip: Point = Point::new(hip_x, hip_y);
        let ankle: Point = Point::new(hip_x + splay, hip_y + depth * 0.16 + build.leg_length);
        let radius: f32 = 0.022;
        layers.push(
            Layer::new(
                Shape::Capsule {
                    from: hip,
                    to: ankle,
                    radius,
                },
                plumage.legs,
            )
            .edged(Edge::Silhouette),
        );
        layers.push(
            Layer::new(
                Shape::Capsule {
                    from: Point::new(ankle.x - 0.05, ankle.y),
                    to: Point::new(ankle.x + 0.09, ankle.y),
                    radius,
                },
                plumage.legs,
            )
            .edged(Edge::Silhouette),
        );
    }

    // Tail, from the rear of the body.
    let droop: f32 = build.tail_droop_degrees.to_radians();
    let direction: Point = Point::new(-libm::cosf(droop), libm::sinf(droop));
    let across: Point = Point::new(-direction.y, direction.x);
    let root: Point = on_body(-0.36, 0.0);
    let root_half: f32 = build.tail_width * 0.42;
    let tip: Point = root.plus(direction.scaled(build.tail_length + 0.1));
    let tip_half: f32 = build.tail_width / 2.0;
    layers.push(
        Layer::new(
            Shape::Polygon(vec![
                root.plus(across.scaled(root_half)),
                tip.plus(across.scaled(tip_half)),
                tip.minus(across.scaled(tip_half)),
                root.minus(across.scaled(root_half)),
            ]),
            plumage.tail,
        )
        .edged(Edge::Silhouette),
    );

    // Body and its patches.
    layers.push(Layer::new(body.clone(), plumage.back).edged(Edge::Silhouette));
    layers.push(
        Layer::new(
            along_body(on_body(0.06, 0.2), 0.46, 0.38, 0.0),
            plumage.belly,
        )
        .clipped(body.clone()),
    );
    if let Some(colour) = plumage.breast {
        layers.push(
            Layer::new(along_body(on_body(0.3, 0.06), 0.26, 0.42, 0.0), colour)
                .clipped(body.clone()),
        );
    }
    if let Some(colour) = plumage.vent {
        layers.push(
            Layer::new(along_body(on_body(-0.34, 0.3), 0.16, 0.2, 0.0), colour)
                .clipped(body.clone()),
        );
    }
    if let Some(colour) = plumage.spots {
        let belly = along_body(on_body(0.06, 0.2), 0.46, 0.38, 0.0);
        for (x, y) in [
            (0.34, 0.0),
            (0.26, 0.14),
            (0.38, 0.22),
            (0.14, 0.06),
            (0.16, 0.26),
            (0.02, 0.18),
            (0.28, 0.34),
            (-0.06, 0.32),
        ] {
            layers.push(
                Layer::new(Shape::circle(on_body(x, y), 0.034), colour).clipped(belly.clone()),
            );
        }
    }

    // Wing folded over the body.
    let wing_centre: Point = on_body(-0.1, -0.08);
    let wing = along_body(wing_centre, 0.38, 0.3, 10.0);
    layers.push(Layer::new(wing.clone(), plumage.wing).edged(Edge::Own));
    if let Some(colour) = plumage.wing_bar {
        for offset in [0.02, 0.14] {
            layers.push(
                Layer::new(
                    along_body(wing_centre.plus(on_body(offset, 0.0)), 0.025, 0.5, -10.0),
                    colour,
                )
                .clipped(wing.clone()),
            );
        }
    }
    if let Some(colour) = plumage.shoulder {
        layers.push(
            Layer::new(along_body(on_body(0.12, -0.12), 0.16, 0.16, 10.0), colour)
                .clipped(wing.clone()),
        );
    }

    // Head, set on the front of the body.
    let radius: f32 = build.head_radius;
    let centre: Point = on_body(0.4, -0.36);
    let head = Shape::circle(centre, radius);
    let at = |x: f32, y: f32| Point::new(centre.x + x * radius, centre.y + y * radius);
    layers.push(Layer::new(head.clone(), plumage.head).edged(Edge::Silhouette));
    if let Some(colour) = plumage.bib {
        layers.push(Layer::new(
            Shape::ellipse(at(0.35, 1.0), radius * 0.34, radius * 0.6, 0.0),
            colour,
        ));
    }
    if let Some(colour) = plumage.cap {
        layers.push(
            Layer::new(
                Shape::Rectangle {
                    min: at(-1.2, -1.2),
                    max: at(1.2, -0.2),
                },
                colour,
            )
            .clipped(head.clone()),
        );
    }
    if let Some(colour) = plumage.crown_stripe {
        layers.push(
            Layer::new(
                Shape::ellipse(at(-0.05, -0.92), radius * 0.6, radius * 0.24, 0.0),
                colour,
            )
            .clipped(head.clone()),
        );
    }
    if let Some(colour) = plumage.nape {
        layers.push(
            Layer::new(Shape::circle(at(-0.95, -0.1), radius * 0.45), colour).clipped(head.clone()),
        );
    }
    if let Some(colour) = plumage.face {
        layers.push(
            Layer::new(Shape::circle(at(0.62, 0.1), radius * 0.62), colour).clipped(head.clone()),
        );
    }
    if let Some(colour) = plumage.cheek {
        layers.push(
            Layer::new(
                Shape::ellipse(at(-0.02, 0.32), radius * 0.52, radius * 0.36, 0.0),
                colour,
            )
            .clipped(head.clone()),
        );
    }
    if let Some(colour) = plumage.eye_stripe {
        layers.push(
            Layer::new(
                Shape::Rectangle {
                    min: at(-1.2, -0.22),
                    max: at(1.2, 0.02),
                },
                colour,
            )
            .clipped(head.clone()),
        );
    }

    // Beak, then the eye on top.
    let beak_root: f32 = radius * 0.78;
    layers.push(
        Layer::new(
            Shape::Polygon(vec![
                at(0.0, 0.0).plus(Point::new(beak_root, -build.beak_depth)),
                at(1.0, 0.08).plus(Point::new(build.beak_length, 0.0)),
                at(0.0, 0.0).plus(Point::new(beak_root, build.beak_depth * 1.1)),
            ]),
            plumage.beak,
        )
        .edged(Edge::Silhouette),
    );
    let eye: Point = at(0.4, -0.08);
    if let Some(colour) = plumage.eye_ring {
        layers.push(Layer::new(Shape::circle(eye, radius * 0.24), colour));
    }
    layers.push(Layer::new(Shape::circle(eye, radius * 0.16), EYE));
    layers.push(Layer::new(
        Shape::circle(
            eye.plus(Point::new(radius * 0.05, -radius * 0.06)),
            radius * 0.05,
        ),
        SNOW,
    ));
    layers
}
