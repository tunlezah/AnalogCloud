//! mDNS / DNS-SD discovery for Chromecast (`_googlecast._tcp.local.`)
//! and AirPlay receivers (`_raop._tcp.local.`).
//!
//! We listen for both service types in parallel, upserting matching
//! `OutputDescriptor`s into the device catalog as they appear and
//! removing them when they go away.

use anyhow::{Context, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent};

use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

use crate::DeviceCatalog;

const CAST_SERVICE: &str = "_googlecast._tcp.local.";
const RAOP_SERVICE: &str = "_raop._tcp.local.";

pub async fn run_discovery(catalog: DeviceCatalog) -> Result<()> {
    let daemon = ServiceDaemon::new().context("creating mDNS daemon")?;

    let cast_rx = daemon.browse(CAST_SERVICE).context("browse cast")?;
    let raop_rx = daemon.browse(RAOP_SERVICE).context("browse raop")?;

    tracing::info!("mDNS browsing for {CAST_SERVICE} and {RAOP_SERVICE}");

    // mdns-sd uses flume channels. We poll each in a dedicated blocking
    // task so we don't have to bridge channel types — tokio happily
    // hosts these on its blocking pool.
    let cast_catalog = catalog.clone();
    tokio::task::spawn_blocking(move || handle_loop(cast_rx, OutputKind::Chromecast, cast_catalog));

    let raop_catalog = catalog.clone();
    tokio::task::spawn_blocking(move || handle_loop(raop_rx, OutputKind::AirPlay, raop_catalog));

    Ok(())
}

fn handle_loop(rx: mdns_sd::Receiver<ServiceEvent>, kind: OutputKind, catalog: DeviceCatalog) {
    let prefix = match kind {
        OutputKind::Chromecast => "chromecast",
        OutputKind::AirPlay => "airplay",
        _ => return,
    };
    let transport = match kind {
        OutputKind::Chromecast => OutputTransport::OpusHttp,
        OutputKind::AirPlay => OutputTransport::Raop,
        _ => return,
    };
    let latency_ms_estimate = Some(match kind {
        OutputKind::Chromecast => 1_500,
        OutputKind::AirPlay => 2_000,
        _ => 1_000,
    });

    while let Ok(event) = rx.recv() {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                let device = friendly_name(info.get_fullname())
                    .unwrap_or_else(|| info.get_hostname().trim_end_matches('.').to_string());
                let id = format!("{prefix}::{device}");
                catalog.upsert_output(OutputDescriptor {
                    id,
                    name: device,
                    kind,
                    transport,
                    state: OutputState::Idle,
                    latency_ms_estimate,
                    active: false,
                    reconnectable: true,
                });
            }
            ServiceEvent::ServiceRemoved(_ty, fullname) => {
                let device = friendly_name(&fullname).unwrap_or(fullname);
                catalog.remove_output(&format!("{prefix}::{device}"));
            }
            _ => {}
        }
    }
}

/// `My Device._googlecast._tcp.local.` → `My Device`.
fn friendly_name(fullname: &str) -> Option<String> {
    let dot = fullname.find('.')?;
    Some(fullname[..dot].to_string())
}
