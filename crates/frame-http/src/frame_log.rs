//! What each frame said when it last woke.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// The longest frame name kept; longer or oddly spelt names count as the default frame.
pub const MOST_NAME_BYTES: usize = 32;

/// The name the log files frames under when they do not give one.
pub const DEFAULT_FRAME: &str = "default";

/// Most frames remembered, so a misbehaving client cannot fill memory with names.
const MOST_FRAMES: usize = 32;

/// One frame's last wake.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameVisit {
    /// The frame's name (`frame=`), or [`DEFAULT_FRAME`].
    pub frame: String,
    /// When it last asked for its page, in unix seconds.
    pub last_wake: i64,
    /// Its battery, in millivolts (`X-Frame-Battery-mV`), if it said.
    pub battery_millivolts: Option<u32>,
    /// Its firmware's version (`X-Frame-Firmware`), if it said.
    pub firmware: Option<String>,
    /// The view it asked for (`view=`), empty for its default.
    pub view: String,
    /// Wakes seen since the server started.
    pub wakes: u64,
}

impl FrameVisit {
    /// Returns a rough charge, in percent, for a single lithium-ion cell at
    /// [`battery_millivolts`](Self::battery_millivolts): 3.3 V empty, 4.15 V full, read off
    /// a typical discharge curve. Rough, because the voltage sags under load and with age.
    #[must_use]
    pub fn battery_percent(&self) -> Option<u8> {
        /// Millivolts and percent along a typical discharge at light load.
        const CURVE: [(u32, u32); 8] = [
            (3_300, 0),
            (3_500, 5),
            (3_600, 12),
            (3_700, 35),
            (3_800, 55),
            (3_900, 70),
            (4_000, 82),
            (4_150, 100),
        ];
        let millivolts: u32 = self.battery_millivolts?;
        let percent: u32 = match CURVE.iter().position(|(knee, _)| millivolts < *knee) {
            Some(0) => 0,
            None => 100,
            Some(index) => {
                let (low_mv, low_percent) = CURVE[index - 1];
                let (high_mv, high_percent) = CURVE[index];
                low_percent
                    + (millivolts - low_mv) * (high_percent - low_percent) / (high_mv - low_mv)
            }
        };
        u8::try_from(percent).ok()
    }

    /// Returns the visit as a JSON object.
    #[must_use]
    pub fn to_json(&self) -> String {
        let number =
            |value: Option<u32>| value.map_or(String::from("null"), |value| value.to_string());
        format!(
            r#"{{"frame":{},"last_wake":{},"battery_mv":{},"battery_percent":{},"firmware":{},"view":{},"wakes":{}}}"#,
            json_string(&self.frame),
            self.last_wake,
            number(self.battery_millivolts),
            number(self.battery_percent().map(u32::from)),
            self.firmware
                .as_deref()
                .map_or(String::from("null"), json_string),
            json_string(&self.view),
            self.wakes,
        )
    }
}

/// The last wake of every frame that has asked for `/panel`, for Home Assistant and the like.
#[derive(Debug, Default)]
pub struct FrameLog {
    visits: Mutex<Vec<FrameVisit>>,
}

impl FrameLog {
    /// Creates an empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Notes that `frame` woke now, asking for `view`, with what its headers said.
    pub fn record(
        &self,
        frame: &str,
        view: &str,
        battery_millivolts: Option<u32>,
        firmware: Option<&str>,
    ) {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
            });
        self.record_at(now, frame, view, battery_millivolts, firmware);
    }

    /// Like [`record`](Self::record), at `now_unix_seconds`.
    pub fn record_at(
        &self,
        now_unix_seconds: i64,
        frame: &str,
        view: &str,
        battery_millivolts: Option<u32>,
        firmware: Option<&str>,
    ) {
        let Ok(mut visits) = self.visits.lock() else {
            return;
        };
        let frame: &str = frame_name(Some(frame));
        let index: usize = match visits.iter().position(|visit| visit.frame == frame) {
            Some(index) => index,
            None => {
                if visits.len() >= MOST_FRAMES {
                    // Forget the frame that has been quiet longest.
                    if let Some(oldest) = visits
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, visit)| visit.last_wake)
                        .map(|(index, _)| index)
                    {
                        visits.remove(oldest);
                    }
                }
                visits.push(FrameVisit {
                    frame: String::from(frame),
                    last_wake: 0,
                    battery_millivolts: None,
                    firmware: None,
                    view: String::new(),
                    wakes: 0,
                });
                visits.len() - 1
            }
        };
        let visit: &mut FrameVisit = &mut visits[index];
        visit.last_wake = now_unix_seconds;
        visit.battery_millivolts = battery_millivolts;
        visit.firmware = firmware
            .map(str::trim)
            .filter(|version| !version.is_empty() && version.len() <= MOST_NAME_BYTES)
            .map(String::from);
        visit.view = String::from(view);
        visit.wakes += 1;
    }

    /// Returns every frame's last wake, by name.
    #[must_use]
    pub fn visits(&self) -> Vec<FrameVisit> {
        let mut visits: Vec<FrameVisit> = self
            .visits
            .lock()
            .map(|visits| visits.clone())
            .unwrap_or_default();
        visits.sort_by(|left, right| left.frame.cmp(&right.frame));
        visits
    }

    /// Returns one frame's last wake.
    #[must_use]
    pub fn visit(&self, frame: &str) -> Option<FrameVisit> {
        self.visits().into_iter().find(|visit| visit.frame == frame)
    }

    /// Returns every frame's last wake as a JSON array.
    #[must_use]
    pub fn to_json(&self) -> String {
        let visits: Vec<String> = self.visits().iter().map(FrameVisit::to_json).collect();
        format!("[{}]", visits.join(","))
    }
}

/// Returns `name` if it is a frame name (`[a-z0-9-]`, 1 to [`MOST_NAME_BYTES`] bytes), else
/// [`DEFAULT_FRAME`].
#[must_use]
pub fn frame_name(name: Option<&str>) -> &str {
    match name {
        Some(name)
            if !name.is_empty()
                && name.len() <= MOST_NAME_BYTES
                && name.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                }) =>
        {
            name
        }
        _ => DEFAULT_FRAME,
    }
}

/// `text` as a JSON string literal.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            control if u32::from(control) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", u32::from(control)));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keeps_each_frames_last_wake() {
        let log = FrameLog::new();
        log.record_at(100, "hall", "", Some(3_950), Some("0.2.0"));
        log.record_at(200, "desk", "week", None, None);
        log.record_at(300, "hall", "rarest", Some(3_900), Some("0.2.0"));
        log.record_at(400, "Not A Name!", "", None, None);
        let visits: Vec<FrameVisit> = log.visits();
        let names: Vec<&str> = visits.iter().map(|visit| visit.frame.as_str()).collect();
        assert_eq!(names, ["default", "desk", "hall"]);
        let hall: FrameVisit = log.visit("hall").expect("seen");
        assert_eq!(
            (hall.last_wake, hall.wakes, hall.view.as_str()),
            (300, 2, "rarest")
        );
        assert_eq!(hall.battery_percent(), Some(70));
        assert!(log.to_json().contains(r#""frame":"hall","last_wake":300,"battery_mv":3900,"battery_percent":70,"firmware":"0.2.0""#));
    }

    #[test]
    fn reads_the_charge_off_the_curve() {
        let at = |millivolts: u32| {
            FrameVisit {
                frame: String::from("f"),
                last_wake: 0,
                battery_millivolts: Some(millivolts),
                firmware: None,
                view: String::new(),
                wakes: 1,
            }
            .battery_percent()
        };
        assert_eq!(at(3_000), Some(0));
        assert_eq!(at(3_750), Some(45));
        assert_eq!(at(4_300), Some(100));
    }

    #[test]
    fn forgets_the_quietest_frame_when_full() {
        let log = FrameLog::new();
        for index in 0..=MOST_FRAMES {
            log.record_at(
                i64::try_from(index).unwrap_or(0),
                &format!("f{index}"),
                "",
                None,
                None,
            );
        }
        assert_eq!(log.visits().len(), MOST_FRAMES);
        assert!(log.visit("f0").is_none());
    }
}
