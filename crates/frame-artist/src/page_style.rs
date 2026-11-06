//! How pages are set out.

use alloc::string::String;

use page_text::PageText;

use crate::layout::Layout;

/// How pages are set out, from the settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageStyle {
    /// Which page to draw.
    pub layout: Layout,
    /// Most birds on a page; 0 for every bird heard.
    pub max_birds: usize,
    /// Whether the collage names its birds.
    pub names: bool,
    /// A heading to print; `None` for the collage's bare page and the list's translated one.
    pub heading: Option<String>,
    /// The page's words in the chosen language.
    pub words: PageText,
}
