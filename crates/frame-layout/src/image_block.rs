//! A picture placed on the page.

/// Where one picture goes. The picture itself is looked up by index in the art the page is
/// drawn with, so the layout stays plain data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageBlock {
    /// Index of the picture in the art passed to the renderer.
    pub art: usize,
    /// Left edge in pixels.
    pub x: i32,
    /// Top edge in pixels.
    pub y: i32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// Whether to draw the picture mirrored left to right.
    pub mirrored: bool,
}
