//! Realtime-safe DSP: equalizer, level meter, optional analog warmth.
//!
//! Everything here must be allocation-free on the hot path.

use analog_cloud_shared::audio::{ChannelLevel, StereoLevel};
use analog_cloud_shared::settings::EqualizerSettings;

/// Stateful peak/RMS meter with simple IIR ballistics.
pub struct LevelMeter {
    peak_l: f32,
    peak_r: f32,
    rms_l: f32,
    rms_r: f32,
    peak_decay: f32,
    rms_alpha: f32,
}

impl LevelMeter {
    pub fn new(sample_rate: u32) -> Self {
        // 1.7 dB / frame peak decay, 300 ms RMS time constant.
        let frame_dt = 1.0 / sample_rate as f32;
        let rms_alpha = 1.0 - (-frame_dt / 0.3_f32).exp();
        Self {
            peak_l: 0.0,
            peak_r: 0.0,
            rms_l: 0.0,
            rms_r: 0.0,
            peak_decay: 0.9995,
            rms_alpha,
        }
    }

    /// Feed an interleaved stereo block (LRLRLR...). The number of
    /// samples must be a multiple of 2.
    pub fn push(&mut self, block: &[f32]) {
        debug_assert!(block.len() % 2 == 0);
        for frame in block.chunks_exact(2) {
            let l = frame[0].abs();
            let r = frame[1].abs();
            self.peak_l = (self.peak_l * self.peak_decay).max(l);
            self.peak_r = (self.peak_r * self.peak_decay).max(r);
            self.rms_l += self.rms_alpha * (l * l - self.rms_l);
            self.rms_r += self.rms_alpha * (r * r - self.rms_r);
        }
    }

    pub fn snapshot(&self) -> StereoLevel {
        StereoLevel {
            left: ChannelLevel {
                peak_dbfs: to_dbfs(self.peak_l),
                rms_dbfs: to_dbfs(self.rms_l.sqrt()),
            },
            right: ChannelLevel {
                peak_dbfs: to_dbfs(self.peak_r),
                rms_dbfs: to_dbfs(self.rms_r.sqrt()),
            },
        }
    }
}

fn to_dbfs(linear: f32) -> f32 {
    if linear <= 1e-9 {
        -120.0
    } else {
        20.0 * linear.log10()
    }
}

/// A trivial parametric EQ placeholder. The real implementation will
/// use cascaded biquads.
pub struct Equalizer {
    bands_db: [f32; 10],
    enabled: bool,
}

impl Equalizer {
    pub fn new(settings: &EqualizerSettings) -> Self {
        Self {
            bands_db: settings.bands_db,
            enabled: settings.enabled,
        }
    }

    pub fn process(&self, _block: &mut [f32]) {
        if !self.enabled {
            return;
        }
        // TODO: biquad cascade. Currently a no-op.
    }
}
