//! Browser output. WebRTC peer connection, ephemeral. The browser is
//! the only "output" the user can close from the client side.
//!
//! The flow:
//!
//! 1. Browser POSTs an SDP **offer** to `/api/webrtc/offer`.
//! 2. The backend creates a `RTCPeerConnection`, adds an outbound Opus
//!    `TrackLocalStaticSample`, sets the remote description, generates
//!    its **answer**, returns it.
//! 3. The session's analysis loop writes encoded Opus frames into the
//!    track; the browser plays them out.
//!
//! Closing the tab tears down the peer connection — but ONLY this
//! ephemeral browser session. Any non-browser sessions are unaffected,
//! because the session manager owns them and they live in the backend.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use super::OutputController;
use crate::{MediaError, Result};
use analog_cloud_shared::output::{OutputDescriptor, OutputKind, OutputState, OutputTransport};

pub struct BrowserOutput {
    descriptor: OutputDescriptor,
    peer: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
    track: Arc<Mutex<Option<Arc<TrackLocalStaticSample>>>>,
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
            peer: Arc::new(Mutex::new(None)),
            track: Arc::new(Mutex::new(None)),
        }
    }

    /// Accept an SDP offer from the browser and return our answer.
    /// Performs the standard "RTCPeerConnection negotiation needed"
    /// dance.
    pub async fn handle_offer(&self, offer_sdp: String) -> Result<String> {
        let mut me = MediaEngine::default();
        me.register_default_codecs()
            .map_err(|e| MediaError::Stack(format!("register codecs: {e}")))?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut me)
            .map_err(|e| MediaError::Stack(format!("register interceptors: {e}")))?;

        let api = APIBuilder::new()
            .with_media_engine(me)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| MediaError::Stack(format!("new_peer_connection: {e}")))?,
        );

        let track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: 48_000,
                channels: 2,
                ..Default::default()
            },
            "audio".to_owned(),
            "analog-cloud".to_owned(),
        ));

        peer.add_track(Arc::clone(&track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| MediaError::Stack(format!("add_track: {e}")))?;

        let offer = RTCSessionDescription::offer(offer_sdp)
            .map_err(|e| MediaError::Stack(format!("parse offer: {e}")))?;
        peer.set_remote_description(offer)
            .await
            .map_err(|e| MediaError::Stack(format!("set_remote_description: {e}")))?;

        let answer = peer
            .create_answer(None)
            .await
            .map_err(|e| MediaError::Stack(format!("create_answer: {e}")))?;

        let mut gather_complete = peer.gathering_complete_promise().await;
        peer.set_local_description(answer)
            .await
            .map_err(|e| MediaError::Stack(format!("set_local_description: {e}")))?;
        // Wait for ICE gathering so we ship a complete SDP (no trickle).
        let _ = gather_complete.recv().await;

        let local = peer
            .local_description()
            .await
            .ok_or_else(|| MediaError::Stack("no local description after gathering".into()))?;

        *self.peer.lock().await = Some(Arc::clone(&peer));
        *self.track.lock().await = Some(track);

        tracing::info!("browser output: WebRTC peer connection up");
        Ok(local.sdp)
    }

    pub async fn tear_down(&self) {
        if let Some(peer) = self.peer.lock().await.take() {
            let _ = peer.close().await;
        }
        *self.track.lock().await = None;
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
        tracing::info!("browser output: awaiting WebRTC offer");
        Ok(())
    }
    async fn stop(&self) -> Result<()> {
        self.tear_down().await;
        Ok(())
    }
    async fn reconnect(&self) -> Result<()> {
        self.tear_down().await;
        Ok(())
    }
}
