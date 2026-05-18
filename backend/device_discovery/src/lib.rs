//! Device discovery. Aggregates:
//!
//! * Audio inputs from the PipeWire graph (Bluetooth + ALSA).
//! * Output endpoints: a fixed `browser` slot, the host PipeWire sink,
//!   any mDNS-advertised Chromecast devices (`_googlecast._tcp`),
//!   and any AirPlay receivers (`_raop._tcp`).
//!
//! The discovery service is the canonical source of truth for what
//! `InputDescriptor`s and `OutputDescriptor`s exist on the network. It
//! does NOT decide which one is *active* — that is the session
//! manager's job.

pub mod mdns;

use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::broadcast;

use analog_cloud_shared::input::InputDescriptor;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

/// Catalog change notification. Subscribers can observe inputs and
/// outputs appearing / disappearing.
#[derive(Clone, Debug)]
pub enum CatalogEvent {
    InputUpserted(InputDescriptor),
    InputRemoved(String),
    OutputUpserted(OutputDescriptor),
    OutputRemoved(String),
}

#[derive(Clone)]
pub struct DeviceCatalog {
    inner: Arc<Inner>,
}

struct Inner {
    state: RwLock<State>,
    events: broadcast::Sender<CatalogEvent>,
}

#[derive(Default)]
struct State {
    inputs: Vec<InputDescriptor>,
    outputs: Vec<OutputDescriptor>,
}

impl DeviceCatalog {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        let me = Self {
            inner: Arc::new(Inner {
                state: RwLock::new(State::default()),
                events: tx,
            }),
        };
        me.seed_defaults();
        me
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CatalogEvent> {
        self.inner.events.subscribe()
    }

    pub fn inputs(&self) -> Vec<InputDescriptor> {
        self.inner.state.read().inputs.clone()
    }

    pub fn outputs(&self) -> Vec<OutputDescriptor> {
        self.inner.state.read().outputs.clone()
    }

    pub fn upsert_input(&self, desc: InputDescriptor) {
        {
            let mut inner = self.inner.state.write();
            if let Some(slot) = inner.inputs.iter_mut().find(|i| i.id == desc.id) {
                *slot = desc.clone();
            } else {
                inner.inputs.push(desc.clone());
            }
        }
        let _ = self.inner.events.send(CatalogEvent::InputUpserted(desc));
    }

    pub fn upsert_output(&self, desc: OutputDescriptor) {
        {
            let mut inner = self.inner.state.write();
            if let Some(slot) = inner.outputs.iter_mut().find(|o| o.id == desc.id) {
                *slot = desc.clone();
            } else {
                inner.outputs.push(desc.clone());
            }
        }
        let _ = self.inner.events.send(CatalogEvent::OutputUpserted(desc));
    }

    pub fn remove_output(&self, id: &str) {
        let removed = {
            let mut inner = self.inner.state.write();
            let len_before = inner.outputs.len();
            inner.outputs.retain(|o| o.id != id);
            inner.outputs.len() != len_before
        };
        if removed {
            let _ = self.inner.events.send(CatalogEvent::OutputRemoved(id.into()));
        }
    }

    /// Spawn the background discovery loops (PipeWire enumeration and
    /// mDNS browse for Chromecast / AirPlay). Returns immediately; the
    /// loops run for the lifetime of the process.
    pub async fn start(&self) {
        // PipeWire one-shot enumeration.
        let inputs = analog_cloud_media_engine::pipewire::enumerate_inputs().await;
        for input in inputs {
            self.upsert_input(input);
        }
        tracing::info!("device-discovery: enumerated inputs");

        // mDNS discovery in the background.
        let catalog = self.clone();
        tokio::spawn(async move {
            if let Err(e) = mdns::run_discovery(catalog).await {
                tracing::warn!(error = %e, "mDNS discovery loop exited");
            }
        });
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

impl Default for DeviceCatalog {
    fn default() -> Self {
        Self::new()
    }
}
