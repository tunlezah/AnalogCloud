use serde::{Deserialize, Serialize};

use crate::audio::AudioFormat;

/// What kind of physical or wireless input is this.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    BluetoothTurntable,
    BluetoothCassette,
    BluetoothGeneric,
    LineIn,
    Usb,
    Virtual,
}

/// Transport / protocol carrying the input into the graph.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputTransport {
    BluetoothA2dp,
    BluetoothHfp,
    Alsa,
    PipewireNode,
    Virtual,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputDescriptor {
    pub id: String,
    pub name: String,
    pub kind: InputKind,
    pub transport: InputTransport,
    pub codec: Option<String>,
    pub native_format: Option<AudioFormat>,
    /// Whether the device is currently connected (Bluetooth link up, jack
    /// plugged, etc.).
    pub connected: bool,
    /// Whether the source is the currently selected input for an active
    /// session.
    pub selected: bool,
    /// Most recent signal-level snapshot, if any.
    pub last_level_dbfs: Option<f32>,
}
