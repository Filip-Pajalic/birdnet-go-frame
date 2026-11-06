//! Answers BirdNET-Go API requests from the simulated garden.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use birdnet_client::HttpResponse;

use crate::garden::{RESIDENTS, Resident, activity_percent, mix};

const SECONDS_PER_DAY: i64 = 86_400;

/// A simulated BirdNET-Go station.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MockBirdnet {
    utc_offset_minutes: i32,
}

impl MockBirdnet {
    /// Creates a station whose garden lives `utc_offset_minutes` east of UTC, which decides
    /// when its dawn chorus is.
    #[must_use]
    pub fn new(utc_offset_minutes: i32) -> Self {
        Self { utc_offset_minutes }
    }

    /// Answers a request for `path_and_query` (everything after the host, e.g.
    /// `/api/v2/detections/recent?limit=5`) as BirdNET-Go would at `now_unix_seconds`.
    #[must_use]
    pub fn respond(&self, path_and_query: &str, now_unix_seconds: i64) -> HttpResponse {
        let (path, query): (&str, &str) = path_and_query
            .split_once('?')
            .unwrap_or((path_and_query, ""));
        let body: String = match path.trim_end_matches('/') {
            "/api/v2/analytics/species/summary" => self.summary(now_unix_seconds),
            "/api/v2/detections/recent" => {
                let limit: usize = query_value(query, "limit")
                    .and_then(|text| text.parse().ok())
                    .unwrap_or(10);
                self.recent(now_unix_seconds, limit.min(1000))
            }
            "/api/v2/health" | "/api/v2/ping" => String::from(r#"{"status":"ok"}"#),
            _ => {
                return HttpResponse::new(404, br#"{"error":"not found"}"#.to_vec());
            }
        };
        HttpResponse::new(200, body.into_bytes())
    }

    /// Returns today's local midnight and the local hour, as unix seconds and 0 to 23.
    fn local_day(&self, now_unix_seconds: i64) -> (i64, u32) {
        let offset: i64 = i64::from(self.utc_offset_minutes) * 60;
        let local: i64 = now_unix_seconds + offset;
        let midnight: i64 = local - local.rem_euclid(SECONDS_PER_DAY) - offset;
        let hour: u32 = u32::try_from(local.rem_euclid(SECONDS_PER_DAY) / 3_600).unwrap_or(0);
        (midnight, hour)
    }

    /// Returns how many times `resident` was heard during local `hour` of the day starting at
    /// `midnight`.
    fn calls_in_hour(resident: &Resident, index: usize, midnight: i64, hour: u32) -> u32 {
        let day_seed: u64 = mix(midnight.unsigned_abs() ^ (index as u64).wrapping_mul(0x51));
        if u32::try_from(day_seed % 100).unwrap_or(0) >= resident.presence_percent {
            return 0;
        }
        let hour_seed: u64 = mix(day_seed ^ u64::from(hour));
        let base: u32 = resident.peak_per_hour * activity_percent(hour) / 100;
        // Vary each hour between half and one and a half times the expected rate.
        let jitter: u32 = u32::try_from(hour_seed % 101).unwrap_or(50);
        base * (50 + jitter) / 100
    }

    fn summary(&self, now_unix_seconds: i64) -> String {
        let (midnight, hour) = self.local_day(now_unix_seconds);
        let mut rows: Vec<String> = Vec::new();
        for (index, resident) in RESIDENTS.iter().enumerate() {
            let mut count: u32 = 0;
            let mut first: Option<u32> = None;
            let mut last: Option<u32> = None;
            for past_hour in 0..=hour {
                let calls: u32 = Self::calls_in_hour(resident, index, midnight, past_hour);
                if calls > 0 {
                    count += calls;
                    first.get_or_insert(past_hour);
                    last = Some(past_hour);
                }
            }
            let (Some(first), Some(last)) = (first, last) else {
                continue;
            };
            rows.push(format!(
                r#"{{"scientific_name":"{}","common_name":"{}","count":{count},"first_heard":"{}","last_heard":"{}","max_confidence":0.9{}}}"#,
                resident.scientific_name,
                resident.common_name,
                self.stamp(midnight, first, 5),
                self.stamp(midnight, last, 55),
                index % 10,
            ));
        }
        // BirdNET-Go sends the busiest species first.
        rows.sort_by_key(|row| core::cmp::Reverse(count_of(row)));
        // Non-bird labels are part of real responses; the client must drop them.
        rows.push(String::from(
            r#"{"scientific_name":"Engine","common_name":"Engine","count":3}"#,
        ));
        format!("[{}]", rows.join(","))
    }

    fn recent(&self, now_unix_seconds: i64, limit: usize) -> String {
        let (midnight, hour) = self.local_day(now_unix_seconds);
        let mut rows: Vec<String> = Vec::new();
        let mut id: u64 = 100_000;
        'hours: for past_hour in (0..=hour).rev() {
            for (index, resident) in RESIDENTS.iter().enumerate() {
                if rows.len() >= limit {
                    break 'hours;
                }
                if Self::calls_in_hour(resident, index, midnight, past_hour) == 0 {
                    continue;
                }
                let minute: u32 =
                    u32::try_from(mix(midnight.unsigned_abs() ^ id) % 60).unwrap_or(0);
                id += 1;
                let local_hour: String = format!("{past_hour:02}:{minute:02}:00");
                rows.push(format!(
                    r#"{{"id":{id},"date":"{}","time":"{local_hour}","scientificName":"{}","commonName":"{}","confidence":0.8{},"verified":"unverified"}}"#,
                    self.date(midnight),
                    resident.scientific_name,
                    resident.common_name,
                    index % 10,
                ));
            }
        }
        format!("[{}]", rows.join(","))
    }

    fn date(&self, midnight: i64) -> String {
        let local: i64 = midnight + i64::from(self.utc_offset_minutes) * 60;
        let days: i64 = local.div_euclid(SECONDS_PER_DAY);
        format!("{}", birdnet_client::CivilDate::from_days_since_epoch(days))
    }

    fn stamp(&self, midnight: i64, hour: u32, minute: u32) -> String {
        let sign: char = if self.utc_offset_minutes < 0 {
            '-'
        } else {
            '+'
        };
        let offset: u32 = self.utc_offset_minutes.unsigned_abs();
        format!(
            "{}T{hour:02}:{minute:02}:00{sign}{:02}:{:02}",
            self.date(midnight),
            offset / 60,
            offset % 60
        )
    }
}

fn query_value<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|pair| {
        pair.split_once('=')
            .filter(|(name, _)| *name == key)
            .map(|(_, value)| value)
    })
}

fn count_of(row: &str) -> u32 {
    let Some((_, rest)) = row.split_once(r#""count":"#) else {
        return 0;
    };
    let digits: &str = rest
        .split(|letter: char| !letter.is_ascii_digit())
        .next()
        .unwrap_or("0");
    digits.parse().unwrap_or(0)
}

#[cfg(test)]
mod test {
    use super::*;

    // 2026-11-06T06:30:00Z, during the dawn chorus at UTC.
    const DAWN: i64 = 1_793_946_600;

    #[test]
    fn answers_the_summary_and_recent_paths() {
        let mock = MockBirdnet::new(0);
        assert_eq!(
            mock.respond("/api/v2/analytics/species/summary?start_date=x", DAWN)
                .status,
            200
        );
        assert_eq!(
            mock.respond("/api/v2/detections/recent?limit=3", DAWN)
                .status,
            200
        );
        assert_eq!(mock.respond("/api/v2/nope", DAWN).status, 404);
    }

    #[test]
    fn same_time_same_answer() {
        let mock = MockBirdnet::new(60);
        let path = "/api/v2/analytics/species/summary";
        assert_eq!(mock.respond(path, DAWN), mock.respond(path, DAWN));
    }

    #[test]
    fn night_before_dawn_is_quiet() {
        // 2026-11-06T02:00:00Z.
        let body = MockBirdnet::new(0)
            .respond("/api/v2/analytics/species/summary", 1_793_930_400)
            .body;
        assert_eq!(
            body,
            br#"[{"scientific_name":"Engine","common_name":"Engine","count":3}]"#.to_vec()
        );
    }
}
