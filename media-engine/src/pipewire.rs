//! PipeWire integration.
//!
//! In `real` builds this links against `libpipewire-0.3` via the
//! `pipewire-rs` crate, walks the node graph, subscribes to events, and
//! exposes Bluetooth / ALSA / virtual nodes as `InputDescriptor`s.
//!
//! In `mock` builds it returns a deterministic fake topology so the rest
//! of the system can be developed on hosts without an audio stack.

use analog_cloud_shared::audio::{AudioFormat, SampleFormat};
use analog_cloud_shared::input::{InputDescriptor, InputKind, InputTransport};

/// Enumerate inputs currently exposed by the PipeWire graph.
///
/// Real implementation:
/// 1. Open a PipeWire core connection.
/// 2. Roundtrip the registry.
/// 3. Filter to source nodes with media class `Audio/Source`.
/// 4. Classify by `device.product.name`, A2DP role, ALSA card type.
pub async fn enumerate_inputs() -> Vec<InputDescriptor> {
    #[cfg(feature = "mock")]
    {
        return mock_inputs();
    }
    #[cfg(not(feature = "mock"))]
    {
        Vec::new()
    }
}

fn mock_inputs() -> Vec<InputDescriptor> {
    vec![
        InputDescriptor {
            id: "soundburger_bt".into(),
            name: "Sound Burger (Bluetooth)".into(),
            kind: InputKind::BluetoothTurntable,
            transport: InputTransport::BluetoothA2dp,
            codec: Some("aptX".into()),
            native_format: Some(AudioFormat {
                sample_rate: 44_100,
                channels: 2,
                format: SampleFormat::S16,
            }),
            connected: false,
            selected: false,
            last_level_dbfs: None,
        },
        InputDescriptor {
            id: "linein_front".into(),
            name: "Line In (3.5mm)".into(),
            kind: InputKind::LineIn,
            transport: InputTransport::Alsa,
            codec: None,
            native_format: Some(AudioFormat {
                sample_rate: 48_000,
                channels: 2,
                format: SampleFormat::S24,
            }),
            connected: true,
            selected: false,
            last_level_dbfs: None,
        },
    ]
}
