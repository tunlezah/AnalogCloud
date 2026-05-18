//! Session manager.
//!
//! Owns the **single** active session. New requests atomically take
//! over the current session: arm → stop old → start new. There is never
//! more than one session in `playing` state at the same time.

use std::sync::Arc;

use parking_lot::Mutex;
use thiserror::Error;
use time::OffsetDateTime;
use tokio::sync::broadcast;
use uuid::Uuid;

use analog_cloud_device_discovery::DeviceCatalog;
use analog_cloud_media_engine::MediaEngine;
use analog_cloud_shared::events::{Event, SessionStopReason};
use analog_cloud_shared::output::{OutputKind, OutputTransport};
use analog_cloud_shared::session::{Session, SessionState, SessionStats, StartSessionRequest};

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("input not found: {0}")]
    InputNotFound(String),
    #[error("output not found: {0}")]
    OutputNotFound(String),
    #[error("media engine error: {0}")]
    Media(#[from] analog_cloud_media_engine::MediaError),
}

pub type Result<T> = std::result::Result<T, SessionError>;

const EVENT_CHANNEL_CAPACITY: usize = 256;

#[derive(Clone)]
pub struct SessionManager {
    inner: Arc<Inner>,
}

struct Inner {
    state: Mutex<State>,
    catalog: DeviceCatalog,
    media: MediaEngine,
    events: broadcast::Sender<Event>,
}

#[derive(Default)]
struct State {
    active: Option<Session>,
}

impl SessionManager {
    pub fn new(catalog: DeviceCatalog, media: MediaEngine) -> Self {
        let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State::default()),
                catalog,
                media,
                events: tx,
            }),
        }
    }

    /// Subscribe to backend events. Each subscriber gets every event
    /// fired after the call.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.inner.events.subscribe()
    }

    /// Returns the currently active session, if any.
    pub fn active(&self) -> Option<Session> {
        self.inner.state.lock().active.clone()
    }

    /// Start or take over with a new session.
    pub async fn start(&self, req: StartSessionRequest) -> Result<Session> {
        // Validate input + output exist in the catalog.
        let inputs = self.inner.catalog.inputs();
        if !inputs.iter().any(|i| i.id == req.input_id) {
            return Err(SessionError::InputNotFound(req.input_id));
        }
        let outputs = self.inner.catalog.outputs();
        let output = outputs
            .iter()
            .find(|o| o.kind == req.output_kind && o.name == req.output_device)
            .cloned()
            .ok_or_else(|| SessionError::OutputNotFound(req.output_device.clone()))?;

        // Stop existing session, if any. Drop the lock before awaiting.
        let previous = {
            let mut guard = self.inner.state.lock();
            guard.active.take()
        };
        if let Some(prev) = previous {
            self.emit(Event::SessionStopped {
                session_id: prev.session_id,
                reason: SessionStopReason::TakenOver,
            });
            self.inner.media.close_stream().await?;
        }

        let transport = transport_for(req.output_kind);
        let session = Session {
            session_id: Uuid::now_v7(),
            input_id: req.input_id.clone(),
            output_kind: req.output_kind,
            output_device: req.output_device.clone(),
            transport,
            started_at: OffsetDateTime::now_utc(),
            controller_clients: Vec::new(),
            state: SessionState::Armed,
            stats: SessionStats::default(),
        };

        self.emit(Event::SessionArmed { session: session.clone() });

        // Open the media graph.
        self.inner.media.open_stream(&req.input_id, &output.id).await?;

        let mut playing = session.clone();
        playing.state = SessionState::Playing;
        {
            let mut guard = self.inner.state.lock();
            guard.active = Some(playing.clone());
        }
        self.emit(Event::SessionStarted { session: playing.clone() });

        Ok(playing)
    }

    /// Stop the active session.
    pub async fn stop(&self, reason: SessionStopReason) -> Result<()> {
        let prev = {
            let mut guard = self.inner.state.lock();
            guard.active.take()
        };
        if let Some(prev) = prev {
            self.inner.media.close_stream().await?;
            self.emit(Event::SessionStopped {
                session_id: prev.session_id,
                reason,
            });
        }
        Ok(())
    }

    fn emit(&self, event: Event) {
        // It's fine if no one is listening.
        let _ = self.inner.events.send(event);
    }
}

fn transport_for(kind: OutputKind) -> OutputTransport {
    match kind {
        OutputKind::Browser => OutputTransport::WebRtcOpus,
        OutputKind::Local => OutputTransport::PipewireSink,
        OutputKind::Chromecast => OutputTransport::OpusHttp,
        OutputKind::AirPlay => OutputTransport::Raop,
    }
}
