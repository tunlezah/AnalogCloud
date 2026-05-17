//! Analog Cloud daemon entry point.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

use analog_cloud_api::{router, AppState};
use analog_cloud_device_discovery::DeviceCatalog;
use analog_cloud_media_engine::outputs::browser::BrowserOutput;
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
    media.set_equalizer(settings.snapshot().equalizer);

    let catalog = DeviceCatalog::new();
    catalog.start().await;
    let sessions = SessionManager::new(catalog.clone(), media.clone());
    let browser_output = Arc::new(BrowserOutput::new());

    // Start the periodic level-sample emitter. In `mock` builds this
    // synthesises a slowly varying meter so the UI has motion; in
    // `real` builds the media engine pushes real samples directly.
    spawn_mock_level_loop(media.clone(), sessions.clone());

    let app = router(AppState {
        catalog,
        sessions,
        settings,
        media,
        browser_output,
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

/// Emits a synthetic StereoLevel sample at 5 Hz so the frontend's VU
/// meter has something to render even before a real audio source is
/// connected. Replaced by the real DSP pipeline in `real` builds.
fn spawn_mock_level_loop(media: MediaEngine, sessions: SessionManager) {
    use analog_cloud_media_engine::LevelSample;
    use analog_cloud_shared::audio::{ChannelLevel, StereoLevel};

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(200));
        let mut phase: f32 = 0.0;
        loop {
            tick.tick().await;
            let Some(active) = sessions.active() else {
                continue;
            };
            phase += 0.4;
            let l = -18.0 + 6.0 * phase.sin();
            let r = -18.0 + 6.0 * (phase + 0.7).sin();
            media.publish_level(LevelSample {
                input_id: active.input_id,
                level: StereoLevel {
                    left: ChannelLevel { peak_dbfs: l, rms_dbfs: l - 6.0 },
                    right: ChannelLevel { peak_dbfs: r, rms_dbfs: r - 6.0 },
                },
            });
        }
    });
}
