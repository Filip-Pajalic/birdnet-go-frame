//! The API client.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use birdnet_model::{Detection, SpeciesSummary};

use crate::client_error::ClientError;
use crate::date_window::DateWindow;
use crate::http_response::HttpResponse;
use crate::http_transport::HttpTransport;

/// Reads detections from one BirdNET-Go instance.
#[derive(Debug)]
pub struct BirdnetClient<T> {
    api_base: String,
    transport: T,
}

impl<T: HttpTransport> BirdnetClient<T> {
    /// Creates a client for the BirdNET-Go at `base_url`, e.g. `http://192.168.1.10:8080`.
    #[must_use]
    pub fn new(base_url: &str, transport: T) -> Self {
        Self {
            api_base: format!("{}/api/v2", base_url.trim_end_matches('/')),
            transport,
        }
    }

    /// Returns the birds heard in `window`, most detections first
    /// (`GET /analytics/species/summary`).
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when BirdNET-Go cannot be reached, refuses the request, or
    /// sends something unreadable.
    pub fn species_summary(
        &mut self,
        window: DateWindow,
    ) -> Result<Vec<SpeciesSummary>, ClientError<T::Error>> {
        let path: String = format!(
            "/analytics/species/summary?start_date={}&end_date={}",
            window.start, window.end
        );
        let body: Vec<u8> = self.fetch(&path)?;
        SpeciesSummary::parse_list(&body).map_err(ClientError::Parse)
    }

    /// Returns up to `limit` of the newest bird detections, newest first
    /// (`GET /detections/recent`).
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when BirdNET-Go cannot be reached, refuses the request, or
    /// sends something unreadable.
    pub fn recent_detections(
        &mut self,
        limit: u32,
    ) -> Result<Vec<Detection>, ClientError<T::Error>> {
        let body: Vec<u8> = self.fetch(&format!("/detections/recent?limit={limit}"))?;
        Detection::parse_list(&body).map_err(ClientError::Parse)
    }

    fn fetch(&mut self, path: &str) -> Result<Vec<u8>, ClientError<T::Error>> {
        let url: String = format!("{}{path}", self.api_base);
        let response: HttpResponse = self.transport.get(&url).map_err(ClientError::Transport)?;
        match response.status {
            200..=299 => Ok(response.body),
            401 | 403 => Err(ClientError::Unauthorized),
            status => Err(ClientError::Status(status)),
        }
    }
}
