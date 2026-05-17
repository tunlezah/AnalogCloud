//! Analog Cloud HTTP / WebSocket / SSE API.
//!
//! Routes:
//!
//! | Method | Path                       | Description                       |
//! |--------|----------------------------|-----------------------------------|
//! | GET    | `/api/health`              | liveness probe                    |
//! | GET    | `/api/state`               | full debug snapshot               |
//! | GET    | `/api/inputs`              | list discovered inputs            |
//! | GET    | `/api/outputs`             | list discovered outputs           |
//! | GET    | `/api/session`             | currently active session, if any  |
//! | POST   | `/api/session`             | start or take over a session      |
//! | DELETE | `/api/session`             | stop active session               |
//! | GET    | `/api/settings`            | current settings                  |
//! | PATCH  | `/api/settings`            | update settings                   |
//! | GET    | `/api/events`              | SSE event stream                  |
//! | POST   | `/api/webrtc/offer`        | submit SDP offer, get answer      |
//! | GET    | `/api/cast/stream.webm`    | Opus-in-WebM pull stream for Cast |
//!
//! The browser is a **detachable** controller — closing it must not
//! affect any non-browser session.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::get,
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use analog_cloud_device_discovery::DeviceCatalog;
use analog_cloud_media_engine::outputs::browser::BrowserOutput;
use analog_cloud_media_engine::MediaEngine;
use analog_cloud_session_manager::SessionManager;
use analog_cloud_settings::SettingsStore;
use analog_cloud_shared::events::{Event, SessionStopReason};
use analog_cloud_shared::session::StartSessionRequest;
use analog_cloud_shared::settings::Settings;

#[derive(Clone)]
pub struct AppState {
    pub catalog: DeviceCatalog,
    pub sessions: SessionManager,
    pub settings: SettingsStore,
    pub media: MediaEngine,
    pub browser_output: Arc<BrowserOutput>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/state", get(debug_state))
        .route("/api/inputs", get(list_inputs))
        .route("/api/outputs", get(list_outputs))
        .route(
            "/api/session",
            get(current_session).post(start_session).delete(stop_session),
        )
        .route("/api/settings", get(get_settings).patch(patch_settings))
        .route("/api/events", get(events))
        .route("/api/webrtc/offer", axum::routing::post(webrtc_offer))
        .route("/api/cast/stream.webm", get(cast_stream))
        .with_state(Arc::new(state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn debug_state(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "inputs":   state.catalog.inputs(),
        "outputs":  state.catalog.outputs(),
        "session":  state.sessions.active(),
        "settings": state.settings.snapshot(),
    }))
}

async fn list_inputs(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!(state.catalog.inputs()))
}

async fn list_outputs(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!(state.catalog.outputs()))
}

async fn current_session(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!(state.sessions.active()))
}

async fn start_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartSessionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = state
        .sessions
        .start(req)
        .await
        .map_err(ApiError::from_session)?;
    Ok(Json(json!(session)))
}

async fn stop_session(State(state): State<Arc<AppState>>) -> Result<StatusCode, ApiError> {
    state
        .sessions
        .stop(SessionStopReason::UserRequested)
        .await
        .map_err(ApiError::from_session)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_settings(State(state): State<Arc<AppState>>) -> Json<Settings> {
    Json(state.settings.snapshot())
}

async fn patch_settings(
    State(state): State<Arc<AppState>>,
    Json(patch): Json<serde_json::Value>,
) -> Result<Json<Settings>, ApiError> {
    let merged = state
        .settings
        .update(|s| {
            if let Ok(new) = serde_json::from_value::<Settings>(merge(json_of(s), patch)) {
                *s = new;
            }
        })
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // Propagate EQ changes into the media engine.
    state.media.set_equalizer(merged.equalizer.clone());
    Ok(Json(merged))
}

fn json_of<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or_default()
}

/// Recursive JSON merge — patch wins on overlapping keys, sub-objects
/// are merged recursively, arrays and scalars are replaced wholesale.
fn merge(base: serde_json::Value, patch: serde_json::Value) -> serde_json::Value {
    use serde_json::Value::Object;
    match (base, patch) {
        (Object(mut b), Object(p)) => {
            for (k, v) in p {
                let entry = b.remove(&k).unwrap_or(serde_json::Value::Null);
                b.insert(k, merge(entry, v));
            }
            Object(b)
        }
        // Arrays and scalars: the patch replaces the base.
        (_, patch) => patch,
    }
}

async fn events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let session_rx = state.sessions.subscribe();
    let level_rx = state.media.subscribe_levels();
    let catalog_rx = state.catalog.subscribe();

    let stream = async_stream::stream! {
        use futures::StreamExt;
        let mut session_stream = BroadcastStream::new(session_rx);
        let mut level_stream = BroadcastStream::new(level_rx);
        let mut catalog_stream = BroadcastStream::new(catalog_rx);

        loop {
            tokio::select! {
                Some(Ok(event)) = session_stream.next() => {
                    if let Ok(data) = serde_json::to_string(&event) {
                        yield Ok(SseEvent::default().data(data));
                    }
                }
                Some(Ok(sample)) = level_stream.next() => {
                    let event = Event::Levels {
                        input_id: sample.input_id,
                        level: sample.level,
                    };
                    if let Ok(data) = serde_json::to_string(&event) {
                        yield Ok(SseEvent::default().data(data));
                    }
                }
                Some(Ok(cat)) = catalog_stream.next() => {
                    use analog_cloud_device_discovery::CatalogEvent;
                    let event = match cat {
                        CatalogEvent::InputUpserted(i) => Event::InputUpdated { input: i },
                        CatalogEvent::InputRemoved(id) => Event::InputRemoved { input_id: id },
                        CatalogEvent::OutputUpserted(o) => Event::OutputUpdated { output: o },
                        CatalogEvent::OutputRemoved(id) => Event::OutputRemoved { output_id: id },
                    };
                    if let Ok(data) = serde_json::to_string(&event) {
                        yield Ok(SseEvent::default().data(data));
                    }
                }
                else => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Deserialize)]
struct WebRtcOffer {
    sdp: String,
}

#[derive(Serialize)]
struct WebRtcAnswer {
    sdp: String,
}

async fn webrtc_offer(
    State(state): State<Arc<AppState>>,
    Json(offer): Json<WebRtcOffer>,
) -> Result<Json<WebRtcAnswer>, ApiError> {
    let answer_sdp = state
        .browser_output
        .handle_offer(offer.sdp)
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(WebRtcAnswer { sdp: answer_sdp }))
}

/// Chromecast pulls audio from this endpoint as Opus-in-WebM. The body
/// is a streaming response — the cast device keeps the connection open
/// and reads frames as they're produced.
///
/// This scaffold returns a 503 until the real GStreamer encoder is
/// wired (Phase 1 still — the active stream needs to be coupled to the
/// HTTP body sink).
async fn cast_stream(State(_state): State<Arc<AppState>>) -> Response {
    // TODO(phase-1): pipe the active session's canonical PCM through an
    // Opus encoder and into the response body via an mpsc -> Body bridge.
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [(header::CONTENT_TYPE, "text/plain")],
        "cast stream endpoint scaffolded; encoder not yet wired",
    )
        .into_response()
}

#[derive(Debug)]
pub struct ApiError(StatusCode, String);

impl ApiError {
    fn from_session(err: analog_cloud_session_manager::SessionError) -> Self {
        use analog_cloud_session_manager::SessionError::*;
        match err {
            InputNotFound(id) => Self(StatusCode::NOT_FOUND, format!("input not found: {id}")),
            OutputNotFound(id) => Self(StatusCode::NOT_FOUND, format!("output not found: {id}")),
            Media(e) => Self(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_replaces_scalars() {
        let base = json!({"a": 1, "b": 2});
        let patch = json!({"b": 99});
        assert_eq!(merge(base, patch), json!({"a": 1, "b": 99}));
    }

    #[test]
    fn merge_replaces_arrays_wholesale() {
        let base = json!({"bands": [0, 0, 0]});
        let patch = json!({"bands": [3, 2, 1]});
        assert_eq!(merge(base, patch), json!({"bands": [3, 2, 1]}));
    }

    #[test]
    fn merge_recurses_into_sub_objects() {
        let base = json!({"eq": {"enabled": false, "bands": [0, 0]}});
        let patch = json!({"eq": {"enabled": true}});
        assert_eq!(
            merge(base, patch),
            json!({"eq": {"enabled": true, "bands": [0, 0]}}),
        );
    }
}
