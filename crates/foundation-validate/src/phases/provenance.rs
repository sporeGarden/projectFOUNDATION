// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phases 2 & 7: Provenance session lifecycle (open/commit).
//!
//! Gracefully degrades if `rhizoCrypt` is unreachable. The pipeline records
//! degradation rather than aborting — science never gated behind provenance.

use foundation_core::config::DiscoveryConfig;
use foundation_core::primal_names;
use foundation_ipc::PrimalClient;
use tracing::{info, warn};

/// Sentinel prefix indicating a live session was opened.
const SESSION_PREFIX: &str = "session:";

/// Status of a provenance session throughout the pipeline.
#[derive(Debug, Clone)]
pub enum SessionStatus {
    /// Session successfully opened with an ID.
    Active(String),
    /// Session could not be opened — reason preserved.
    Degraded(String),
}

impl SessionStatus {
    /// Whether the session is active and can be committed.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active(_))
    }

    /// Get the session ID if active.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Active(id) => Some(id),
            Self::Degraded(_) => None,
        }
    }

    /// Human-readable summary for reporting.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Active(id) => format!("{SESSION_PREFIX}{id}"),
            Self::Degraded(reason) => format!("degraded: {reason}"),
        }
    }
}

/// Phase 2: Open a provenance session via `rhizoCrypt`.
///
/// Returns [`SessionStatus::Active`] on success or [`SessionStatus::Degraded`]
/// with reason on failure. Never panics.
pub async fn open_session(config: &DiscoveryConfig, gate_name: &str) -> SessionStatus {
    let client = match PrimalClient::discover(primal_names::slugs::RHIZOCRYPT, config) {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "rhizoCrypt unreachable — provenance degraded");
            return SessionStatus::Degraded(String::from("rhizoCrypt unreachable"));
        }
    };

    match client
        .call_raw(
            "dag.session.create",
            Some(serde_json::json!({
                "gate": gate_name,
                "purpose": "validation"
            })),
        )
        .await
    {
        Ok(resp) => {
            let session_id = resp
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned();
            info!(session_id = %session_id, "provenance session opened");
            SessionStatus::Active(session_id)
        }
        Err(e) => {
            warn!(error = %e, "provenance session open failed — degraded");
            SessionStatus::Degraded(String::from("rhizoCrypt RPC failed"))
        }
    }
}

/// Phase 7: Commit a provenance session via `rhizoCrypt`.
///
/// Only attempts commit if the session is active. Gracefully degrades on failure.
pub async fn commit_session(
    config: &DiscoveryConfig,
    session: &SessionStatus,
    gate_name: &str,
) -> String {
    let Some(session_id) = session.session_id() else {
        return session.summary();
    };

    let client = match PrimalClient::discover(primal_names::slugs::RHIZOCRYPT, config) {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "rhizoCrypt unreachable for commit");
            return format!("degraded: commit skipped ({session_id})");
        }
    };

    match client
        .call_raw(
            "dag.session.commit",
            Some(serde_json::json!({
                "session_id": session_id,
                "gate": gate_name,
            })),
        )
        .await
    {
        Ok(_) => {
            info!(session_id, "provenance committed");
            format!("committed:{session_id}")
        }
        Err(e) => {
            warn!(session_id, error = %e, "provenance commit failed");
            format!("commit_failed:{session_id}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn session_status_active() {
        let status = SessionStatus::Active(String::from("abc123"));
        assert!(status.is_active());
        assert_eq!(status.session_id(), Some("abc123"));
        assert_eq!(status.summary(), "session:abc123");
    }

    #[test]
    fn session_status_degraded() {
        let status = SessionStatus::Degraded(String::from("no connection"));
        assert!(!status.is_active());
        assert_eq!(status.session_id(), None);
        assert_eq!(status.summary(), "degraded: no connection");
    }

    #[tokio::test]
    async fn open_session_degrades_when_rhizocrypt_unreachable() {
        let config = DiscoveryConfig {
            metadata: None,
            sockets: HashMap::new(),
            bootstrap_tcp: None,
        };

        let result = open_session(&config, "test-gate").await;
        assert!(!result.is_active());
    }

    #[tokio::test]
    async fn commit_skipped_when_degraded() {
        let config = DiscoveryConfig {
            metadata: None,
            sockets: HashMap::new(),
            bootstrap_tcp: None,
        };
        let session = SessionStatus::Degraded(String::from("test"));
        let result = commit_session(&config, &session, "test-gate").await;
        assert!(result.starts_with("degraded:"));
    }
}
