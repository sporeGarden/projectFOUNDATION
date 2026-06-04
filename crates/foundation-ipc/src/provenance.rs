// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance trio operations — rhizoCrypt + loamSpine + sweetGrass.
//!
//! Replaces the bash `|| true` 4-call pattern with typed, error-surfacing
//! provenance session management. Partial provenance is valid (DAG-only or
//! DAG+spine without braid) per the trio partial completion policy.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::client::PrimalClient;

/// A provenance session spanning all three trio primals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceSession {
    /// rhizoCrypt DAG session ID.
    pub dag_session_id: Option<String>,
    /// loamSpine entry ID.
    pub spine_entry_id: Option<String>,
    /// sweetGrass braid ID.
    pub braid_id: Option<String>,
    /// Which trio members were reachable.
    pub available: TrioAvailability,
}

/// Which members of the provenance trio are available.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TrioAvailability {
    /// rhizoCrypt (DAG).
    pub dag: bool,
    /// loamSpine (ledger).
    pub spine: bool,
    /// sweetGrass (attribution).
    pub attribution: bool,
}

impl TrioAvailability {
    /// Full trio available.
    #[must_use]
    pub const fn full(&self) -> bool {
        self.dag && self.spine && self.attribution
    }

    /// At least DAG is available (minimum for provenance).
    #[must_use]
    pub const fn has_dag(&self) -> bool {
        self.dag
    }
}

impl ProvenanceSession {
    /// Create a new provenance session by initializing all available trio members.
    ///
    /// This replaces the bash Phase 2 pattern. Failures on individual trio members
    /// produce warnings (degraded provenance) rather than silent `|| true`.
    pub async fn create(
        session_name: &str,
        rhizocrypt: Option<&PrimalClient>,
        loamspine: Option<&PrimalClient>,
        sweetgrass: Option<&PrimalClient>,
    ) -> Self {
        let mut session = Self {
            dag_session_id: None,
            spine_entry_id: None,
            braid_id: None,
            available: TrioAvailability::default(),
        };

        if let Some(rc) = rhizocrypt {
            match rc
                .call_raw(
                    crate::methods::dag::SESSION_CREATE,
                    Some(json!({
                        "session_name": session_name,
                        "session_type": "foundation_validation"
                    })),
                )
                .await
            {
                Ok(result) => {
                    session.dag_session_id = result
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    session.available.dag = true;
                    info!(session_id = ?session.dag_session_id, "DAG session created");
                }
                Err(e) => {
                    warn!(error = %e, "rhizoCrypt DAG session creation failed — continuing without DAG");
                }
            }
        }

        if let Some(ls) = loamspine {
            match ls
                .call_raw(
                    crate::methods::entry::CREATE,
                    Some(json!({
                        "entry_type": "foundation_validation",
                        "session_name": session_name
                    })),
                )
                .await
            {
                Ok(result) => {
                    session.spine_entry_id = result
                        .get("entry_id")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    session.available.spine = true;
                    info!(entry_id = ?session.spine_entry_id, "spine entry created");
                }
                Err(e) => {
                    warn!(error = %e, "loamSpine entry creation failed — continuing without spine");
                }
            }
        }

        if let Some(sg) = sweetgrass {
            session.available.attribution = sg
                .call_raw(crate::methods::health::LIVENESS, Some(json!({})))
                .await
                .is_ok();
            if !session.available.attribution {
                warn!("sweetGrass unreachable — continuing without attribution braid");
            }
        }

        session
    }

    /// Commit provenance at the end of a validation run (Phase 7).
    ///
    /// Commits the DAG session, appends to the spine, and creates the braid.
    /// Each step that fails produces a warning rather than aborting.
    pub async fn commit(
        &mut self,
        merkle_root: &str,
        rhizocrypt: Option<&PrimalClient>,
        loamspine: Option<&PrimalClient>,
        sweetgrass: Option<&PrimalClient>,
    ) -> CommitResult {
        let mut result = CommitResult::default();

        if let (Some(rc), Some(session_id)) = (rhizocrypt, &self.dag_session_id) {
            match rc
                .call_raw(
                    crate::methods::dag::SESSION_COMPLETE,
                    Some(json!({
                        "session_id": session_id,
                        "merkle_root": merkle_root
                    })),
                )
                .await
            {
                Ok(_) => {
                    result.dag_committed = true;
                    info!("DAG session committed with merkle root");
                }
                Err(e) => {
                    warn!(error = %e, "DAG session commit failed");
                }
            }
        }

        if let (Some(ls), Some(entry_id)) = (loamspine, &self.spine_entry_id) {
            let params = spine_commit_params(entry_id, merkle_root);
            match ls
                .call_raw(crate::methods::entry::APPEND, Some(params))
                .await
            {
                Ok(_) => {
                    result.spine_committed = true;
                    info!("spine entry committed");
                }
                Err(e) => {
                    warn!(error = %e, "spine entry commit failed");
                }
            }
        }

        if let Some(sg) = sweetgrass {
            if self.available.attribution {
                match sg
                    .call_raw(
                        crate::methods::braid::CREATE,
                        Some(json!({
                            "session_id": self.dag_session_id,
                            "spine_entry_id": self.spine_entry_id,
                            "merkle_root": merkle_root,
                            "attestation_type": "foundation_validation"
                        })),
                    )
                    .await
                {
                    Ok(resp) => {
                        self.braid_id = resp
                            .get("braid_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        result.braid_created = true;
                        info!(braid_id = ?self.braid_id, "attribution braid created");
                    }
                    Err(e) => {
                        warn!(error = %e, "sweetGrass braid creation failed");
                    }
                }
            }
        }

        result
    }
}

/// Build spine commit params with BLAKE3 hash as hex (not byte array).
fn spine_commit_params(entry_id: &str, merkle_root: &str) -> Value {
    json!({
        "entry_id": entry_id,
        "data_hash": merkle_root,
        "entry_type": "SessionCommit",
        "metadata": {
            "source": "foundation_validation",
            "hash_format": "hex"
        }
    })
}

/// Result of a provenance commit operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CommitResult {
    /// Whether the DAG session was committed.
    pub dag_committed: bool,
    /// Whether the spine entry was committed.
    pub spine_committed: bool,
    /// Whether the attribution braid was created.
    pub braid_created: bool,
}

impl CommitResult {
    /// Whether full provenance was achieved.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.dag_committed && self.spine_committed && self.braid_created
    }

    /// Human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for CommitResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: &[Option<&str>] = &[
            self.dag_committed.then_some("DAG"),
            self.spine_committed.then_some("spine"),
            self.braid_created.then_some("braid"),
        ];

        let mut first = true;
        for part in parts.iter().flatten() {
            if first {
                f.write_str("committed: ")?;
                first = false;
            } else {
                f.write_str(" + ")?;
            }
            f.write_str(part)?;
        }

        if first {
            f.write_str("no provenance committed")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trio_availability_full() {
        let trio = TrioAvailability {
            dag: true,
            spine: true,
            attribution: true,
        };
        assert!(trio.full());
        assert!(trio.has_dag());
    }

    #[test]
    fn trio_availability_partial() {
        let trio = TrioAvailability {
            dag: true,
            spine: false,
            attribution: false,
        };
        assert!(!trio.full());
        assert!(trio.has_dag());
    }

    #[test]
    fn trio_availability_empty() {
        let trio = TrioAvailability::default();
        assert!(!trio.full());
        assert!(!trio.has_dag());
    }

    #[test]
    fn commit_result_full() {
        let result = CommitResult {
            dag_committed: true,
            spine_committed: true,
            braid_created: true,
        };
        assert!(result.is_full());
        assert_eq!(result.summary(), "committed: DAG + spine + braid");
    }

    #[test]
    fn commit_result_partial() {
        let result = CommitResult {
            dag_committed: true,
            spine_committed: true,
            braid_created: false,
        };
        assert!(!result.is_full());
        assert_eq!(result.summary(), "committed: DAG + spine");
    }

    #[test]
    fn commit_result_empty() {
        let result = CommitResult::default();
        assert!(!result.is_full());
        assert_eq!(result.summary(), "no provenance committed");
    }

    #[test]
    fn spine_commit_params_format() {
        let params = spine_commit_params("entry-123", "abcdef");
        assert_eq!(params["entry_id"], "entry-123");
        assert_eq!(params["data_hash"], "abcdef");
        assert_eq!(params["entry_type"], "SessionCommit");
    }

    #[test]
    fn session_serialization() {
        let session = ProvenanceSession {
            dag_session_id: Some(String::from("sess-1")),
            spine_entry_id: Some(String::from("entry-2")),
            braid_id: None,
            available: TrioAvailability {
                dag: true,
                spine: true,
                attribution: false,
            },
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("\"dag_session_id\":\"sess-1\""));
        assert!(json.contains("\"braid_id\":null"));
    }

    #[tokio::test]
    async fn session_create_with_no_primals() {
        let session = ProvenanceSession::create("test-session", None, None, None).await;
        assert!(session.dag_session_id.is_none());
        assert!(session.spine_entry_id.is_none());
        assert!(session.braid_id.is_none());
        assert!(!session.available.full());
    }
}
