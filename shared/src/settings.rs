use serde::{Deserialize, Serialize};

use crate::output::AudioMode;

/// 10-band global equalizer, in dB. Bands are fixed at:
/// 32, 64, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz.
pub const EQ_BANDS_HZ: [u32; 10] =
    [32, 64, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct EqualizerSettings {
    pub enabled: bool,
    pub preset: EqPreset,
    pub bands_db: [f32; 10],
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: EqPreset::Flat,
            bands_db: [0.0; 10],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EqPreset {
    Flat,
    VinylWarmth,
    BassFocus,
    Speech,
    Tape,
    Night,
    Custom,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnrichmentMode {
    Off,
    Immediate,
    Delayed,
    Manual,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichmentSettings {
    pub mode: EnrichmentMode,
    pub delayed_wait_seconds: u32,
}

impl Default for EnrichmentSettings {
    fn default() -> Self {
        Self {
            mode: EnrichmentMode::Delayed,
            delayed_wait_seconds: 30,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HassSettings {
    pub enabled: bool,
    pub mqtt_host: Option<String>,
    pub mqtt_port: Option<u16>,
    pub mqtt_username: Option<String>,
    pub mqtt_topic_prefix: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub audio_mode: AudioMode,
    pub equalizer: EqualizerSettings,
    pub enrichment: EnrichmentSettings,
    pub hass: HassSettings,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio_mode: AudioMode::default(),
            equalizer: EqualizerSettings::default(),
            enrichment: EnrichmentSettings::default(),
            hass: HassSettings::default(),
            theme: "analog-core".to_string(),
        }
    }
}
