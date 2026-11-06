//! Today's weather, as read from one forecast response.

use alloc::string::String;

use lite_json::Json;

use crate::sky::Sky;

/// The weather now and over today, in the garden's own time zone.
#[derive(Clone, Debug, PartialEq)]
pub struct Weather {
    /// What the sky is doing now.
    pub sky: Sky,
    /// Air temperature now, in degrees Celsius.
    pub temperature_c: f32,
    /// Today's highest temperature, in degrees Celsius.
    pub high_c: f32,
    /// Today's lowest temperature, in degrees Celsius.
    pub low_c: f32,
    /// Wind speed now, in metres a second.
    pub wind_m_s: f32,
    /// Local sunrise, `HH:MM`.
    pub sunrise: String,
    /// Local sunset, `HH:MM`.
    pub sunset: String,
}

impl Weather {
    /// Reads a forecast response asked for with [`crate::OpenMeteo`]'s fields. Returns `None`
    /// when a field is missing.
    #[must_use]
    pub fn from_json(document: &Json) -> Option<Self> {
        let current: &Json = document.get("current")?;
        let daily: &Json = document.get("daily")?;
        let first = |key: &str| daily.get(key)?.as_array()?.first().cloned();
        let clock = |key: &str| -> Option<String> {
            let Json::String(stamp) = first(key)? else {
                return None;
            };
            Some(String::from(stamp.split_once('T')?.1.get(..5)?))
        };
        let code: f64 = current.number("weather_code")?;
        Some(Self {
            sky: Sky::from_code(code.clamp(0.0, 999.0) as u32),
            temperature_c: current.number("temperature_2m")? as f32,
            high_c: first("temperature_2m_max")?.as_number()? as f32,
            low_c: first("temperature_2m_min")?.as_number()? as f32,
            wind_m_s: current.number("wind_speed_10m").unwrap_or(0.0) as f32,
            sunrise: clock("sunrise")?,
            sunset: clock("sunset")?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// A real answer, trimmed of its units.
    const ANSWER: &[u8] = br#"{"latitude":59.3289,"utc_offset_seconds":3600,"current":{"time":"2026-11-06T11:30","interval":900,"temperature_2m":5.1,"weather_code":0,"wind_speed_10m":2.30},"daily":{"time":["2026-11-06"],"weather_code":[3],"temperature_2m_max":[6.0],"temperature_2m_min":[2.4],"sunrise":["2026-11-06T07:19"],"sunset":["2026-11-06T15:42"]}}"#;

    #[test]
    fn reads_a_forecast() {
        let document: Json = Json::parse(ANSWER).expect("JSON");
        let weather: Weather = Weather::from_json(&document).expect("complete");
        assert_eq!(weather.sky, Sky::Clear);
        assert!((weather.temperature_c - 5.1).abs() < 0.01);
        assert!((weather.high_c - 6.0).abs() < 0.01 && (weather.low_c - 2.4).abs() < 0.01);
        assert_eq!(
            (weather.sunrise.as_str(), weather.sunset.as_str()),
            ("07:19", "15:42")
        );
    }

    #[test]
    fn missing_fields_read_as_none() {
        let document: Json = Json::parse(br#"{"current":{"temperature_2m":1}}"#).expect("JSON");
        assert_eq!(Weather::from_json(&document), None);
    }
}
