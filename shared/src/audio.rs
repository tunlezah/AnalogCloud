use serde::{Deserialize, Serialize};

/// The canonical internal audio format.
///
/// Every source is resampled to this format **exactly once** at the
/// ingest boundary. No downstream stage may resample.
pub const CANONICAL_SAMPLE_RATE_HZ: u32 = 48_000;
pub const CANONICAL_CHANNELS: u16 = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleFormat {
    F32,
    S16,
    S24,
    S32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: SampleFormat,
}

impl AudioFormat {
    pub const fn canonical() -> Self {
        Self {
            sample_rate: CANONICAL_SAMPLE_RATE_HZ,
            channels: CANONICAL_CHANNELS,
            format: SampleFormat::F32,
        }
    }
}

/// A peak/RMS level pair for a single channel, in dBFS.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ChannelLevel {
    pub peak_dbfs: f32,
    pub rms_dbfs: f32,
}

/// Stereo level snapshot.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StereoLevel {
    pub left: ChannelLevel,
    pub right: ChannelLevel,
}
