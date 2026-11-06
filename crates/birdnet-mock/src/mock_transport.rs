//! Plugs the simulated station into a client without any networking.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use birdnet_client::{HttpRequest, HttpResponse, HttpTransport, Method};

use crate::mock_birdnet::MockBirdnet;

/// The one-time code the mock's login hands out.
const CODE: &str = "mock-code";

/// An [`HttpTransport`] answered by a [`MockBirdnet`] in-process.
///
/// The URL's host is ignored; only the path and query matter. `clock` returns the current unix
/// time in seconds, so tests can pin it. [`with_password`](Self::with_password) turns on a
/// private mode that logs in the way BirdNET-Go's does.
#[derive(Clone, Debug)]
pub struct MockTransport<C> {
    station: MockBirdnet,
    clock: C,
    password: Option<String>,
    /// Every request seen, as method and path, for tests to check.
    seen: Vec<(Method, String)>,
}

impl<C: FnMut() -> i64> MockTransport<C> {
    /// Creates a transport for a station at UTC, reading the time from `clock`.
    #[must_use]
    pub fn new(clock: C) -> Self {
        Self::with_utc_offset(clock, 0)
    }

    /// Creates a transport for a station `utc_offset_minutes` east of UTC.
    #[must_use]
    pub fn with_utc_offset(clock: C, utc_offset_minutes: i32) -> Self {
        Self {
            station: MockBirdnet::new(utc_offset_minutes),
            clock,
            password: None,
            seen: Vec::new(),
        }
    }

    /// Turns on private mode: the API answers 401 until a client logs in with `password`.
    #[must_use]
    pub fn with_password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    /// The requests seen so far, as method and path.
    #[must_use]
    pub fn seen(&self) -> &[(Method, String)] {
        &self.seen
    }

    /// Answers BirdNET-Go's login steps, or `None` for any other path.
    fn login_step(
        &self,
        request: &HttpRequest,
        path: &str,
        password: &str,
    ) -> Option<HttpResponse> {
        let cookie: &str = request.header("Cookie").unwrap_or_default();
        let route: &str = path.split('?').next().unwrap_or_default();
        let response: HttpResponse = match (request.method, route) {
            (Method::Get, "/") => {
                with_cookie(HttpResponse::new(200, b"<!doctype html>".to_vec()), "guest")
            }
            (Method::Post, "/api/v2/auth/login") => {
                let expected: String = alloc::format!("\"password\":\"{password}\"");
                let body: &str = core::str::from_utf8(&request.body).unwrap_or_default();
                if cookie.contains("session=") && body.contains(&expected) {
                    // The ampersand comes escaped, as Go's JSON encoder sends it.
                    let reply: String = alloc::format!(
                        r#"{{"success":true,"message":"Login successful","redirectUrl":"/api/v2/auth/callback?code={CODE}&redirect=%2Fui%2F"}}"#
                    );
                    HttpResponse::new(200, reply.into_bytes())
                } else {
                    HttpResponse::new(
                        401,
                        br#"{"success":false,"message":"Invalid credentials"}"#.to_vec(),
                    )
                }
            }
            (Method::Get, "/api/v2/auth/callback") => {
                let query: &str = path.split_once('?').map_or("", |(_, query)| query);
                if query
                    .split('&')
                    .any(|pair| pair == alloc::format!("code={CODE}"))
                {
                    let mut redirect: HttpResponse =
                        with_cookie(HttpResponse::new(302, Vec::new()), "logged-in");
                    redirect
                        .headers
                        .push(("Location".to_string(), "/ui/".to_string()));
                    redirect
                } else {
                    HttpResponse::new(401, Vec::new())
                }
            }
            _ => return None,
        };
        Some(response)
    }
}

impl<C: FnMut() -> i64> HttpTransport for MockTransport<C> {
    type Error = core::convert::Infallible;

    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, Self::Error> {
        let url: &str = &request.url;
        let after_scheme: &str = url.split_once("://").map_or(url, |(_, rest)| rest);
        let path: &str = after_scheme
            .find('/')
            .map_or("/", |start| &after_scheme[start..]);
        self.seen.push((request.method, path.to_string()));
        if let Some(password) = self.password.clone() {
            if let Some(response) = self.login_step(request, path, &password) {
                return Ok(response);
            }
            let logged_in: bool = request
                .header("Cookie")
                .is_some_and(|cookie| cookie.split("; ").any(|pair| pair == "session=logged-in"));
            if !logged_in {
                return Ok(HttpResponse::new(
                    401,
                    br#"{"error":"authentication required"}"#.to_vec(),
                ));
            }
        }
        Ok(self.station.respond(path, (self.clock)()))
    }
}

fn with_cookie(mut response: HttpResponse, session: &str) -> HttpResponse {
    response.headers.push((
        "Set-Cookie".to_string(),
        alloc::format!("session={session}; Path=/; HttpOnly; SameSite=Lax"),
    ));
    response
}

#[cfg(test)]
mod test {
    use birdnet_client::{BirdnetClient, ClientError, DateWindow};

    use super::*;

    const NOW: i64 = 1_793_955_600;

    #[test]
    fn logs_in_to_a_private_station() {
        let transport = MockTransport::new(|| NOW).with_password("s3cret pw");
        let mut client =
            BirdnetClient::new("http://birdnet:8080", transport).with_login("admin", "s3cret pw");
        let species = client
            .species_summary(DateWindow::hours_back(NOW, 0, 24))
            .expect("logs in, then reads");
        assert!(!species.is_empty());
        // The session holds: the next read needs no second login.
        client
            .species_summary(DateWindow::hours_back(NOW, 0, 24))
            .expect("still logged in");
    }

    #[test]
    fn says_why_when_it_cannot_read() {
        let window = DateWindow::hours_back(NOW, 0, 24);
        let mut without = BirdnetClient::new(
            "http://birdnet",
            MockTransport::new(|| NOW).with_password("pw"),
        );
        assert!(matches!(
            without.species_summary(window),
            Err(ClientError::Unauthorized)
        ));
        let mut wrong = BirdnetClient::new(
            "http://birdnet",
            MockTransport::new(|| NOW).with_password("pw"),
        )
        .with_login("admin", "nope");
        assert!(matches!(
            wrong.species_summary(window),
            Err(ClientError::LoginRefused)
        ));
    }
}
