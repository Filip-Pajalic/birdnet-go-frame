//! One coloured part of a bird.

use frame_core::Rgb;

use crate::shape::Shape;

/// How a layer is outlined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    /// No outline.
    None,
    /// Part of the bird's outer shape: the outline goes around all such layers together.
    Silhouette,
    /// Outlined on its own, like a wing folded over the body.
    Own,
}

/// A shape, its colour, and an optional shape it is clipped to.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Layer {
    pub(crate) shape: Shape,
    pub(crate) colour: Rgb,
    pub(crate) clip: Option<Shape>,
    pub(crate) edge: Edge,
}

impl Layer {
    pub(crate) fn new(shape: Shape, colour: Rgb) -> Self {
        Self {
            shape,
            colour,
            clip: None,
            edge: Edge::None,
        }
    }

    pub(crate) fn clipped(mut self, clip: Shape) -> Self {
        self.clip = Some(clip);
        self
    }

    pub(crate) fn edged(mut self, edge: Edge) -> Self {
        self.edge = edge;
        self
    }
}
