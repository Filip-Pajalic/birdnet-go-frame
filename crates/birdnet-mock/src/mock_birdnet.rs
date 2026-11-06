//! Answers BirdNET-Go API requests from the simulated garden.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use birdnet_client::{CivilDate, HttpResponse};

use crate::garden::{RESIDENTS, Resident, activity_percent, mix};
use crate::mock_weather;

const SECONDS_PER_DAY: i64 = 86_400;

/// A simulated BirdNET-Go station.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MockBirdnet {
    utc_offset_minutes: i32,
}

/// One local day of the garden: its date, its midnight in unix seconds, and how many of its
/// hours have begun.
#[derive(Clone, Copy)]
struct Day {
    date: CivilDate,
    midnight: i64,
    hours: u32,
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
    ///
    /// The summary honours `start_date` and `end_date` (today when left out), the hourly batch
    /// answers `date` and every `species`, and `/v1/forecast` answers like Open-Meteo with
    /// the garden's weather, so one mock serves the whole page.
    #[must_use]
    pub fn respond(&self, path_and_query: &str, now_unix_seconds: i64) -> HttpResponse {
        let (path, query): (&str, &str) = path_and_query
            .split_once('?')
            .unwrap_or((path_and_query, ""));
        let today: Day = self.today(now_unix_seconds);
        let date = |key: &str| query_value(query, key).and_then(CivilDate::parse);
        let body: String = match path.trim_end_matches('/') {
            "/api/v2/analytics/species/summary" => {
                let start: CivilDate = date("start_date").unwrap_or(today.date);
                let end: CivilDate = date("end_date").unwrap_or(today.date);
                self.summary(start, end, today)
            }
            "/api/v2/analytics/time/hourly/batch" => {
                let Some(day) = date("date") else {
                    return HttpResponse::new(400, br#"{"error":"date is required"}"#.to_vec());
                };
                let species: Vec<String> = query
                    .split('&')
                    .filter_map(|pair| pair.strip_prefix("species="))
                    .map(unescape)
                    .collect();
                self.hourly(day, &species, today)
            }
            "/api/v2/detections/recent" => {
                let limit: usize = query_value(query, "limit")
                    .and_then(|text| text.parse().ok())
                    .unwrap_or(10);
                self.recent(today, limit.min(1000))
            }
            "/api/v2/health" | "/api/v2/ping" => String::from(r#"{"status":"ok"}"#),
            "/v1/forecast" => mock_weather::forecast(today.date, today.hours, self.offset()),
            _ => {
                return HttpResponse::new(404, br#"{"error":"not found"}"#.to_vec());
            }
        };
        HttpResponse::new(200, body.into_bytes())
    }

    fn offset(&self) -> i64 {
        i64::from(self.utc_offset_minutes) * 60
    }

    /// Returns today, with the hours begun so far (the current one included).
    fn today(&self, now_unix_seconds: i64) -> Day {
        let local: i64 = now_unix_seconds + self.offset();
        let hour: u32 = u32::try_from(local.rem_euclid(SECONDS_PER_DAY) / 3_600).unwrap_or(0);
        Day {
            hours: hour + 1,
            ..self.day(CivilDate::from_days_since_epoch(
                local.div_euclid(SECONDS_PER_DAY),
            ))
        }
    }

    /// Returns the whole of `date`.
    fn day(&self, date: CivilDate) -> Day {
        Day {
            date,
            midnight: date.days_since_epoch() * SECONDS_PER_DAY - self.offset(),
            hours: 24,
        }
    }

    /// Returns the days from `start` to `end` that have begun by `today`.
    fn days(&self, start: CivilDate, end: CivilDate, today: Day) -> impl Iterator<Item = Day> {
        let first: i64 = start.days_since_epoch();
        let last: i64 = end.min(today.date).days_since_epoch();
        let station: Self = *self;
        (first..=last).map(move |days| {
            let date: CivilDate = CivilDate::from_days_since_epoch(days);
            if date == today.date {
                today
            } else {
                station.day(date)
            }
        })
    }

    /// Returns how many times `resident` was heard during local `hour` of `day`.
    fn calls_in_hour(resident: &Resident, index: usize, day: Day, hour: u32) -> u32 {
        if !resident.in_season(day.date.month) {
            return 0;
        }
        let day_seed: u64 = mix(day.midnight.unsigned_abs() ^ (index as u64).wrapping_mul(0x51));
        if u32::try_from(day_seed % 100).unwrap_or(0) >= resident.presence_percent {
            return 0;
        }
        let hour_seed: u64 = mix(day_seed ^ u64::from(hour));
        let base: u32 = resident.peak_per_hour * activity_percent(hour) / 100;
        // Vary each hour between half and one and a half times the expected rate.
        let jitter: u32 = u32::try_from(hour_seed % 101).unwrap_or(50);
        base * (50 + jitter) / 100
    }

    fn summary(&self, start: CivilDate, end: CivilDate, today: Day) -> String {
        let mut rows: Vec<(u32, String)> = Vec::new();
        for (index, resident) in RESIDENTS.iter().enumerate() {
            let mut count: u32 = 0;
            let mut first: Option<(Day, u32)> = None;
            let mut last: Option<(Day, u32)> = None;
            for day in self.days(start, end, today) {
                for hour in 0..day.hours {
                    let calls: u32 = Self::calls_in_hour(resident, index, day, hour);
                    if calls > 0 {
                        count += calls;
                        first.get_or_insert((day, hour));
                        last = Some((day, hour));
                    }
                }
            }
            let (Some((first_day, first_hour)), Some((last_day, last_hour))) = (first, last) else {
                continue;
            };
            rows.push((
                count,
                format!(
                    r#"{{"scientific_name":"{}","common_name":"{}","count":{count},"first_heard":"{}","last_heard":"{}","max_confidence":0.9{}}}"#,
                    resident.scientific_name,
                    resident.common_name,
                    self.stamp(first_day.date, first_hour, 5),
                    self.stamp(last_day.date, last_hour, 55),
                    index % 10,
                ),
            ));
        }
        // BirdNET-Go sends the busiest species first.
        rows.sort_by_key(|(count, _)| core::cmp::Reverse(*count));
        let mut rows: Vec<String> = rows.into_iter().map(|(_, row)| row).collect();
        // Non-bird labels are part of real responses; the client must drop them.
        rows.push(String::from(
            r#"{"scientific_name":"Engine","common_name":"Engine","count":3}"#,
        ));
        format!("[{}]", rows.join(","))
    }

    /// Answers the hourly batch: every species asked for, known to the garden or not, with
    /// all 24 hours, as BirdNET-Go does.
    fn hourly(&self, date: CivilDate, species: &[String], today: Day) -> String {
        let day: Option<Day> = self.days(date, date, today).next();
        let entries: Vec<String> = species
            .iter()
            .map(|name| {
                let resident: Option<(usize, &Resident)> = RESIDENTS
                    .iter()
                    .enumerate()
                    .find(|(_, resident)| resident.scientific_name.eq_ignore_ascii_case(name));
                let hours: Vec<String> = (0..24)
                    .map(|hour| {
                        let count: u32 = match (resident, day) {
                            (Some((index, resident)), Some(day)) if hour < day.hours => {
                                Self::calls_in_hour(resident, index, day, hour)
                            }
                            _ => 0,
                        };
                        format!(r#"{{"hour":{hour},"count":{count}}}"#)
                    })
                    .collect();
                format!(r#""{name}":[{}]"#, hours.join(","))
            })
            .collect();
        format!("{{{}}}", entries.join(","))
    }

    fn recent(&self, today: Day, limit: usize) -> String {
        let mut rows: Vec<String> = Vec::new();
        let mut id: u64 = 100_000;
        'hours: for hour in (0..today.hours).rev() {
            for (index, resident) in RESIDENTS.iter().enumerate() {
                if rows.len() >= limit {
                    break 'hours;
                }
                if Self::calls_in_hour(resident, index, today, hour) == 0 {
                    continue;
                }
                let minute: u32 =
                    u32::try_from(mix(today.midnight.unsigned_abs() ^ id) % 60).unwrap_or(0);
                id += 1;
                let local_hour: String = format!("{hour:02}:{minute:02}:00");
                rows.push(format!(
                    r#"{{"id":{id},"date":"{}","time":"{local_hour}","scientificName":"{}","commonName":"{}","confidence":0.8{},"verified":"unverified"}}"#,
                    today.date,
                    resident.scientific_name,
                    resident.common_name,
                    index % 10,
                ));
            }
        }
        format!("[{}]", rows.join(","))
    }

    fn stamp(&self, date: CivilDate, hour: u32, minute: u32) -> String {
        let sign: char = if self.utc_offset_minutes < 0 {
            '-'
        } else {
            '+'
        };
        let offset: u32 = self.utc_offset_minutes.unsigned_abs();
        format!(
            "{date}T{hour:02}:{minute:02}:00{sign}{:02}:{:02}",
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

/// Decodes a percent-encoded query value, `+` included.
fn unescape(value: &str) -> String {
    let bytes: &[u8] = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index: usize = 0;
    while index < bytes.len() {
        let decoded: Option<u8> = (bytes[index] == b'%')
            .then(|| value.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (decoded, bytes[index]) {
            (Some(byte), _) => {
                out.push(byte);
                index += 3;
            }
            (None, b'+') => {
                out.push(b' ');
                index += 1;
            }
            (None, byte) => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod test {
    use birdnet_client::{BirdnetClient, DateWindow};
    use birdnet_model::{SpeciesHours, SpeciesSummary};

    use super::*;
    use crate::mock_transport::MockTransport;

    // 2026-11-06T06:30:00Z, during the dawn chorus at UTC.
    const DAWN: i64 = 1_793_946_600;

    #[test]
    fn answers_the_paths_the_frame_reads() {
        let mock = MockBirdnet::new(0);
        for path in [
            "/api/v2/analytics/species/summary?start_date=2026-11-01",
            "/api/v2/detections/recent?limit=3",
            "/api/v2/analytics/time/hourly/batch?date=2026-11-06&species=Parus%20major",
            "/v1/forecast?latitude=59.3&longitude=18.1",
        ] {
            assert_eq!(mock.respond(path, DAWN).status, 200, "{path}");
        }
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

    #[test]
    fn longer_windows_hear_more_and_keep_the_seasons() {
        let mut client = BirdnetClient::new("mock", MockTransport::new(|| DAWN));
        let today: Vec<SpeciesSummary> = client
            .species_summary(DateWindow::days_back(DAWN, 0, 1))
            .expect("mock answers");
        let year: Vec<SpeciesSummary> = client
            .species_summary(DateWindow::year_to_date(DAWN, 0))
            .expect("mock answers");
        assert!(
            year.len() > today.len(),
            "{} vs {}",
            year.len(),
            today.len()
        );
        let named = |rows: &[SpeciesSummary], name: &str| {
            rows.iter()
                .find(|row| row.scientific_name.as_str() == name)
                .cloned()
        };
        // Swifts are about in summer only, so they are on the year list but not in November.
        let swift: SpeciesSummary = named(&year, "Apus apus").expect("heard in summer");
        assert!(
            swift
                .first_heard
                .as_deref()
                .is_some_and(|first| first.starts_with("2026-05"))
        );
        assert!(named(&today, "Apus apus").is_none());
        let tit: SpeciesSummary = named(&year, "Parus major").expect("a resident");
        assert!(
            tit.first_heard
                .as_deref()
                .is_some_and(|first| first.starts_with("2026-01-01"))
        );
    }

    #[test]
    fn hourly_counts_match_the_summary() {
        let mut client = BirdnetClient::new("mock", MockTransport::new(|| DAWN));
        let today: Vec<SpeciesSummary> = client
            .species_summary(DateWindow::days_back(DAWN, 0, 1))
            .expect("mock answers");
        let names: Vec<&str> = today
            .iter()
            .map(|row| row.scientific_name.as_str())
            .collect();
        let date: CivilDate = DateWindow::local_date(DAWN, 0);
        let hours: Vec<SpeciesHours> = client.hourly(date, &names).expect("mock answers");
        assert_eq!(hours.len(), today.len());
        for (row, day) in today.iter().zip(&hours) {
            assert_eq!(row.count, day.total(), "{}", row.common_name);
            // Nothing is heard after 06:59 yet.
            assert!(day.hours[7..].iter().all(|&count| count == 0));
        }
    }

    #[test]
    fn unescapes_query_values() {
        assert_eq!(unescape("Parus%20major"), "Parus major");
        assert_eq!(unescape("a+b%2"), "a b%2");
    }
}
