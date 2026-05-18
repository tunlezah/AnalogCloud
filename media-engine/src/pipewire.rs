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
pub async fn enumerate_inputs() -> Vec<InputDescriptor> {
    #[cfg(feature = "real")]
    {
        // Real enumeration runs in a blocking thread because pipewire's
        // mainloop is synchronous.
        if let Some(inputs) = tokio::task::spawn_blocking(real::enumerate)
            .await
            .ok()
            .and_then(|r| r.ok())
        {
            return inputs;
        }
        tracing::warn!("pipewire real enumeration failed; falling back to mock topology");
        return mock_inputs();
    }
    #[cfg(not(feature = "real"))]
    {
        mock_inputs()
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

#[cfg(feature = "real")]
mod real {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use anyhow::{Context, Result};
    use pipewire::context::Context as PwContext;
    use pipewire::main_loop::MainLoop;
    use pipewire::types::ObjectType;

    use analog_cloud_shared::audio::{AudioFormat, SampleFormat};
    use analog_cloud_shared::input::{InputDescriptor, InputKind, InputTransport};

    pub fn enumerate() -> Result<Vec<InputDescriptor>> {
        pipewire::init();

        let mainloop = MainLoop::new(None).context("pipewire MainLoop::new")?;
        let context = PwContext::new(&mainloop).context("pipewire Context::new")?;
        let core = context.connect(None).context("pipewire connect")?;
        let registry = core.get_registry().context("pipewire get_registry")?;

        let nodes: Rc<RefCell<Vec<InputDescriptor>>> = Rc::new(RefCell::new(Vec::new()));
        let nodes_in = nodes.clone();

        let _listener = registry
            .add_listener_local()
            .global(move |global| {
                if global.type_ != ObjectType::Node {
                    return;
                }
                let Some(props) = global.props.as_ref() else { return };

                let media_class = props.get("media.class").unwrap_or("");
                if !is_audio_source(media_class) {
                    return;
                }

                let id = props
                    .get("node.name")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("pw_node_{}", global.id));
                let name = props
                    .get("node.description")
                    .or_else(|| props.get("node.nick"))
                    .or_else(|| props.get("node.name"))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| id.clone());

                let (kind, transport) = classify(media_class, props.get("device.api"), &name);
                let native_format = parse_format(props.get("audio.rate"), props.get("audio.format"));

                nodes_in.borrow_mut().push(InputDescriptor {
                    id,
                    name,
                    kind,
                    transport,
                    codec: props.get("audio.codec").map(|s| s.to_string()),
                    native_format,
                    connected: true,
                    selected: false,
                    last_level_dbfs: None,
                });
            })
            .register();

        // Round-trip the registry to flush initial globals, then exit
        // the loop.
        let timer = mainloop.loop_().add_timer({
            let ml = mainloop.clone();
            move |_| ml.quit()
        });
        timer
            .update_timer(Some(Duration::from_millis(150)), None)
            .into_result()
            .ok();

        mainloop.run();

        Ok(nodes.take())
    }

    fn is_audio_source(media_class: &str) -> bool {
        matches!(
            media_class,
            "Audio/Source" | "Audio/Source/Virtual" | "Stream/Output/Audio"
        )
    }

    fn classify(media_class: &str, api: Option<&str>, name: &str) -> (InputKind, InputTransport) {
        let lname = name.to_ascii_lowercase();
        if api == Some("bluez5") || lname.contains("bluetooth") || lname.contains("a2dp") {
            let kind = if lname.contains("turntable") || lname.contains("burger") {
                InputKind::BluetoothTurntable
            } else if lname.contains("cassette") || lname.contains("tape") {
                InputKind::BluetoothCassette
            } else {
                InputKind::BluetoothGeneric
            };
            return (kind, InputTransport::BluetoothA2dp);
        }
        if api == Some("alsa") || media_class == "Audio/Source" {
            return (InputKind::LineIn, InputTransport::Alsa);
        }
        (InputKind::Virtual, InputTransport::PipewireNode)
    }

    fn parse_format(rate: Option<&str>, format: Option<&str>) -> Option<AudioFormat> {
        let rate = rate?.parse::<u32>().ok()?;
        let format = match format.unwrap_or("F32") {
            f if f.contains("F32") => SampleFormat::F32,
            f if f.contains("S24") => SampleFormat::S24,
            f if f.contains("S32") => SampleFormat::S32,
            _ => SampleFormat::S16,
        };
        Some(AudioFormat {
            sample_rate: rate,
            channels: 2,
            format,
        })
    }
}
