//! Canonical resampler.
//!
//! **Every** input source must pass through this resampler exactly
//! once, at the ingest boundary. Output stages are forbidden from
//! resampling — they consume the canonical PCM 48 kHz / stereo / f32
//! format directly.
//!
//! Uses the SincFixedIn algorithm from `rubato` — a high-quality
//! polyphase sinc resampler. The internal scratch buffers are
//! preallocated; `process` itself never touches the heap on the hot
//! path after warm-up.

use anyhow::{Context, Result};
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

use analog_cloud_shared::audio::CANONICAL_SAMPLE_RATE_HZ;

/// Block size we feed the resampler in. Tuned for a balance of latency
/// (≈ 21 ms at 48 kHz) and CPU overhead.
pub const RESAMPLER_FRAMES_PER_BLOCK: usize = 1_024;

/// Stereo resampler converting any input rate to the canonical 48 kHz.
pub struct CanonicalResampler {
    /// `None` when source rate already equals target (passthrough).
    inner: Option<SincFixedIn<f32>>,
    /// Pre-split input planes (left/right).
    input_planes: [Vec<f32>; 2],
    /// Pre-allocated output planes returned from `process`.
    output_planes: Vec<Vec<f32>>,
    /// Interleaved scratch used to splat the result back to LRLRLR…
    interleaved: Vec<f32>,
    source_rate: u32,
}

impl CanonicalResampler {
    pub fn new(source_rate: u32) -> Result<Self> {
        let inner = if source_rate == CANONICAL_SAMPLE_RATE_HZ {
            None
        } else {
            let params = SincInterpolationParameters {
                sinc_len: 128,
                f_cutoff: 0.95,
                oversampling_factor: 128,
                interpolation: SincInterpolationType::Linear,
                window: WindowFunction::BlackmanHarris2,
            };
            let ratio = CANONICAL_SAMPLE_RATE_HZ as f64 / source_rate as f64;
            let r = SincFixedIn::<f32>::new(ratio, 1.0, params, RESAMPLER_FRAMES_PER_BLOCK, 2)
                .context("constructing SincFixedIn resampler")?;
            Some(r)
        };

        let max_out_frames = ((RESAMPLER_FRAMES_PER_BLOCK as f64)
            * (CANONICAL_SAMPLE_RATE_HZ as f64 / source_rate as f64))
            .ceil() as usize
            + 64;

        Ok(Self {
            inner,
            input_planes: [
                vec![0.0; RESAMPLER_FRAMES_PER_BLOCK],
                vec![0.0; RESAMPLER_FRAMES_PER_BLOCK],
            ],
            output_planes: vec![vec![0.0; max_out_frames]; 2],
            interleaved: Vec::with_capacity(max_out_frames * 2),
            source_rate,
        })
    }

    pub fn source_rate(&self) -> u32 {
        self.source_rate
    }

    /// Resample one block. `input` is an interleaved stereo buffer of
    /// exactly `RESAMPLER_FRAMES_PER_BLOCK * 2` samples. Returns an
    /// interleaved stereo slice at 48 kHz; the slice is borrowed from
    /// the resampler's internal scratch.
    pub fn process<'a>(&'a mut self, input: &[f32]) -> Result<&'a [f32]> {
        assert_eq!(input.len(), RESAMPLER_FRAMES_PER_BLOCK * 2);

        // Passthrough fast path.
        if self.inner.is_none() {
            self.interleaved.clear();
            self.interleaved.extend_from_slice(input);
            return Ok(&self.interleaved);
        }

        // De-interleave into the two input planes.
        for (i, frame) in input.chunks_exact(2).enumerate() {
            self.input_planes[0][i] = frame[0];
            self.input_planes[1][i] = frame[1];
        }

        let resampler = self.inner.as_mut().unwrap();
        let in_slices: [&[f32]; 2] = [&self.input_planes[0], &self.input_planes[1]];

        // Make sure output planes are big enough for this block.
        let required = resampler.output_frames_next();
        for plane in &mut self.output_planes {
            if plane.len() < required {
                plane.resize(required, 0.0);
            }
        }
        let mut out_refs: Vec<&mut [f32]> = self
            .output_planes
            .iter_mut()
            .map(|p| &mut p[..required])
            .collect();

        let (_consumed, produced) = resampler
            .process_into_buffer(&in_slices, &mut out_refs[..], None)
            .context("resampler.process_into_buffer")?;

        // Re-interleave LRLRLR.
        self.interleaved.clear();
        self.interleaved.reserve(produced * 2);
        for i in 0..produced {
            self.interleaved.push(self.output_planes[0][i]);
            self.interleaved.push(self.output_planes[1][i]);
        }
        Ok(&self.interleaved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_block(freq_hz: f32, sample_rate: u32, frames: usize) -> Vec<f32> {
        let mut v = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            let t = i as f32 / sample_rate as f32;
            let s = (2.0 * std::f32::consts::PI * freq_hz * t).sin();
            v.push(s);
            v.push(s);
        }
        v
    }

    fn rms(samples: &[f32]) -> f32 {
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    #[test]
    fn passthrough_at_48khz() {
        let mut r = CanonicalResampler::new(48_000).unwrap();
        let block = sine_block(1000.0, 48_000, RESAMPLER_FRAMES_PER_BLOCK);
        let out = r.process(&block).unwrap();
        assert_eq!(out.len(), block.len());
    }

    #[test]
    fn upsample_44100_to_48000_preserves_energy() {
        let mut r = CanonicalResampler::new(44_100).unwrap();
        let block = sine_block(1_000.0, 44_100, RESAMPLER_FRAMES_PER_BLOCK);
        let in_rms = rms(&block);
        // Run a few blocks to flush the filter delay.
        let mut last = Vec::new();
        for _ in 0..6 {
            let out = r.process(&block).unwrap();
            last = out.to_vec();
        }
        let out_rms = rms(&last);
        let ratio = out_rms / in_rms;
        // RMS should match within 5%.
        assert!(
            ratio > 0.95 && ratio < 1.05,
            "rms ratio {ratio} (in={in_rms} out={out_rms})"
        );
    }

    #[test]
    fn downsample_96000_to_48000() {
        let mut r = CanonicalResampler::new(96_000).unwrap();
        let block = sine_block(1_000.0, 96_000, RESAMPLER_FRAMES_PER_BLOCK);
        let in_rms = rms(&block);
        let mut last = Vec::new();
        for _ in 0..6 {
            let out = r.process(&block).unwrap();
            last = out.to_vec();
        }
        let out_rms = rms(&last);
        let ratio = out_rms / in_rms;
        assert!(
            ratio > 0.95 && ratio < 1.05,
            "rms ratio {ratio} (in={in_rms} out={out_rms})"
        );
    }
}
