use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::output::{OutputKind, OutputTransport};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Armed,
    Playing,
    Reconnecting,
    Stopped,
    Errored,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub latency_ms: u32,
    pub buffer_ms: u32,
    pub codec: Option<String>,
    pub bitrate_kbps: Option<u32>,
    pub reconnects: u32,
    pub cpu_usage_pct: Option<f32>,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            buffer_ms: 0,
            codec: None,
            bitrate_kbps: None,
            reconnects: 0,
            cpu_usage_pct: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub input_id: String,
    pub output_kind: OutputKind,
    pub output_device: String,
    pub transport: OutputTransport,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
    /// IDs of clients (e.g. browsers) currently subscribed as controllers.
    pub controller_clients: Vec<String>,
    pub state: SessionState,
    pub stats: SessionStats,
}

/// Request body to start (or take over with) a new session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartSessionRequest {
    pub input_id: String,
    pub output_kind: OutputKind,
    pub output_device: String,
}
