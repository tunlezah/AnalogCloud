//! Chromecast output. The cast device **pulls** an Opus-in-WebM (or
//! AAC) HTTP stream that the backend serves. We never push raw PCM.

use async_trait::async_trait;

use super::OutputController;
use crate::Result;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

pub struct ChromecastOutput {
    descriptor: OutputDescriptor,
    /// Friendly name of the cast device (e.g. "Living Room").
    pub device_name: String,
}

impl ChromecastOutput {
    pub fn new(device_name: impl Into<String>) -> Self {
        let device_name = device_name.into();
        Self {
            descriptor: OutputDescriptor {
                id: format!("chromecast::{device_name}"),
                name: device_name.clone(),
                kind: OutputKind::Chromecast,
                transport: OutputTransport::OpusHttp,
                state: OutputState::Idle,
                latency_ms_estimate: Some(1_500),
                active: false,
                reconnectable: true,
            },
            device_name,
        }
    }
}

#[async_trait]
impl OutputController for ChromecastOutput {
    fn kind(&self) -> OutputKind { OutputKind::Chromecast }
    fn descriptor(&self) -> OutputDescriptor { self.descriptor.clone() }
    async fn start(&self) -> Result<()> {
        tracing::info!(device = %self.device_name, "chromecast: launching cast app");
        Ok(())
    }
    async fn stop(&self) -> Result<()> { Ok(()) }
    async fn reconnect(&self) -> Result<()> { Ok(()) }
}
