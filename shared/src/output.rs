use serde::{Deserialize, Serialize};

/// Where the audio is going.
///
/// Exactly **one** output may be active at a time — see the architecture
/// document.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    Browser,
    Local,
    Chromecast,
    AirPlay,
}

/// Transport for an output. Determines codec, latency profile, and how
/// the receiver connects.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputTransport {
    /// WebRTC peer connection — browser only.
    WebRtcOpus,
    /// Direct PipeWire sink — local speakers only.
    PipewireSink,
    /// Opus muxed in WebM, pulled by the Cast device over HTTP.
    OpusHttp,
    /// AAC in ADTS, pulled by the Cast device over HTTP.
    AacHttp,
    /// AirPlay RAOP sender.
    Raop,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputState {
    Idle,
    Negotiating,
    Streaming,
    Reconnecting,
    Errored,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputDescriptor {
    pub id: String,
    pub name: String,
    pub kind: OutputKind,
    pub transport: OutputTransport,
    pub state: OutputState,
    pub latency_ms_estimate: Option<u32>,
    /// Is this the currently active output for a `playing` session.
    pub active: bool,
    /// Can the backend re-establish this output without user intervention.
    pub reconnectable: bool,
}

/// Audio quality preference. User-selectable.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioMode {
    /// Prioritize fidelity, minimal transcoding, high-quality paths.
    #[default]
    Reference,
    /// Prioritize reliability, compatibility, resilient transports.
    Compatible,
}
