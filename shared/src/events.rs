//! Event envelope sent over the SSE stream to the frontend, and also
//! used internally on the backend's event bus.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::audio::StereoLevel;
use crate::input::InputDescriptor;
use crate::output::OutputDescriptor;
use crate::session::{Session, SessionStats};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// A new input was discovered or an existing one changed.
    InputUpdated { input: InputDescriptor },
    /// An input went away.
    InputRemoved { input_id: String },
    /// An output endpoint was discovered or its state changed.
    OutputUpdated { output: OutputDescriptor },
    /// An output endpoint went away.
    OutputRemoved { output_id: String },
    /// A new session is being constructed.
    SessionArmed { session: Session },
    /// A session reached `playing` state.
    SessionStarted { session: Session },
    /// A session's stats sample.
    SessionStats {
        session_id: Uuid,
        stats: SessionStats,
    },
    /// Periodic level meter sample for the active input.
    Levels {
        input_id: String,
        level: StereoLevel,
    },
    /// A session was stopped (manually, by takeover, or by error).
    SessionStopped {
        session_id: Uuid,
        reason: SessionStopReason,
    },
    /// User-facing notification (toast).
    Notice {
        severity: NoticeSeverity,
        message: String,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStopReason {
    UserRequested,
    TakenOver,
    InputLost,
    OutputLost,
    Errored,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeSeverity {
    Info,
    Warn,
    Error,
}
