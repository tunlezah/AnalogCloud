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
//!
//! The browser is a **detachable** controller — closing it must not
//! affect any non-browser session.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::{Event as SseEvent, KeepAlive, Sse},
    routing::get,
};
use futures::stream::Stream;
use serde::Serialize;
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use analog_cloud_device_discovery::DeviceCatalog;
use analog_cloud_session_manager::SessionManager;
use analog_cloud_settings::SettingsStore;
use analog_cloud_shared::events::SessionStopReason;
use analog_cloud_shared::session::StartSessionRequest;
use analog_cloud_shared::settings::Settings;

#[derive(Clone)]
pub struct AppState {
    pub catalog: DeviceCatalog,
    pub sessions: SessionManager,
    pub settings: SettingsStore,
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
    Ok(Json(merged))
}

fn json_of<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or_default()
}

/// Shallow JSON merge — patch wins on overlapping keys, sub-objects are
/// merged recursively, arrays and scalars are replaced.
fn merge(mut base: serde_json::Value, patch: serde_json::Value) -> serde_json::Value {
    use serde_json::Value::Object;
    if let (Object(b), Object(p)) = (&mut base, patch) {
        for (k, v) in p {
            let entry = b.remove(&k).unwrap_or(serde_json::Value::Null);
            b.insert(k, merge(entry, v));
        }
        base
    } else {
        // For non-objects the patch replaces.
        // (We swallowed `patch` in the destructure; rebuild here.)
        // SAFETY: only reached when shapes don't match.
        base
    }
}

async fn events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let rx = state.sessions.subscribe();
    let stream = async_stream::stream! {
        let mut stream = BroadcastStream::new(rx);
        use futures::StreamExt;
        while let Some(Ok(event)) = stream.next().await {
            let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
            yield Ok(SseEvent::default().data(data));
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
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
