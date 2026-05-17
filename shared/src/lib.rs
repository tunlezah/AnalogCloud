//! Shared types used across the backend, media engine, and serialized
//! to the frontend over HTTP / WebSocket / SSE.
//!
//! These types form the wire contract of Analog Cloud. Keep them small,
//! stable, and serde-compatible.

pub mod audio;
pub mod events;
pub mod input;
pub mod output;
pub mod session;
pub mod settings;
