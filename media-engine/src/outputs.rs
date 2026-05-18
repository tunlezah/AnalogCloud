//! Output controllers — one per destination kind. Each owns the
//! lifecycle of a single backend-resident stream.

use async_trait::async_trait;

use crate::Result;
use analog_cloud_shared::output::{OutputDescriptor, OutputKind};

/// A backend-owned output. The session manager talks to outputs via
/// this trait. Browser outputs implement it too, even though they are
/// ephemeral.
#[async_trait]
pub trait OutputController: Send + Sync {
    fn kind(&self) -> OutputKind;
    fn descriptor(&self) -> OutputDescriptor;

    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn reconnect(&self) -> Result<()>;
}

pub mod browser;
pub mod local;
pub mod chromecast;
pub mod airplay;
