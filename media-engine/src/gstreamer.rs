//! GStreamer pipeline construction.
//!
//! GStreamer is used for encoding the canonical PCM stream into the
//! transport-appropriate codec for an output (Opus/AAC for Cast, Opus
//! for WebRTC, RAOP frames for AirPlay).
//!
//! In `mock` builds this module only returns string descriptors. In
//! `real` builds it also exposes a `build_pipeline` helper that
//! actually constructs typed `gstreamer::Pipeline` objects from the
//! same descriptor.

use analog_cloud_shared::output::OutputTransport;

/// Build a string description of the GStreamer pipeline we would
/// construct for `transport`. Useful for debug surfaces and tests; the
/// real implementation builds typed `gstreamer::Pipeline` objects.
pub fn describe_pipeline(transport: OutputTransport) -> &'static str {
    match transport {
        OutputTransport::WebRtcOpus => {
            // WebRTC sends Opus packets via the `webrtc` crate directly,
            // not via GStreamer. This descriptor is present for
            // symmetry / debug.
            "appsrc ! audioconvert ! opusenc bitrate=128000 ! rtpopuspay ! appsink"
        }
        OutputTransport::PipewireSink => {
            "appsrc ! audioconvert ! audioresample ! pipewiresink"
        }
        OutputTransport::OpusHttp => {
            "appsrc ! audioconvert ! opusenc bitrate=160000 ! webmmux streamable=true ! appsink"
        }
        OutputTransport::AacHttp => {
            "appsrc ! audioconvert ! avenc_aac bitrate=192000 ! aacparse ! appsink"
        }
        OutputTransport::Raop => {
            "appsrc ! audioconvert ! audio/x-raw,format=S16LE,rate=44100,channels=2 ! raopsink"
        }
    }
}

#[cfg(feature = "real")]
pub mod real {
    use anyhow::{Context, Result};
    use gstreamer as gst;
    use gst::prelude::*;

    use super::*;

    /// Construct a real, typed pipeline from the descriptor. Returns a
    /// `gst::Pipeline` ready to be linked into the active session.
    pub fn build_pipeline(transport: OutputTransport) -> Result<gst::Pipeline> {
        gst::init().context("gst::init")?;
        let desc = describe_pipeline(transport);
        let bin = gst::parse::launch(desc).context("gst::parse::launch")?;
        bin.downcast::<gst::Pipeline>()
            .map_err(|_| anyhow::anyhow!("pipeline element is not a gst::Pipeline"))
    }
}
