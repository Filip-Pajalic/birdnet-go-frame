//! The messages that make a frame a device in Home Assistant.

use birdnet_client::CivilDate;
use frame_http::FrameVisit;

use crate::message::Message;

/// The topics frames' states are published under.
pub const STATE_PREFIX: &str = "birdnet-go-frame";

/// One sensor: its key, name, the state field it shows, its unit, device class and category.
struct Sensor {
    key: &'static str,
    name: &'static str,
    field: &'static str,
    unit: Option<&'static str>,
    device_class: Option<&'static str>,
    diagnostic: bool,
}

const SENSORS: [Sensor; 5] = [
    Sensor {
        key: "battery",
        name: "Battery",
        field: "battery_percent",
        unit: Some("%"),
        device_class: Some("battery"),
        diagnostic: false,
    },
    Sensor {
        key: "battery_voltage",
        name: "Battery voltage",
        field: "battery_mv",
        unit: Some("mV"),
        device_class: Some("voltage"),
        diagnostic: true,
    },
    Sensor {
        key: "last_wake",
        name: "Last wake",
        field: "last_wake_time",
        unit: None,
        device_class: Some("timestamp"),
        diagnostic: false,
    },
    Sensor {
        key: "view",
        name: "Page",
        field: "view",
        unit: None,
        device_class: None,
        diagnostic: false,
    },
    Sensor {
        key: "firmware",
        name: "Firmware",
        field: "firmware",
        unit: None,
        device_class: None,
        diagnostic: true,
    },
];

/// Returns the discovery messages for `visit`'s frame, under Home Assistant's `prefix`
/// (`homeassistant` unless changed), followed by its state.
#[must_use]
pub fn frame_messages(prefix: &str, visit: &FrameVisit) -> Vec<Message> {
    let object: String = format!("birdnet_go_frame_{}", visit.frame.replace('-', "_"));
    let state_topic: String = format!("{STATE_PREFIX}/{}/state", visit.frame);
    let firmware: String = visit.firmware.as_deref().map_or(String::new(), |version| {
        format!(r#","sw_version":"{}""#, escape(version))
    });
    let device: String = format!(
        r#"{{"identifiers":["{object}"],"name":"Bird frame {}","manufacturer":"birdnet-go-frame","model":"Spectra 6 e-ink frame"{firmware}}}"#,
        escape(&visit.frame)
    );
    let mut messages: Vec<Message> = SENSORS
        .iter()
        .map(|sensor| {
            let mut config: String = format!(
                r#"{{"name":"{}","unique_id":"{object}_{}","object_id":"{object}_{}","state_topic":"{state_topic}","value_template":"{{{{ value_json.{} }}}}""#,
                sensor.name, sensor.key, sensor.key, sensor.field
            );
            if let Some(unit) = sensor.unit {
                config.push_str(&format!(r#","unit_of_measurement":"{unit}","state_class":"measurement""#));
            }
            if let Some(class) = sensor.device_class {
                config.push_str(&format!(r#","device_class":"{class}""#));
            }
            if sensor.diagnostic {
                config.push_str(r#","entity_category":"diagnostic""#);
            }
            config.push_str(&format!(r#","device":{device}}}"#));
            Message {
                topic: format!("{prefix}/sensor/{object}/{}/config", sensor.key),
                payload: config,
            }
        })
        .collect();
    let mut state: String = visit.to_json();
    state.pop();
    state.push_str(&format!(
        r#","last_wake_time":"{}"}}"#,
        iso_time(visit.last_wake)
    ));
    messages.push(Message {
        topic: state_topic,
        payload: state,
    });
    messages
}

/// Formats unix seconds as an ISO 8601 time in UTC, as Home Assistant's timestamps take it.
fn iso_time(unix_seconds: i64) -> String {
    let date: CivilDate = CivilDate::from_days_since_epoch(unix_seconds.div_euclid(86_400));
    let seconds: i64 = unix_seconds.rem_euclid(86_400);
    format!(
        "{date}T{:02}:{:02}:{:02}+00:00",
        seconds / 3_600,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// `text` safe inside a JSON string.
fn escape(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control())
        .flat_map(|character| match character {
            '"' | '\\' => vec!['\\', character],
            other => vec![other],
        })
        .collect()
}

#[cfg(test)]
mod test {
    use frame_http::FrameLog;

    use super::*;

    #[test]
    fn makes_a_device_with_its_sensors_and_state() {
        let log = FrameLog::new();
        log.record_at(
            1_793_955_600,
            "hall-way",
            "story",
            Some(3_900),
            Some("0.2.0"),
        );
        let visit: FrameVisit = log.visit("hall-way").expect("it woke");
        let messages: Vec<Message> = frame_messages("homeassistant", &visit);
        assert_eq!(messages.len(), SENSORS.len() + 1);
        let battery: &Message = &messages[0];
        assert_eq!(
            battery.topic,
            "homeassistant/sensor/birdnet_go_frame_hall_way/battery/config"
        );
        assert!(
            battery
                .payload
                .contains(r#""value_template":"{{ value_json.battery_percent }}""#),
            "{}",
            battery.payload
        );
        assert!(battery.payload.contains(r#""sw_version":"0.2.0""#));
        let state: &Message = messages.last().expect("a state");
        assert_eq!(state.topic, "birdnet-go-frame/hall-way/state");
        assert!(
            state
                .payload
                .ends_with(r#""last_wake_time":"2026-11-06T09:00:00+00:00"}"#),
            "{}",
            state.payload
        );
        assert!(state.payload.contains(r#""battery_percent":70"#));
    }
}
