//! Which page a client asks for: a panel, or a canvas of any size.

use frame_core::{PanelSize, Resolution, Rotation, e6};

/// The smallest and largest canvas side a client may ask for, in pixels.
const SIDES: core::ops::RangeInclusive<usize> = 64..=3840;
/// The most pixels a canvas may have: 4K.
const MOST_PIXELS: usize = 3840 * 2160;

/// What a page is drawn for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// An e-ink panel hung at a rotation; served as `/frame.e6` too.
    Panel {
        /// The panel.
        panel: PanelSize,
        /// How it hangs.
        rotation: Rotation,
    },
    /// Any screen or picture, as the viewer sees it; PNG only.
    Canvas(Resolution),
}

impl Target {
    /// Returns the page size as the viewer sees it.
    #[must_use]
    pub fn resolution(self) -> Resolution {
        match self {
            Self::Panel { panel, rotation } => e6::page_resolution_for(panel, rotation),
            Self::Canvas(resolution) => resolution,
        }
    }

    /// Reads a request's query (`panel=7.3&rotation=90`, or `w=1920&h=1080`), starting from
    /// `default` for anything left out. Unknown parameters are ignored.
    ///
    /// # Errors
    ///
    /// Returns a message for the client when a value cannot be read, only one of `w` and `h`
    /// is given, a canvas is too small or too large, or a panel and a canvas are both asked for.
    pub fn from_query(query: &str, default: Self) -> Result<Self, String> {
        let (mut panel, mut rotation): (Option<PanelSize>, Option<Rotation>) = (None, None);
        let (mut width, mut height): (Option<usize>, Option<usize>) = (None, None);
        for pair in query.split('&').filter(|pair| !pair.is_empty()) {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            match key {
                "panel" => panel = Some(value.parse().map_err(|error| format!("{error}"))?),
                "rotation" => {
                    rotation = Some(value.parse().map_err(|error| format!("{error}"))?);
                }
                "w" => width = Some(side("w", value)?),
                "h" => height = Some(side("h", value)?),
                _ => {}
            }
        }
        match (width, height) {
            (None, None) => {
                let (default_panel, default_rotation) = match default {
                    Self::Panel { panel, rotation } => (panel, rotation),
                    Self::Canvas(_) => (PanelSize::default(), Rotation::Degrees0),
                };
                if panel.is_none() && rotation.is_none() {
                    return Ok(default);
                }
                Ok(Self::Panel {
                    panel: panel.unwrap_or(default_panel),
                    rotation: rotation.unwrap_or(default_rotation),
                })
            }
            (Some(width), Some(height)) => {
                if panel.is_some() || rotation.is_some() {
                    return Err(String::from("ask for a panel or for w and h, not both"));
                }
                if width * height > MOST_PIXELS {
                    return Err(format!("w x h must be at most {MOST_PIXELS} pixels"));
                }
                Ok(Self::Canvas(Resolution::new(width, height)))
            }
            _ => Err(String::from("give both w and h")),
        }
    }
}

fn side(name: &str, value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|side| SIDES.contains(side))
        .ok_or_else(|| {
            format!(
                "{name} must be a number from {} to {}",
                SIDES.start(),
                SIDES.end()
            )
        })
}

#[cfg(test)]
mod test {
    use super::*;

    const DEFAULT: Target = Target::Panel {
        panel: PanelSize::Inches13Point3,
        rotation: Rotation::Degrees90,
    };

    #[test]
    fn reads_panels_and_canvases() {
        assert_eq!(Target::from_query("", DEFAULT), Ok(DEFAULT));
        assert_eq!(
            Target::from_query("panel=7.3", DEFAULT),
            Ok(Target::Panel {
                panel: PanelSize::Inches7Point3,
                rotation: Rotation::Degrees90
            })
        );
        assert_eq!(
            Target::from_query("rotation=0&x=1", DEFAULT).map(Target::resolution),
            Ok(Resolution::new(1600, 1200))
        );
        assert_eq!(
            Target::from_query("w=1920&h=1080", DEFAULT),
            Ok(Target::Canvas(Resolution::new(1920, 1080)))
        );
    }

    #[test]
    fn refuses_what_it_cannot_draw() {
        for query in [
            "panel=10",
            "rotation=45",
            "w=1920",
            "w=10&h=10",
            "w=3840&h=3840",
            "w=800&h=480&panel=7.3",
            "w=abc&h=100",
        ] {
            assert!(Target::from_query(query, DEFAULT).is_err(), "{query}");
        }
    }
}
