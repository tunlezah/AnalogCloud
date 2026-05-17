//! Browser output. WebRTC peer connection, ephemeral. The browser is
//! the only "output" the user can close from the client side.

use async_trait::async_trait;

use super::OutputController;
use crate::Result;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

pub struct BrowserOutput {
    descriptor: OutputDescriptor,
}

impl BrowserOutput {
    pub fn new() -> Self {
        Self {
            descriptor: OutputDescriptor {
                id: "browser".into(),
                name: "This Browser".into(),
                kind: OutputKind::Browser,
                transport: OutputTransport::WebRtcOpus,
                state: OutputState::Idle,
                latency_ms_estimate: Some(120),
                active: false,
                reconnectable: false,
            },
        }
    }
}

#[async_trait]
impl OutputController for BrowserOutput {
    fn kind(&self) -> OutputKind {
        OutputKind::Browser
    }
    fn descriptor(&self) -> OutputDescriptor {
        self.descriptor.clone()
    }
    async fn start(&self) -> Result<()> {
        tracing::info!("browser output: negotiating WebRTC offer");
        Ok(())
    }
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    async fn reconnect(&self) -> Result<()> {
        Ok(())
    }
}
