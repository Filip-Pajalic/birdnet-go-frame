//! Announces each frame's wakes to Home Assistant over MQTT.

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use frame_http::{FrameLog, FrameVisit};
use home_assistant::Message;

use crate::config::Mqtt;

/// How often the frame log is checked for new wakes.
const CHECK: Duration = Duration::from_secs(15);
/// How long the broker may take to answer.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Publishes every frame's discovery messages and state whenever it wakes, forever. A broker
/// that cannot be reached is tried again at the next check.
pub fn run(mqtt: &Mqtt, log: &FrameLog) {
    let mut published: HashMap<String, (i64, u64)> = HashMap::new();
    let login: Option<(&str, &str)> = mqtt
        .login
        .as_ref()
        .map(|(user, password)| (user.as_str(), password.as_str()));
    eprintln!(
        "server: announcing frames to Home Assistant at {}",
        mqtt.address
    );
    loop {
        let fresh: Vec<FrameVisit> = log
            .visits()
            .into_iter()
            .filter(|visit| published.get(&visit.frame) != Some(&(visit.last_wake, visit.wakes)))
            .collect();
        if !fresh.is_empty() {
            let messages: Vec<Message> = fresh
                .iter()
                .flat_map(|visit| {
                    home_assistant::discovery::frame_messages(&mqtt.discovery_prefix, visit)
                })
                .collect();
            match home_assistant::mqtt::publish(
                &mqtt.address,
                "birdnet-go-frame",
                login,
                &messages,
                TIMEOUT,
            ) {
                Ok(()) => {
                    for visit in fresh {
                        published.insert(visit.frame, (visit.last_wake, visit.wakes));
                    }
                }
                Err(error) => eprintln!("server: MQTT {}: {error}", mqtt.address),
            }
        }
        thread::sleep(CHECK);
    }
}
