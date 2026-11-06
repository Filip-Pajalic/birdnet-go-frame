//! One message to publish.

/// A retained MQTT message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// The topic, e.g. `birdnet-go-frame/hall/state`.
    pub topic: String,
    /// The payload, JSON for everything this crate sends.
    pub payload: String,
}
