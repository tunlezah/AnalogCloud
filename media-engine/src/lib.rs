//! Analog Cloud media engine.
//!
//! Wraps the underlying audio stack (PipeWire / WirePlumber / GStreamer
//! / FFmpeg) behind a small, async, backend-friendly API.
//!
//! ## Design
//!
//! * **Ingest** — discover and open Bluetooth/line-in sources, normalize
//!   to the canonical PCM format (48 kHz · stereo · float32).
//! * **DSP** — equalization, level metering, optional analog warmth.
//! * **Transcode** — encode the canonical stream for an output transport
//!   (Opus, AAC, raw PCM, RAOP).
//! * **Outputs** — manage the lifecycle of one local sink, one HTTP
//!   pull-stream, one RAOP sender, and one WebRTC peer.
//!
//! Only one output stream may be active at a time. The session manager
//! enforces this at the API level; the engine enforces it at the graph
//! level.

#![allow(dead_code)]

pub mod dsp;
pub mod gstreamer;
pub mod outputs;
pub mod pipewire;
pub mod resampler;
pub mod transcoding;

use std::sync::Arc;

use parking_lot::Mutex;
use thiserror::Error;
use tokio::sync::broadcast;

use analog_cloud_shared::audio::{AudioFormat, StereoLevel};
use analog_cloud_shared::input::InputDescriptor;
use analog_cloud_shared::output::OutputDescriptor;
use analog_cloud_shared::settings::EqualizerSettings;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("input not found: {0}")]
    InputNotFound(String),
    #[error("output not found: {0}")]
    OutputNotFound(String),
    #[error("another session already owns the output graph")]
    OutputBusy,
    #[error("media stack error: {0}")]
    Stack(String),
}

pub type Result<T> = std::result::Result<T, MediaError>;

/// A single level-meter sample emitted by the engine's analysis loop.
#[derive(Clone, Debug)]
pub struct LevelSample {
    pub input_id: String,
    pub level: StereoLevel,
}

/// Handle to the running media engine. Cheap to clone — internal state
/// is in an Arc.
#[derive(Clone)]
pub struct MediaEngine {
    inner: Arc<Inner>,
}

struct Inner {
    state: Mutex<State>,
    levels: broadcast::Sender<LevelSample>,
}

#[derive(Default)]
struct State {
    active_stream: Option<ActiveStream>,
    eq: Option<EqualizerSettings>,
}

struct ActiveStream {
    input_id: String,
    output_id: String,
    format: AudioFormat,
}

impl MediaEngine {
    /// Initialize the engine. In the `mock` feature this spins up a
    /// deterministic in-memory simulation; in `real` it would connect
    /// to PipeWire.
    pub async fn start() -> Result<Self> {
        tracing::info!("media-engine starting");
        let (tx, _) = broadcast::channel(128);
        Ok(Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State::default()),
                levels: tx,
            }),
        })
    }

    /// Subscribe to periodic level-meter samples.
    pub fn subscribe_levels(&self) -> broadcast::Receiver<LevelSample> {
        self.inner.levels.subscribe()
    }

    /// Open a stream from `input_id` to `output_id`. Atomically tears
    /// down any existing stream first.
    ///
    /// The caller is expected to have already validated that the input
    /// and output exist (the session manager does this against its
    /// device catalog). The engine just executes.
    pub async fn open_stream(&self, input_id: &str, output_id: &str) -> Result<()> {
        let mut state = self.inner.state.lock();
        state.active_stream = Some(ActiveStream {
            input_id: input_id.to_string(),
            output_id: output_id.to_string(),
            format: AudioFormat::canonical(),
        });
        tracing::info!(input_id, output_id, "stream opened");
        Ok(())
    }

    pub async fn close_stream(&self) -> Result<()> {
        self.inner.state.lock().active_stream = None;
        tracing::info!("stream closed");
        Ok(())
    }

    /// Update the global EQ. Cheap; takes effect on the next processing
    /// block.
    pub fn set_equalizer(&self, settings: EqualizerSettings) {
        self.inner.state.lock().eq = Some(settings);
    }

    /// The currently active input, if any.
    pub fn active_input_id(&self) -> Option<String> {
        self.inner
            .state
            .lock()
            .active_stream
            .as_ref()
            .map(|s| s.input_id.clone())
    }

    /// Inject a level sample into the broadcast channel. Used by the
    /// analysis loop and by integration tests.
    pub fn publish_level(&self, sample: LevelSample) {
        let _ = self.inner.levels.send(sample);
    }

    // Compatibility shims used by other code expecting the catalog —
    // empty in the engine; the device-discovery crate owns the catalog.
    pub fn list_inputs(&self) -> Vec<InputDescriptor> {
        Vec::new()
    }

    pub fn list_outputs(&self) -> Vec<OutputDescriptor> {
        Vec::new()
    }
}
