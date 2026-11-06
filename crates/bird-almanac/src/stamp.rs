//! Reading BirdNET-Go's timestamps.

use birdnet_client::CivilDate;

/// Returns the date of a stamp such as `2026-11-06T05:12:00+01:00`.
pub(crate) fn date(stamp: Option<&str>) -> Option<CivilDate> {
    CivilDate::parse(stamp?)
}

/// Returns the local `HH:MM` of a stamp such as `2026-11-06T05:12:00+01:00` or
/// `2026-11-06 05:12:00`.
pub(crate) fn clock(stamp: &str) -> Option<&str> {
    let time: &str = stamp.get(11..16)?;
    let bytes: &[u8] = time.as_bytes();
    let shaped: bool = bytes[2] == b':'
        && [0, 1, 3, 4]
            .iter()
            .all(|&index| bytes[index].is_ascii_digit());
    shaped.then_some(time)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_both_shapes() {
        assert_eq!(clock("2026-11-06T05:12:00+01:00"), Some("05:12"));
        assert_eq!(clock("2026-11-06 23:59:01"), Some("23:59"));
        assert_eq!(clock("2026-11-06"), None);
        assert_eq!(
            date(Some("2026-11-06T05:12:00Z")),
            CivilDate::parse("2026-11-06")
        );
    }
}
