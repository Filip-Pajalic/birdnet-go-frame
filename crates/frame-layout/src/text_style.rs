//! What role a piece of text plays, which decides its ink.

/// The role of a text block on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextStyle {
    /// The page heading.
    Heading,
    /// A species name.
    Name,
    /// A secondary figure such as a detection count.
    Detail,
    /// A small line at the foot of the page.
    Footer,
}
