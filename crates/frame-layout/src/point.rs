//! A point on the page.

use core::ops::{Add, Mul, Sub};

/// A point, or an offset, in pixels from the page's top-left corner.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Pixels rightwards.
    pub x: f32,
    /// Pixels downwards.
    pub y: f32,
}

impl Point {
    /// Creates a point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the unit vector at `radians` clockwise from rightwards (y grows downwards).
    #[must_use]
    pub fn at_angle(radians: f32) -> Self {
        Self::new(libm::cosf(radians), libm::sinf(radians))
    }

    /// Returns this vector turned a quarter clockwise.
    #[must_use]
    pub fn normal(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Returns the distance to `other`.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        libm::hypotf(self.x - other.x, self.y - other.y)
    }

    /// Returns the point `t` of the way from `self` to `other`.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}
