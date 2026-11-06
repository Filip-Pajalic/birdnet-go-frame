//! Several frames per house, each with its own page: `FRAMES`.

use std::fmt;
use std::str::FromStr;

use frame_artist::{Border, Layout, PageView};
use frame_core::{PanelSize, Rotation};

/// One frame's settings. Anything a frame leaves out comes from the server's own settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameSettings {
    /// The name the frame asks with (`frame=`): `[a-z0-9-]`, at most 32 bytes.
    pub name: String,
    /// Which page to draw.
    pub layout: Layout,
    /// The page shown when the frame does not ask for one (`view=`).
    pub view: PageView,
    /// The frame's panel.
    pub panel: PanelSize,
    /// How it hangs.
    pub rotation: Rotation,
    /// Most birds on a page; 0 for every bird heard.
    pub max_birds: usize,
    /// Whether the collage names its birds.
    pub names: bool,
    /// A heading to print over the birds.
    pub heading: Option<String>,
    /// Whether the bird pages end with a line about their main bird.
    pub facts: bool,
    /// The border round the pages.
    pub border: Border,
}

/// Why `FRAMES` could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramesError(String);

impl fmt::Display for FramesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for FramesError {}

impl FrameSettings {
    /// Reads `FRAMES`: frames separated by `;`, each a name, `:`, and `key=value` settings
    /// separated by `,`, e.g. `hall:layout=collage;desk:layout=list,panel=7.3,view=story`.
    /// The keys are `layout`, `view`, `panel`, `rotation`, `max_birds`, `names`, `heading`,
    /// `facts` and `border`;
    /// a frame starts from `base`, the server's own settings.
    ///
    /// # Errors
    ///
    /// Returns [`FramesError`] naming the frame and the setting that cannot be read.
    pub fn parse_all(text: &str, base: &Self) -> Result<Vec<Self>, FramesError> {
        let mut frames: Vec<Self> = Vec::new();
        for entry in text
            .split(';')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
        {
            let (name, settings) = entry.split_once(':').unwrap_or((entry, ""));
            let name: &str = name.trim();
            if frame_http::frame_name(Some(name)) != name {
                return Err(FramesError(format!(
                    "frame name {name:?}: use a-z, 0-9 and -, at most 32"
                )));
            }
            if frames.iter().any(|frame| frame.name == name) {
                return Err(FramesError(format!("frame {name:?} is named twice")));
            }
            let mut frame = Self {
                name: String::from(name),
                ..base.clone()
            };
            for setting in settings
                .split(',')
                .map(str::trim)
                .filter(|setting| !setting.is_empty())
            {
                let (key, value) = setting.split_once('=').unwrap_or((setting, ""));
                frame
                    .set(key.trim(), value.trim())
                    .map_err(|problem| FramesError(format!("frame {name:?}: {key}: {problem}")))?;
            }
            frames.push(frame);
        }
        Ok(frames)
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        fn read<T: FromStr>(value: &str) -> Result<T, String>
        where
            T::Err: fmt::Display,
        {
            value
                .parse()
                .map_err(|error: T::Err| format!("cannot read {value:?}: {error}"))
        }
        match key {
            "layout" => self.layout = read(value)?,
            "view" => self.view = read(value)?,
            "panel" => self.panel = read(value)?,
            "rotation" => self.rotation = read(value)?,
            "max_birds" => self.max_birds = read(value)?,
            "names" => self.names = read(value)?,
            "facts" => self.facts = read(value)?,
            "border" => self.border = read(value)?,
            "heading" => {
                self.heading = Some(String::from(value)).filter(|heading| !heading.is_empty())
            }
            _ => return Err(String::from("unknown setting")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn base() -> FrameSettings {
        FrameSettings {
            name: String::from(frame_http::DEFAULT_FRAME),
            layout: Layout::Collage,
            view: PageView::Today,
            panel: PanelSize::Inches13Point3,
            rotation: Rotation::Degrees0,
            max_birds: 0,
            names: true,
            heading: None,
            facts: true,
            border: Border::Season,
        }
    }

    #[test]
    fn reads_each_frame_from_the_base() {
        let frames = FrameSettings::parse_all(
            " hall ; desk:layout=list, panel=7.3,rotation=90,view=story,max_birds=12,names=false,heading=Desk birds,facts=false,border=off",
            &base(),
        )
        .expect("valid");
        assert_eq!(frames.len(), 2);
        assert_eq!(
            frames[0],
            FrameSettings {
                name: String::from("hall"),
                ..base()
            }
        );
        let desk: &FrameSettings = &frames[1];
        assert_eq!(
            (
                desk.layout,
                desk.view,
                desk.panel,
                desk.rotation,
                desk.max_birds,
                desk.names
            ),
            (
                Layout::List,
                PageView::Story,
                PanelSize::Inches7Point3,
                Rotation::Degrees90,
                12,
                false
            )
        );
        assert_eq!(desk.heading.as_deref(), Some("Desk birds"));
        assert_eq!((desk.facts, desk.border), (false, Border::Off));
    }

    #[test]
    fn names_what_it_cannot_read() {
        for (text, needle) in [
            ("Hall", "frame name"),
            ("hall;hall", "named twice"),
            ("desk:colour=red", "unknown setting"),
            ("desk:view=month", "view"),
            ("desk:border=tinsel", "border"),
        ] {
            let error: FramesError = FrameSettings::parse_all(text, &base()).expect_err(text);
            assert!(error.to_string().contains(needle), "{text}: {error}");
        }
    }
}
