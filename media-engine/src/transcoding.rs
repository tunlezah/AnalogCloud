//! Transcoders that consume the canonical PCM stream and produce
//! transport-appropriate byte streams (Opus-in-WebM, AAC-in-ADTS, etc.).

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Codec {
    Opus,
    Aac,
    Pcm,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct EncodingProfile {
    pub codec: Codec,
    pub bitrate_kbps: u32,
}

impl EncodingProfile {
    pub const OPUS_WEBRTC: Self = Self { codec: Codec::Opus, bitrate_kbps: 128 };
    pub const OPUS_CAST: Self =   Self { codec: Codec::Opus, bitrate_kbps: 160 };
    pub const AAC_CAST: Self =    Self { codec: Codec::Aac,  bitrate_kbps: 192 };
}
