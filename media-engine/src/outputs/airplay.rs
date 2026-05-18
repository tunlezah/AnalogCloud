//! AirPlay output. RAOP-compatible sender. Backend-owned and
//! reconnectable.

use async_trait::async_trait;

use super::OutputController;
use crate::Result;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

pub struct AirPlayOutput {
    descriptor: OutputDescriptor,
    pub device_name: String,
}

impl AirPlayOutput {
    pub fn new(device_name: impl Into<String>) -> Self {
        let device_name = device_name.into();
        Self {
            descriptor: OutputDescriptor {
                id: format!("airplay::{device_name}"),
                name: device_name.clone(),
                kind: OutputKind::AirPlay,
                transport: OutputTransport::Raop,
                state: OutputState::Idle,
                latency_ms_estimate: Some(2_000),
                active: false,
                reconnectable: true,
            },
            device_name,
        }
    }
}

#[async_trait]
impl OutputController for AirPlayOutput {
    fn kind(&self) -> OutputKind { OutputKind::AirPlay }
    fn descriptor(&self) -> OutputDescriptor { self.descriptor.clone() }
    async fn start(&self) -> Result<()> {
        tracing::info!(device = %self.device_name, "airplay: opening RAOP session");
        Ok(())
    }
    async fn stop(&self) -> Result<()> { Ok(()) }
    async fn reconnect(&self) -> Result<()> { Ok(()) }
}
