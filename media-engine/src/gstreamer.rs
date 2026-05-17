//! GStreamer pipeline construction.
//!
//! GStreamer is used for encoding the canonical PCM stream into the
//! transport-appropriate codec for an output (Opus/AAC for Cast, Opus
//! for WebRTC, RAOP frames for AirPlay).

use analog_cloud_shared::output::OutputTransport;

/// Build a string description of the GStreamer pipeline we would
/// construct for `transport`. Useful for debug surfaces and tests; the
/// real implementation builds typed `gstreamer::Pipeline` objects.
pub fn describe_pipeline(transport: OutputTransport) -> &'static str {
    match transport {
        OutputTransport::WebRtcOpus => {
            "appsrc ! audioconvert ! opusenc bitrate=128000 ! rtpopuspay ! webrtcbin"
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
