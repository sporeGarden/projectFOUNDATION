// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized JSON-RPC method name constants.
//!
//! All RPC method names follow the `domain.verb` convention established by
//! primalSpring. Centralizing them here ensures consistency and discoverability.

/// Health check methods (universal across all primals).
pub mod health {
    /// Liveness probe — "is the process alive?"
    pub const LIVENESS: &str = "health.liveness";
    /// Readiness probe — "is the primal ready to serve?"
    pub const READINESS: &str = "health.readiness";
    /// Version/capability check.
    pub const CHECK: &str = "health.check";
}

/// DAG / provenance methods (rhizoCrypt).
pub mod dag {
    /// Create a new DAG session.
    pub const SESSION_CREATE: &str = "dag.session.create";
    /// Commit a DAG session (gate-level commit).
    pub const SESSION_COMMIT: &str = "dag.session.commit";
    /// Complete/finalize a DAG session with merkle root.
    pub const SESSION_COMPLETE: &str = "dag.session.complete";
}

/// Spine / ledger methods (loamSpine).
pub mod entry {
    /// Create a new ledger entry.
    pub const CREATE: &str = "entry.create";
    /// Append data to an existing entry.
    pub const APPEND: &str = "entry.append";
}

/// Braid / attribution methods (sweetGrass).
pub mod braid {
    /// Create an attribution braid linking DAG + spine + attestation.
    pub const CREATE: &str = "braid.create";
}

/// Family/composition identity methods (discovery meta-primal).
pub mod family {
    /// Resolve the family/composition ID.
    pub const ID: &str = "family.id";
}

/// Foundation-specific methods (this primal's own interface).
pub mod foundation {
    /// Query ecosystem health data.
    pub const ECOSYSTEM_HEALTH: &str = "foundation.ecosystem_health";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_follow_domain_verb_convention() {
        for method in [
            health::LIVENESS,
            health::READINESS,
            health::CHECK,
            dag::SESSION_CREATE,
            dag::SESSION_COMMIT,
            dag::SESSION_COMPLETE,
            entry::CREATE,
            entry::APPEND,
            braid::CREATE,
            family::ID,
            foundation::ECOSYSTEM_HEALTH,
        ] {
            assert!(
                method.contains('.'),
                "method '{method}' must follow domain.verb convention"
            );
        }
    }

    #[test]
    fn health_methods_share_prefix() {
        assert!(health::LIVENESS.starts_with("health."));
        assert!(health::READINESS.starts_with("health."));
        assert!(health::CHECK.starts_with("health."));
    }

    #[test]
    fn dag_methods_share_prefix() {
        assert!(dag::SESSION_CREATE.starts_with("dag."));
        assert!(dag::SESSION_COMMIT.starts_with("dag."));
        assert!(dag::SESSION_COMPLETE.starts_with("dag."));
    }

    #[test]
    fn no_duplicate_method_names() {
        let all = [
            health::LIVENESS,
            health::READINESS,
            health::CHECK,
            dag::SESSION_CREATE,
            dag::SESSION_COMMIT,
            dag::SESSION_COMPLETE,
            entry::CREATE,
            entry::APPEND,
            braid::CREATE,
            family::ID,
            foundation::ECOSYSTEM_HEALTH,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b, "duplicate method name: {a}");
            }
        }
    }
}
