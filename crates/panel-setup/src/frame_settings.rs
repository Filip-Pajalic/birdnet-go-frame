//! What a frame needs to know to reach its server, as kept in flash.

use alloc::string::String;
use alloc::vec::Vec;
use core::net::Ipv4Addr;

use frame_core::{Crc32, Rotation};
use panel_wake::FrameAddress;

use crate::form;
use crate::settings_error::SettingsError;

/// The bytes a record starts with: a name and the layout's version.
const MAGIC: &[u8; 4] = b"BFS1";
/// The byte that says "no rotation": the server decides.
const NO_ROTATION: u8 = 0xff;
/// The longest Wi-Fi name, in bytes.
const SSID_MAX: usize = 32;
/// The shortest and longest WPA2 password, in bytes.
const PASSWORD_MIN: usize = 8;
const PASSWORD_MAX: usize = 63;
/// The longest frame name.
const FRAME_MAX: usize = 32;

/// A frame's settings: its Wi-Fi, its server, how it is turned and what it is called.
///
/// Set on the setup page, kept in flash as a checksummed record, read back every wake.
///
/// ```
/// use panel_setup::FrameSettings;
///
/// let settings: FrameSettings = FrameSettings::from_form(
///     b"ssid=Garden&password=correct+horse&server=192.168.1.20%3A8081&rotation=90&frame=hall",
///     None,
/// )
/// .expect("valid settings");
/// assert_eq!(settings.ssid, "Garden");
/// assert_eq!(settings.server.port(), 8081);
/// let record: Vec<u8> = settings.to_record();
/// assert_eq!(FrameSettings::from_record(&record), Some(settings));
/// assert_eq!(FrameSettings::from_record(&[0xff; 64]), None);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameSettings {
    /// The Wi-Fi network's name.
    pub ssid: String,
    /// The Wi-Fi password, empty for an open network.
    pub password: String,
    /// The frame server.
    pub server: FrameAddress,
    /// The rotation to ask for, or `None` to leave it to the server.
    pub rotation: Option<Rotation>,
    /// The frame's name, which picks its layout on the server; empty for the default.
    pub frame: String,
}

impl FrameSettings {
    /// Bytes a record can take at most, well within one 4 KiB flash sector.
    pub const RECORD_MAX: usize = 4 + 1 + SSID_MAX + 1 + PASSWORD_MAX + 6 + 1 + 1 + FRAME_MAX + 4;

    /// Checks settings that came from somewhere other than the form, e.g. the build.
    ///
    /// # Errors
    ///
    /// Returns the first [`SettingsError`] found.
    pub fn new(
        ssid: &str,
        password: &str,
        server: FrameAddress,
        rotation: Option<Rotation>,
        frame: &str,
    ) -> Result<Self, SettingsError> {
        let ssid: &str = ssid.trim();
        if ssid.is_empty() || ssid.len() > SSID_MAX {
            return Err(SettingsError::Ssid);
        }
        if !password.is_empty() && !(PASSWORD_MIN..=PASSWORD_MAX).contains(&password.len()) {
            return Err(SettingsError::Password);
        }
        let frame: &str = frame.trim();
        let name_ok: bool = frame.len() <= FRAME_MAX
            && frame
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if !name_ok {
            return Err(SettingsError::FrameName);
        }
        Ok(Self {
            ssid: String::from(ssid),
            password: String::from(password),
            server,
            rotation,
            frame: String::from(frame),
        })
    }

    /// Reads the setup page's form, `application/x-www-form-urlencoded`. An empty password
    /// keeps `current`'s, so the page never has to show the saved one; to join an open network,
    /// tick the form's `open` box.
    ///
    /// # Errors
    ///
    /// Returns the first [`SettingsError`] found.
    pub fn from_form(body: &[u8], current: Option<&Self>) -> Result<Self, SettingsError> {
        let mut ssid: String = String::new();
        let mut password: String = String::new();
        let mut open: bool = false;
        let mut server: String = String::new();
        let mut rotation: String = String::new();
        let mut frame: String = String::new();
        for (name, value) in form::fields(body) {
            let value: String = value.map_err(|_| SettingsError::Encoding)?;
            match name {
                "ssid" => ssid = value,
                "password" => password = value,
                "open" => open = true,
                "server" => server = value,
                "rotation" => rotation = value,
                "frame" => frame = value,
                _ => {}
            }
        }
        if password.is_empty() && !open {
            if let Some(current) = current.filter(|current| current.ssid == ssid.trim()) {
                password.clone_from(&current.password);
            }
        }
        let server: FrameAddress = server.parse().map_err(|_| SettingsError::Server)?;
        let rotation: Option<Rotation> = match rotation.trim() {
            "" | "server" => None,
            degrees => Some(degrees.parse().map_err(|_| SettingsError::Rotation)?),
        };
        Self::new(&ssid, &password, server, rotation, &frame)
    }

    /// The settings as a record for flash: a magic, the fields, and a CRC-32 of both.
    #[must_use]
    pub fn to_record(&self) -> Vec<u8> {
        let mut record: Vec<u8> = Vec::with_capacity(Self::RECORD_MAX);
        record.extend_from_slice(MAGIC);
        for text in [&self.ssid, &self.password] {
            push_text(&mut record, text);
        }
        record.extend_from_slice(&self.server.ip().octets());
        record.extend_from_slice(&self.server.port().to_le_bytes());
        record.push(self.rotation.map_or(NO_ROTATION, Rotation::quarter_turns));
        push_text(&mut record, &self.frame);
        let crc: u32 = Crc32::checksum(&record);
        record.extend_from_slice(&crc.to_le_bytes());
        record
    }

    /// Reads [`to_record`](Self::to_record)'s bytes, which may be followed by anything (the
    /// rest of the flash sector). Returns `None` for erased flash, a damaged record, or settings
    /// that no longer pass the checks.
    #[must_use]
    pub fn from_record(bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader { bytes, at: 0 };
        if reader.take(MAGIC.len())? != MAGIC {
            return None;
        }
        let ssid: &str = reader.text()?;
        let password: &str = reader.text()?;
        let ip: &[u8] = reader.take(4)?;
        let ip = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
        let port: &[u8] = reader.take(2)?;
        let port: u16 = u16::from_le_bytes([port[0], port[1]]);
        let rotation: Option<Rotation> = match reader.take(1)?[0] {
            NO_ROTATION => None,
            0 => Some(Rotation::Degrees0),
            1 => Some(Rotation::Degrees90),
            2 => Some(Rotation::Degrees180),
            3 => Some(Rotation::Degrees270),
            _ => return None,
        };
        let frame: &str = reader.text()?;
        let end: usize = reader.at;
        let crc: &[u8] = reader.take(4)?;
        if Crc32::checksum(&bytes[..end]) != u32::from_le_bytes([crc[0], crc[1], crc[2], crc[3]]) {
            return None;
        }
        Self::new(ssid, password, FrameAddress::new(ip, port), rotation, frame).ok()
    }
}

/// Appends `text` with a length byte in front. The checks keep every text under 256 bytes.
fn push_text(record: &mut Vec<u8>, text: &str) {
    let bytes: &[u8] = text.as_bytes();
    let length: usize = bytes.len().min(usize::from(u8::MAX));
    record.push(length as u8);
    record.extend_from_slice(&bytes[..length]);
}

/// Reads a record front to back.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    /// The next `count` bytes, if there are that many.
    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let taken: &[u8] = self.bytes.get(self.at..self.at.checked_add(count)?)?;
        self.at += count;
        Some(taken)
    }

    /// The next length-prefixed text.
    fn text(&mut self) -> Option<&'a str> {
        let length: usize = usize::from(self.take(1)?[0]);
        core::str::from_utf8(self.take(length)?).ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn garden() -> FrameSettings {
        FrameSettings::new(
            "Garden",
            "correct horse",
            FrameAddress::new(Ipv4Addr::new(192, 168, 1, 20), 8081),
            Some(Rotation::Degrees270),
            "hall",
        )
        .expect("valid")
    }

    #[test]
    fn records_round_trip_and_damage_reads_as_nothing() {
        let settings: FrameSettings = garden();
        let mut record: Vec<u8> = settings.to_record();
        assert!(record.len() <= FrameSettings::RECORD_MAX);
        record.extend_from_slice(&[0xff; 100]);
        assert_eq!(FrameSettings::from_record(&record), Some(settings.clone()));
        for at in [0, 5, 20, record.len() - 101] {
            let mut damaged: Vec<u8> = record.clone();
            damaged[at] ^= 0x04;
            assert_eq!(FrameSettings::from_record(&damaged), None, "{at}");
        }
        assert_eq!(FrameSettings::from_record(&record[..10]), None);
        assert_eq!(FrameSettings::from_record(&[]), None);
        let open = FrameSettings {
            password: String::new(),
            rotation: None,
            frame: String::new(),
            ..settings
        };
        assert_eq!(FrameSettings::from_record(&open.to_record()), Some(open));
    }

    #[test]
    fn the_form_keeps_the_saved_password_unless_told_otherwise() {
        let current: FrameSettings = garden();
        let body: &[u8] = b"ssid=Garden&password=&server=192.168.1.20:8081&rotation=server&frame=";
        let kept: FrameSettings = FrameSettings::from_form(body, Some(&current)).expect("valid");
        assert_eq!(kept.password, "correct horse");
        assert_eq!(kept.rotation, None);
        let opened: FrameSettings =
            FrameSettings::from_form(&[body, b"&open=on"].concat(), Some(&current)).expect("valid");
        assert_eq!(opened.password, "");
        let elsewhere: FrameSettings =
            FrameSettings::from_form(b"ssid=Cafe&server=10.0.0.2", Some(&current)).expect("valid");
        assert_eq!(
            (elsewhere.password.as_str(), elsewhere.server.port()),
            ("", 8080)
        );
    }

    #[test]
    fn the_form_says_what_is_wrong() {
        let cases: [(&[u8], SettingsError); 7] = [
            (b"ssid=&server=10.0.0.2", SettingsError::Ssid),
            (
                b"ssid=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&server=10.0.0.2",
                SettingsError::Ssid,
            ),
            (
                b"ssid=a&password=short&server=10.0.0.2",
                SettingsError::Password,
            ),
            (b"ssid=a&server=frame.local", SettingsError::Server),
            (
                b"ssid=a&server=10.0.0.2&rotation=45",
                SettingsError::Rotation,
            ),
            (
                b"ssid=a&server=10.0.0.2&frame=Hall",
                SettingsError::FrameName,
            ),
            (b"ssid=%zz&server=10.0.0.2", SettingsError::Encoding),
        ];
        for (body, error) in cases {
            assert_eq!(FrameSettings::from_form(body, None), Err(error));
        }
    }
}
