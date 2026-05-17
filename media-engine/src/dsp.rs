//! Realtime-safe DSP: cascaded biquad equalizer + peak/RMS level meter.
//!
//! Everything here must be allocation-free on the hot path. The
//! `Equalizer::process` and `LevelMeter::push` methods take `&mut [f32]`
//! and run with no heap touches.
//!
//! ## EQ topology
//!
//! 10 cascaded peaking biquads per channel, one per band. Bands are
//! fixed at the ISO frequencies:
//!
//! ```text
//! 32, 64, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz
//! ```
//!
//! Q is fixed at 1.41 (≈ a constant-Q feel across the spectrum). Gains
//! are clamped to ±18 dB.

use biquad::{Biquad, Coefficients, DirectForm1, Type, Q_BUTTERWORTH_F32};

use analog_cloud_shared::audio::{ChannelLevel, StereoLevel, CANONICAL_SAMPLE_RATE_HZ};
use analog_cloud_shared::settings::{EqualizerSettings, EQ_BANDS_HZ};

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
        // 300 ms RMS time constant; peak decay matched to ~1.7 dB / frame
        // at 48 kHz.
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

const NUM_BANDS: usize = 10;
const MAX_GAIN_DB: f32 = 18.0;

/// Cascaded peaking-biquad equalizer.
///
/// One filter chain per channel. Stereo only — matches the canonical
/// internal format.
pub struct Equalizer {
    sample_rate: u32,
    enabled: bool,
    bands_db: [f32; NUM_BANDS],
    left: [DirectForm1<f32>; NUM_BANDS],
    right: [DirectForm1<f32>; NUM_BANDS],
}

impl Equalizer {
    pub fn new(settings: &EqualizerSettings) -> Self {
        Self::with_sample_rate(settings, CANONICAL_SAMPLE_RATE_HZ)
    }

    pub fn with_sample_rate(settings: &EqualizerSettings, sample_rate: u32) -> Self {
        let bands_db = clamp_bands(settings.bands_db);
        let mut me = Self {
            sample_rate,
            enabled: settings.enabled,
            bands_db,
            left: build_chain(sample_rate, &bands_db),
            right: build_chain(sample_rate, &bands_db),
        };
        // Reset to zero state.
        for b in &mut me.left {
            b.reset_state();
        }
        for b in &mut me.right {
            b.reset_state();
        }
        me
    }

    /// Re-tune the cascade in-place.
    pub fn update(&mut self, settings: &EqualizerSettings) {
        let bands_db = clamp_bands(settings.bands_db);
        self.enabled = settings.enabled;
        if bands_db != self.bands_db {
            self.bands_db = bands_db;
            for (i, &gain_db) in bands_db.iter().enumerate() {
                let coeffs = peaking_coeffs(self.sample_rate, EQ_BANDS_HZ[i], gain_db);
                self.left[i].update_coefficients(coeffs);
                self.right[i].update_coefficients(coeffs);
            }
        }
    }

    /// Process an interleaved stereo block in-place.
    pub fn process(&mut self, block: &mut [f32]) {
        if !self.enabled {
            return;
        }
        debug_assert!(block.len() % 2 == 0);
        for frame in block.chunks_exact_mut(2) {
            let mut l = frame[0];
            let mut r = frame[1];
            for i in 0..NUM_BANDS {
                l = self.left[i].run(l);
                r = self.right[i].run(r);
            }
            frame[0] = l;
            frame[1] = r;
        }
    }
}

fn clamp_bands(b: [f32; NUM_BANDS]) -> [f32; NUM_BANDS] {
    let mut out = [0.0; NUM_BANDS];
    for i in 0..NUM_BANDS {
        out[i] = b[i].clamp(-MAX_GAIN_DB, MAX_GAIN_DB);
    }
    out
}

fn build_chain(sample_rate: u32, bands_db: &[f32; NUM_BANDS]) -> [DirectForm1<f32>; NUM_BANDS] {
    let mut out: [DirectForm1<f32>; NUM_BANDS] = std::array::from_fn(|i| {
        let coeffs = peaking_coeffs(sample_rate, EQ_BANDS_HZ[i], bands_db[i]);
        DirectForm1::<f32>::new(coeffs)
    });
    for b in &mut out {
        b.reset_state();
    }
    out
}

fn peaking_coeffs(sample_rate: u32, center_hz: u32, gain_db: f32) -> Coefficients<f32> {
    // PeakingEQ keeps gain at all other frequencies near unity; cascading
    // them yields the canonical 10-band graphic EQ response.
    //
    // NOTE: `biquad::Coefficients::from_params` normalizes the cutoff as
    // `f0 / (2 * fs)`, which under-shoots omega by a factor of 4 compared
    // to the RBJ Audio EQ Cookbook. We compute the Nyquist-normalized
    // cutoff directly (`2 * f0 / fs`) and feed it to
    // `from_normalized_params` so the peak actually lands at `center_hz`.
    let normalized = 2.0 * center_hz as f32 / sample_rate as f32;
    Coefficients::<f32>::from_normalized_params(
        Type::PeakingEQ(gain_db),
        normalized,
        Q_BUTTERWORTH_F32,
    )
    .expect("biquad coefficients valid for canonical EQ bands")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(n_frames: usize, sample: impl Fn(usize) -> (f32, f32)) -> Vec<f32> {
        let mut v = Vec::with_capacity(n_frames * 2);
        for i in 0..n_frames {
            let (l, r) = sample(i);
            v.push(l);
            v.push(r);
        }
        v
    }

    fn sine(freq_hz: f32, sample_rate: u32, n_frames: usize) -> Vec<f32> {
        block(n_frames, |i| {
            let t = i as f32 / sample_rate as f32;
            let s = (2.0 * std::f32::consts::PI * freq_hz * t).sin();
            (s, s)
        })
    }

    fn rms(samples: &[f32]) -> f32 {
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    #[test]
    fn flat_eq_is_unity() {
        let settings = EqualizerSettings::default();
        let mut eq = Equalizer::new(&settings);
        let mut sig = sine(1000.0, 48_000, 4_800);
        let before = rms(&sig);
        eq.process(&mut sig);
        let after = rms(&sig);
        assert!((before - after).abs() / before < 0.02, "before={before} after={after}");
    }

    #[test]
    fn disabled_eq_is_unchanged() {
        let mut settings = EqualizerSettings::default();
        settings.enabled = false;
        settings.bands_db = [12.0; NUM_BANDS];
        let mut eq = Equalizer::new(&settings);
        let mut sig = sine(1000.0, 48_000, 1_024);
        let copy = sig.clone();
        eq.process(&mut sig);
        assert_eq!(sig, copy);
    }

    #[test]
    fn boosting_a_band_amplifies_its_center() {
        let mut bands = [0.0_f32; NUM_BANDS];
        // Boost the 1 kHz band by +12 dB.
        let idx = EQ_BANDS_HZ.iter().position(|&hz| hz == 1000).unwrap();
        bands[idx] = 12.0;
        let settings = EqualizerSettings {
            enabled: true,
            preset: analog_cloud_shared::settings::EqPreset::Custom,
            bands_db: bands,
        };
        let mut eq = Equalizer::new(&settings);
        // Skip first half of the buffer to let the filter settle.
        let mut sig = sine(1000.0, 48_000, 8_000);
        eq.process(&mut sig);
        let tail = &sig[sig.len() / 2..];
        let r = rms(tail);
        // +12 dB ≈ ×3.98. The cascade has neighboring bands slightly
        // affecting the response, so allow ±25 % of expected.
        assert!(r > 2.5 && r < 6.0, "rms after +12 dB boost: {r}");
    }

    #[test]
    fn cutting_a_band_attenuates_its_center() {
        let mut bands = [0.0_f32; NUM_BANDS];
        let idx = EQ_BANDS_HZ.iter().position(|&hz| hz == 1000).unwrap();
        bands[idx] = -12.0;
        let settings = EqualizerSettings {
            enabled: true,
            preset: analog_cloud_shared::settings::EqPreset::Custom,
            bands_db: bands,
        };
        let mut eq = Equalizer::new(&settings);
        let mut sig = sine(1000.0, 48_000, 8_000);
        eq.process(&mut sig);
        let tail = &sig[sig.len() / 2..];
        let r = rms(tail);
        // -12 dB ≈ ×0.25.
        assert!(r < 0.5, "rms after -12 dB cut: {r}");
    }

    #[test]
    fn meter_reports_near_zero_for_silence() {
        let mut m = LevelMeter::new(48_000);
        m.push(&vec![0.0; 4_800]);
        let s = m.snapshot();
        assert!(s.left.peak_dbfs <= -100.0);
        assert!(s.right.peak_dbfs <= -100.0);
    }

    #[test]
    fn meter_reports_near_zero_dbfs_for_full_scale() {
        let mut m = LevelMeter::new(48_000);
        let sig = sine(1000.0, 48_000, 9_600);
        m.push(&sig);
        let s = m.snapshot();
        // Peak of unit sine is 1.0 → 0 dBFS.
        assert!(s.left.peak_dbfs > -1.0 && s.left.peak_dbfs <= 0.05);
    }
}
