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
//!   pull-stream, and one RAOP sender.
//!
//! Only one output stream may be active at a time. The session manager
//! enforces this at the API level; the engine enforces it at the graph
//! level.

#![allow(dead_code)]

pub mod dsp;
pub mod gstreamer;
pub mod outputs;
pub mod pipewire;
pub mod transcoding;

use std::sync::Arc;

use parking_lot::Mutex;
use thiserror::Error;

use analog_cloud_shared::audio::AudioFormat;
use analog_cloud_shared::input::InputDescriptor;
use analog_cloud_shared::output::OutputDescriptor;

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

/// Handle to the running media engine. Cheap to clone — internal state
/// is in an Arc.
#[derive(Clone)]
pub struct MediaEngine {
    inner: Arc<Inner>,
}

struct Inner {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    inputs: Vec<InputDescriptor>,
    outputs: Vec<OutputDescriptor>,
    active_stream: Option<ActiveStream>,
}

struct ActiveStream {
    input_id: String,
    output_id: String,
    format: AudioFormat,
}

impl MediaEngine {
    /// Initialize the engine. In the `mock` feature this just spins up
    /// an in-memory simulation; in `real` it would connect to PipeWire.
    pub async fn start() -> Result<Self> {
        tracing::info!("media-engine starting (mock backend)");
        Ok(Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State::default()),
            }),
        })
    }

    pub fn list_inputs(&self) -> Vec<InputDescriptor> {
        self.inner.state.lock().inputs.clone()
    }

    pub fn list_outputs(&self) -> Vec<OutputDescriptor> {
        self.inner.state.lock().outputs.clone()
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
}
