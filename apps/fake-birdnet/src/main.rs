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

use std::env;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use birdnet_mock::MockBirdnet;
use lan_http::{Response, Server};

fn main() -> ExitCode {
    let address: String = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:8090"));
    let offset: i32 = env::var("UTC_OFFSET_MINUTES")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(0);
    let server: Server = match Server::bind(&address) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("fake-birdnet: cannot listen on {address}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let station = MockBirdnet::new(offset);
    eprintln!("fake-birdnet: simulated BirdNET-Go on http://{address}/api/v2");
    server.run(move |request| {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
            });
        let answer = station.respond(request.target(), now);
        Response::new(answer.status, answer.body).with_header("Content-Type", "application/json")
    });
    ExitCode::SUCCESS
}
