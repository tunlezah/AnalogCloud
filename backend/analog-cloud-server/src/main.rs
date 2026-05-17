//! Analog Cloud daemon entry point.

use std::net::SocketAddr;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

use analog_cloud_api::{AppState, router};
use analog_cloud_device_discovery::DeviceCatalog;
use analog_cloud_media_engine::MediaEngine;
use analog_cloud_session_manager::SessionManager;
use analog_cloud_settings::SettingsStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "analog-cloud starting");

    let settings = SettingsStore::open_default()
        .await
        .context("opening settings store")?;
    let media = MediaEngine::start().await.context("starting media engine")?;
    let catalog = DeviceCatalog::new();
    catalog.start().await;
    let sessions = SessionManager::new(catalog.clone(), media.clone());

    let app = router(AppState {
        catalog,
        sessions,
        settings,
    });

    let bind: SocketAddr = std::env::var("ANALOG_CLOUD_BIND")
        .unwrap_or_else(|_| "127.0.0.1:7777".to_string())
        .parse()
        .context("invalid ANALOG_CLOUD_BIND")?;

    tracing::info!(%bind, "http listener up");
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,analog_cloud=debug"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .init();
}
