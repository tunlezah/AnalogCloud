//! Optional online metadata enrichment.
//!
//! Provider: MusicBrainz + AcoustID. The enricher fingerprints the
//! canonical PCM stream and resolves the result to a release / track /
//! artist. Behavior is governed by `EnrichmentMode`:
//!
//! * `Off`        — never fingerprint.
//! * `Immediate`  — fingerprint on input lock.
//! * `Delayed`    — fingerprint after `wait_seconds` of stable signal.
//! * `Manual`     — only on user request.
//!
//! Enrichment **never** affects playback. Metadata is purely
//! informational.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u16>,
    pub artwork_url: Option<String>,
    pub musicbrainz_recording_id: Option<String>,
}

/// Marker for the enrichment service. Real implementation will own an
/// HTTP client, a fingerprinter, and a small SQLite cache.
#[derive(Clone, Default)]
pub struct MetadataService;

impl MetadataService {
    pub fn new() -> Self {
        Self
    }

    /// Resolve a fingerprint to metadata. Returns `None` if no match.
    pub async fn resolve_fingerprint(&self, _fingerprint: &str) -> Option<TrackMetadata> {
        None
    }
}
