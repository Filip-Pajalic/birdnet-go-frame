//! The forecast client.

use alloc::format;
use alloc::string::{String, ToString};

use birdnet_client::{HttpRequest, HttpResponse, HttpTransport};
use lite_json::Json;

use crate::weather::Weather;
use crate::weather_error::WeatherError;

/// The fields asked for: what [`Weather`] reads, in the place's own time zone, wind in m/s.
const FIELDS: &str = "current=temperature_2m,weather_code,wind_speed_10m\
&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset\
&timezone=auto&forecast_days=1&wind_speed_unit=ms";

/// Reads today's weather from Open-Meteo, or anything that answers like it.
#[derive(Debug)]
pub struct OpenMeteo<T> {
    origin: String,
    transport: T,
}

impl<T: HttpTransport> OpenMeteo<T> {
    /// The public service, over plain HTTP.
    pub const PUBLIC: &'static str = "http://api.open-meteo.com";

    /// Creates a client for the service at `base_url`, e.g. [`Self::PUBLIC`].
    #[must_use]
    pub fn new(base_url: &str, transport: T) -> Self {
        Self {
            origin: base_url.trim_end_matches('/').to_string(),
            transport,
        }
    }

    /// Returns today's weather at `latitude` and `longitude`, in degrees.
    ///
    /// # Errors
    ///
    /// Returns [`WeatherError`] when the service cannot be reached, refuses, or sends an
    /// unreadable forecast.
    pub fn today(
        &mut self,
        latitude: f32,
        longitude: f32,
    ) -> Result<Weather, WeatherError<T::Error>> {
        let url: String = format!(
            "{}/v1/forecast?latitude={latitude:.3}&longitude={longitude:.3}&{FIELDS}",
            self.origin
        );
        let response: HttpResponse = self
            .transport
            .send(&HttpRequest::get(&url))
            .map_err(WeatherError::Transport)?;
        if !(200..=299).contains(&response.status) {
            return Err(WeatherError::Status(response.status));
        }
        let document: Json = Json::parse(&response.body).map_err(|_| WeatherError::Unreadable)?;
        Weather::from_json(&document).ok_or(WeatherError::Unreadable)
    }
}
