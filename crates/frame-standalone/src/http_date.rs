//! The time from an HTTP `Date` header, so a frame with no clock of its own knows the day.

/// Returns the unix time in an HTTP date such as `Fri, 06 Nov 2026 07:00:00 GMT` (RFC 9110's
/// IMF-fixdate, which every current server sends), or `None` for anything else.
#[must_use]
pub fn parse(text: &str) -> Option<i64> {
    let (_, rest) = text.trim().split_once(", ")?;
    let mut parts = rest.split(' ');
    let day: i64 = parts.next()?.parse().ok()?;
    let month: i64 = month(parts.next()?)?;
    let year: i64 = parts.next()?.parse().ok()?;
    let mut clock = parts.next()?.split(':');
    let hour: i64 = clock.next()?.parse().ok()?;
    let minute: i64 = clock.next()?.parse().ok()?;
    let second: i64 = clock.next()?.parse().ok()?;
    if parts.next()? != "GMT" || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }
    // 60 for a leap second.
    if second > 60 {
        return None;
    }
    Some(days_since_epoch(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn month(name: &str) -> Option<i64> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let index: usize = MONTHS.iter().position(|month| *month == name)?;
    i64::try_from(index + 1).ok()
}

/// Days from 1970-01-01 to a date in the proleptic Gregorian calendar (Howard Hinnant's
/// `days_from_civil`).
fn days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let year: i64 = if month <= 2 { year - 1 } else { year };
    let era: i64 = year.div_euclid(400);
    let year_of_era: i64 = year - era * 400;
    let shifted_month: i64 = (month + 9) % 12;
    let day_of_year: i64 = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era: i64 = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_imf_fixdates() {
        assert_eq!(parse("Thu, 01 Jan 1970 00:00:00 GMT"), Some(0));
        assert_eq!(parse("Fri, 06 Nov 2026 07:00:00 GMT"), Some(1_793_948_400));
        assert_eq!(parse("Tue, 29 Feb 2028 23:59:59 GMT"), Some(1_835_481_599));
    }

    #[test]
    fn refuses_other_shapes() {
        assert_eq!(parse("Friday, 06-Nov-26 07:00:00 GMT"), None);
        assert_eq!(parse("Fri, 06 Nov 2026 07:00:00 CET"), None);
        assert_eq!(parse("Fri, 06 Nov 2026 24:00:00 GMT"), None);
        assert_eq!(parse(""), None);
    }
}
