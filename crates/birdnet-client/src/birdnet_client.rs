//! The API client.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use birdnet_model::{Detection, SpeciesHours, SpeciesSummary};

use crate::civil_date::CivilDate;
use lite_json::Json;

use crate::client_error::ClientError;
use crate::date_window::DateWindow;
use crate::hourly_path;
use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::http_transport::HttpTransport;

/// Reads detections from one BirdNET-Go instance.
#[derive(Debug)]
pub struct BirdnetClient<T> {
    /// `http://host:port`, with no trailing slash.
    origin: String,
    transport: T,
    login: Option<(String, String)>,
    /// Session cookies, by name, sent with every request.
    cookies: Vec<(String, String)>,
}

impl<T: HttpTransport> BirdnetClient<T> {
    /// Creates a client for the BirdNET-Go at `base_url`, e.g. `http://192.168.1.10:8080`.
    #[must_use]
    pub fn new(base_url: &str, transport: T) -> Self {
        Self {
            origin: base_url.trim_end_matches('/').to_string(),
            transport,
            login: None,
            cookies: Vec::new(),
        }
    }

    /// Logs in with `username` and `password` when BirdNET-Go asks for it (its private mode
    /// with password login on). BirdNET-Go's password login takes any user name; `admin` is
    /// what its own tools send.
    #[must_use]
    pub fn with_login(mut self, username: &str, password: &str) -> Self {
        self.login = Some((username.to_string(), password.to_string()));
        self
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

    /// Returns the birds heard this year at unix time `now`, in a garden `utc_offset_minutes`
    /// east of UTC, or last year's while this year has none yet: the birds a page with none of
    /// its own picks a bird of the day from.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when BirdNET-Go cannot be reached, refuses the request, or
    /// sends something unreadable.
    pub fn known_species(
        &mut self,
        now: i64,
        utc_offset_minutes: i32,
    ) -> Result<Vec<SpeciesSummary>, ClientError<T::Error>> {
        let year: Vec<SpeciesSummary> =
            self.species_summary(DateWindow::year_to_date(now, utc_offset_minutes))?;
        if !year.is_empty() {
            return Ok(year);
        }
        self.species_summary(DateWindow::previous_year(now, utc_offset_minutes))
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

    /// Returns how many times each of `species` (scientific names, at most
    /// [`HOURLY_BATCH`](Self::HOURLY_BATCH) of them) was heard in each local hour of `date`
    /// (`GET /analytics/time/hourly/batch`). More names are asked for in several requests.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when BirdNET-Go cannot be reached, refuses the request, or
    /// sends something unreadable.
    pub fn hourly(
        &mut self,
        date: CivilDate,
        species: &[&str],
    ) -> Result<Vec<SpeciesHours>, ClientError<T::Error>> {
        let mut all: Vec<SpeciesHours> = Vec::with_capacity(species.len());
        for batch in species.chunks(Self::HOURLY_BATCH) {
            let path: String = hourly_path::hourly_path(date, batch);
            let body: Vec<u8> = self.fetch(&path)?;
            all.extend(SpeciesHours::parse_batch(&body).map_err(ClientError::Parse)?);
        }
        Ok(all)
    }

    /// The most species BirdNET-Go answers in one hourly batch request.
    pub const HOURLY_BATCH: usize = hourly_path::HOURLY_BATCH;

    /// GETs `/api/v2{path}`, logging in once and trying again if BirdNET-Go asks for it.
    fn fetch(&mut self, path: &str) -> Result<Vec<u8>, ClientError<T::Error>> {
        let url: String = format!("{}/api/v2{path}", self.origin);
        let mut response: HttpResponse = self.send(HttpRequest::get(&url))?;
        if matches!(response.status, 401 | 403) && self.login.is_some() {
            self.log_in()?;
            response = self.send(HttpRequest::get(&url))?;
        }
        match response.status {
            200..=299 => Ok(response.body),
            401 | 403 => Err(ClientError::Unauthorized),
            status => Err(ClientError::Status(status)),
        }
    }

    /// BirdNET-Go's password login, as its own web page does it: open a session, post the
    /// password for a one-time code, and redeem the code at the callback, which leaves the
    /// session logged in.
    fn log_in(&mut self) -> Result<(), ClientError<T::Error>> {
        let Some((username, password)) = self.login.clone() else {
            return Err(ClientError::Unauthorized);
        };
        self.cookies.clear();
        let root: String = format!("{}/", self.origin);
        self.send(HttpRequest::get(&root))?;
        let body: String = format!(
            "{{\"username\":{},\"password\":{}}}",
            json_string(&username),
            json_string(&password)
        );
        let url: String = format!("{}/api/v2/auth/login", self.origin);
        let answer: HttpResponse = self.send(HttpRequest::post_json(&url, body.into_bytes()))?;
        if !(200..=299).contains(&answer.status) {
            return Err(ClientError::LoginRefused);
        }
        let reply: Json = Json::parse(&answer.body).map_err(|_| ClientError::LoginRefused)?;
        let accepted: bool = matches!(reply.get("success"), Some(Json::Bool(true)));
        let callback: &str = reply
            .str("redirectUrl")
            .filter(|_| accepted)
            .ok_or(ClientError::LoginRefused)?;
        let callback_url: String = if callback.starts_with("http://") {
            callback.to_string()
        } else {
            format!("{}/{}", self.origin, callback.trim_start_matches('/'))
        };
        let redeemed: HttpResponse = self.send(HttpRequest::get(&callback_url))?;
        if !(200..=399).contains(&redeemed.status) {
            return Err(ClientError::LoginRefused);
        }
        Ok(())
    }

    /// Sends `request` with the session's cookies, and keeps any the response sets.
    fn send(&mut self, request: HttpRequest) -> Result<HttpResponse, ClientError<T::Error>> {
        let request: HttpRequest = if self.cookies.is_empty() {
            request
        } else {
            let jar: Vec<String> = self
                .cookies
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect();
            request.with_header("Cookie", &jar.join("; "))
        };
        let response: HttpResponse = self
            .transport
            .send(&request)
            .map_err(ClientError::Transport)?;
        for set in response.header_values("Set-Cookie") {
            let pair: &str = set.split(';').next().unwrap_or_default();
            let Some((name, value)) = pair.split_once('=') else {
                continue;
            };
            let name: &str = name.trim();
            self.cookies.retain(|(kept, _)| kept != name);
            if !value.is_empty() {
                self.cookies
                    .push((name.to_string(), value.trim().to_string()));
            }
        }
        Ok(response)
    }
}

/// `text` percent-encoded for a query value: letters, digits and `-._~` stay as they are.
pub(crate) fn query_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// `text` as a JSON string literal.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
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
    fn escapes_query_values() {
        assert_eq!(query_escape("Parus major"), "Parus%20major");
        assert_eq!(query_escape("a&b=ö"), "a%26b%3D%C3%B6");
    }

    #[test]
    fn escapes_passwords_for_json() {
        assert_eq!(json_string("p\"a\\ss\u{1}å"), "\"p\\\"a\\\\ss\\u0001å\"");
    }
}
