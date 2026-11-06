//! The simulated garden's weather, answered as Open-Meteo's forecast endpoint would.

use alloc::format;
use alloc::string::String;

use birdnet_client::CivilDate;

use crate::garden::mix;

/// WMO weather codes the mock picks from: clear, partly cloudy, overcast, fog, drizzle, rain,
/// snow and showers.
const CODES: [u8; 8] = [0, 2, 3, 45, 51, 61, 71, 80];

/// Returns a forecast body for `date`, `hours_begun` hours into the local day, at a garden
/// `offset_seconds` east of UTC. The weather depends only on the date and hour.
pub(crate) fn forecast(date: CivilDate, hours_begun: u32, offset_seconds: i64) -> String {
    let days: i64 = date.days_since_epoch();
    let seed: u64 = mix(days.unsigned_abs().wrapping_mul(0x9e37));
    // A year's swing of temperature and daylight, coldest and darkest in mid January.
    let season: f32 = libm::cosf((days as f32 - 20.0) / 365.25 * core::f32::consts::TAU);
    let mean: f32 = 7.0 - 11.0 * season;
    let spread: f32 = 2.0 + (seed % 5) as f32;
    let hour: u32 = hours_begun.saturating_sub(1);
    let warmth: f32 = libm::sinf((hour as f32 - 9.0) / 24.0 * core::f32::consts::TAU);
    let now: f32 = mean + spread * warmth;
    // Day length from about 6 to 18.5 hours, as at 59 degrees north.
    let daylight_minutes: i64 = (735.0 - 375.0 * season) as i64;
    let sunrise: i64 = 12 * 60 + 10 - daylight_minutes / 2;
    let sunset: i64 = sunrise + daylight_minutes;
    let code: u8 = CODES[usize::try_from((seed >> 8) % 8).unwrap_or(0)];
    let wind: f32 = (seed >> 16) as f32 % 90.0 / 10.0;
    format!(
        r#"{{"latitude":59.33,"longitude":18.07,"utc_offset_seconds":{offset_seconds},"current":{{"time":"{date}T{hour:02}:00","interval":900,"temperature_2m":{now:.1},"weather_code":{code},"wind_speed_10m":{wind:.1}}},"daily":{{"time":["{date}"],"weather_code":[{code}],"temperature_2m_max":[{:.1}],"temperature_2m_min":[{:.1}],"sunrise":["{date}T{:02}:{:02}"],"sunset":["{date}T{:02}:{:02}"]}}}}"#,
        mean + spread,
        mean - spread,
        sunrise / 60,
        sunrise % 60,
        sunset / 60,
        sunset % 60,
    )
}
