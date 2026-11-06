//! Tells Home Assistant about the frames: each frame becomes a device with its battery, battery
//! voltage, last wake, page and firmware as sensors, found by Home Assistant's MQTT discovery.
//!
//! [`mqtt`] holds just enough MQTT 3.1.1 to publish retained messages at QoS 0, on the standard
//! library alone; [`discovery`] writes the messages for a frame. Without a broker, Home
//! Assistant can read the same facts from the server's `/api/frames` with its REST sensor.
//!
//! ```
//! use frame_http::FrameLog;
//!
//! let log = FrameLog::new();
//! log.record_at(1_793_955_600, "hall", "today", Some(3_900), Some("0.2.0"));
//! let visit = log.visit("hall").expect("it woke");
//! let messages = home_assistant::discovery::frame_messages("homeassistant", &visit);
//! assert!(messages.iter().any(|message| message.topic == "birdnet-go-frame/hall/state"));
//! ```

pub mod discovery;
pub mod mqtt;

mod message;

pub use message::Message;
