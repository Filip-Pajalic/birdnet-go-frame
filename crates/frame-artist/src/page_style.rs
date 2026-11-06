//! How pages are set out.

use alloc::string::String;

use frame_layout::Border;
use page_text::PageText;

use crate::layout::Layout;

/// How pages are set out, from the settings.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Whether the bird pages say something about their main bird.
    pub facts: bool,
    /// The border round every page.
    pub border: Border,
    /// Whether the garden is south of the equator, where the seasons' borders come round
    /// half a year apart.
    pub southern: bool,
}

impl Default for PageStyle {
    /// The collage, every bird, named, no heading, in English, with a fact line and the
    /// season's border.
    fn default() -> Self {
        Self {
            layout: Layout::default(),
            max_birds: 0,
            names: false,
            heading: None,
            words: PageText::default(),
            facts: true,
            border: Border::Season,
            southern: false,
        }
    }
}
