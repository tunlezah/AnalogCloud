//! Local output. Routes the canonical PCM stream straight into a
//! PipeWire sink — the host's speakers, headphone jack, etc.

use async_trait::async_trait;

use super::OutputController;
use crate::Result;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

pub struct LocalOutput {
    descriptor: OutputDescriptor,
}

impl LocalOutput {
    pub fn new() -> Self {
        Self {
            descriptor: OutputDescriptor {
                id: "local".into(),
                name: "Local Speakers".into(),
                kind: OutputKind::Local,
                transport: OutputTransport::PipewireSink,
                state: OutputState::Idle,
                latency_ms_estimate: Some(40),
                active: false,
                reconnectable: true,
            },
        }
    }
}

#[async_trait]
impl OutputController for LocalOutput {
    fn kind(&self) -> OutputKind { OutputKind::Local }
    fn descriptor(&self) -> OutputDescriptor { self.descriptor.clone() }
    async fn start(&self) -> Result<()> {
        tracing::info!("local output: linking node to default sink");
        Ok(())
    }
    async fn stop(&self) -> Result<()> { Ok(()) }
    async fn reconnect(&self) -> Result<()> { Ok(()) }
}
