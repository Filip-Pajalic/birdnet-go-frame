//! Where the bird is on a plate.

/// The box around the bird on a plate, as fractions of the plate's width and height.
///
/// Plates often hold more than the bird: a branch, a tree trunk, a patch of ground. The box
/// lets a layout size the bird itself rather than the whole plate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Subject {
    /// Left edge, 0 to 1.
    pub left: f32,
    /// Top edge, 0 to 1.
    pub top: f32,
    /// Right edge, 0 to 1, more than `left`.
    pub right: f32,
    /// Bottom edge, 0 to 1, more than `top`.
    pub bottom: f32,
}

impl Subject {
    /// Returns the box if its edges are in order and inside the plate.
    #[must_use]
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Option<Self> {
        let valid: bool = (0.0..=1.0).contains(&left)
            && (0.0..=1.0).contains(&top)
            && left < right
            && right <= 1.0
            && top < bottom
            && bottom <= 1.0;
        valid.then_some(Self {
            left,
            top,
            right,
            bottom,
        })
    }
}
