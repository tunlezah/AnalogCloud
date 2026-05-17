//! Home Assistant integration.
//!
//! HASS is **informational only**. It receives state via MQTT and may
//! display: active input, active output, playback state, metadata,
//! volume, artwork URL, and a periodic waveform snapshot.
//!
//! HASS must **not** be allowed to route audio. We never subscribe to
//! command topics that affect routing — the backend is the sole
//! routing authority.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HassPublishConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub topic_prefix: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct HassState {
    pub input: Option<String>,
    pub output: Option<String>,
    pub playing: bool,
    pub track: Option<String>,
    pub artist: Option<String>,
    pub artwork_url: Option<String>,
    pub volume_pct: Option<u8>,
}

/// Stub publisher. The real implementation owns an `rumqttc` client
/// and republishes on every backend event.
#[derive(Clone, Default)]
pub struct HassPublisher;

impl HassPublisher {
    pub fn new() -> Self {
        Self
    }
    pub async fn publish(&self, _state: &HassState) {
        tracing::trace!("hass: publish (stub)");
    }
}
