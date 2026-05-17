//! Device discovery. Aggregates:
//!
//! * Audio inputs from the PipeWire graph (Bluetooth + ALSA).
//! * Output endpoints: a fixed `browser` slot, the host PipeWire sink,
//!   any mDNS-advertised Chromecast devices, and any AirPlay receivers.
//!
//! The discovery service is the canonical source of truth for what
//! `InputDescriptor`s and `OutputDescriptor`s exist on the network. It
//! does NOT decide which one is *active* — that is the session
//! manager's job.

use std::sync::Arc;

use parking_lot::RwLock;

use analog_cloud_shared::input::InputDescriptor;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

#[derive(Clone, Default)]
pub struct DeviceCatalog {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    inputs: Vec<InputDescriptor>,
    outputs: Vec<OutputDescriptor>,
}

impl DeviceCatalog {
    pub fn new() -> Self {
        let me = Self::default();
        me.seed_defaults();
        me
    }

    pub fn inputs(&self) -> Vec<InputDescriptor> {
        self.inner.read().inputs.clone()
    }

    pub fn outputs(&self) -> Vec<OutputDescriptor> {
        self.inner.read().outputs.clone()
    }

    pub fn upsert_input(&self, desc: InputDescriptor) {
        let mut inner = self.inner.write();
        if let Some(slot) = inner.inputs.iter_mut().find(|i| i.id == desc.id) {
            *slot = desc;
        } else {
            inner.inputs.push(desc);
        }
    }

    pub fn upsert_output(&self, desc: OutputDescriptor) {
        let mut inner = self.inner.write();
        if let Some(slot) = inner.outputs.iter_mut().find(|o| o.id == desc.id) {
            *slot = desc;
        } else {
            inner.outputs.push(desc);
        }
    }

    /// Spawn the background discovery loops (PipeWire, mDNS, RAOP).
    /// In the mock build this is a no-op; the seeded defaults stand.
    pub async fn start(&self) {
        let inputs = analog_cloud_media_engine::pipewire::enumerate_inputs().await;
        for input in inputs {
            self.upsert_input(input);
        }
        tracing::info!("device-discovery: enumerated inputs");
    }

    fn seed_defaults(&self) {
        // Browser and local outputs always exist.
        self.upsert_output(OutputDescriptor {
            id: "browser".into(),
            name: "This Browser".into(),
            kind: OutputKind::Browser,
            transport: OutputTransport::WebRtcOpus,
            state: OutputState::Idle,
            latency_ms_estimate: Some(120),
            active: false,
            reconnectable: false,
        });
        self.upsert_output(OutputDescriptor {
            id: "local".into(),
            name: "Local Speakers".into(),
            kind: OutputKind::Local,
            transport: OutputTransport::PipewireSink,
            state: OutputState::Idle,
            latency_ms_estimate: Some(40),
            active: false,
            reconnectable: true,
        });
    }
}
