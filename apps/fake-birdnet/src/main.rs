//! A stand-in BirdNET-Go on HTTP, answered by `birdnet-mock`.
//!
//! Point birdnet-go-frame (or fugleramme) at it to develop without a microphone:
//!
//! ```text
//! fake-birdnet 127.0.0.1:8090
//! BIRDNET_URL=http://127.0.0.1:8090 birdnet-go-frame
//! ```
//!
//! Set `UTC_OFFSET_MINUTES` to move the simulated dawn chorus to your time zone.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};

use birdnet_mock::MockBirdnet;
use tiny_http::{Header, Response, Server};

fn main() -> ExitCode {
    let address: String = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:8090"));
    let offset: i32 = env::var("UTC_OFFSET_MINUTES")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(0);
    let server: Server = match Server::http(&address) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("fake-birdnet: cannot listen on {address}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let station = MockBirdnet::new(offset);
    eprintln!("fake-birdnet: simulated BirdNET-Go on http://{address}/api/v2");
    for request in server.incoming_requests() {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
            });
        let answer = station.respond(request.url(), now);
        let Ok(content_type) = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        else {
            continue;
        };
        let response = Response::from_data(answer.body)
            .with_status_code(answer.status)
            .with_header(content_type);
        if let Err(error) = request.respond(response) {
            report(&error);
        }
    }
    ExitCode::SUCCESS
}

fn report(error: &io::Error) {
    eprintln!("fake-birdnet: could not answer a request: {error}");
}
