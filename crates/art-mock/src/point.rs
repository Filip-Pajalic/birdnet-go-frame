//! A position in drawing units.

/// A position in drawing units, `y` pointing down. A bird's body is one unit long.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Point {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Point {
    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns this point turned by `angle` radians about the origin (clockwise on screen).
    pub(crate) fn rotated(self, angle: f32) -> Self {
        let (sine, cosine): (f32, f32) = (libm::sinf(angle), libm::cosf(angle));
        Self::new(
            self.x * cosine - self.y * sine,
            self.x * sine + self.y * cosine,
        )
    }

    pub(crate) fn plus(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    pub(crate) fn minus(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    pub(crate) fn scaled(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }

    pub(crate) fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub(crate) fn length(self) -> f32 {
        libm::sqrtf(self.dot(self))
    }
}
