//! A width and height in pixels.

/// A width and height in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Resolution {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
}

impl Resolution {
    /// Creates a resolution.
    #[must_use]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Returns the same resolution turned a quarter, with width and height swapped.
    #[must_use]
    pub const fn transposed(self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }

    /// Returns the number of pixels.
    #[must_use]
    pub const fn area(self) -> usize {
        self.width * self.height
    }
}
