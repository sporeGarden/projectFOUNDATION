// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ecosystem health dashboard — JSON-RPC data model.
//!
//! Exposes FOUNDATION's comprehensive validation view (41,500+ checks)
//! as structured data suitable for petalTongue rendering or sporePrint
//! content generation.
//!
//! JSON-RPC method: `foundation.ecosystem_health`

use serde::{Deserialize, Serialize};

use crate::error::DegradationLevel;

/// Response payload for `foundation.ecosystem_health` JSON-RPC method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealth {
    /// Overall ecosystem status.
    pub status: DegradationLevel,
    /// Wave number of the most recent validation.
    pub wave: u32,
    /// Timestamp of last validation run (ISO 8601).
    pub last_validated: String,
    /// Total quantitative checks across all springs and primals.
    pub total_checks: u64,
    /// Per-spring health summaries.
    pub springs: Vec<SpringHealth>,
    /// Per-primal health summaries.
    pub primals: Vec<PrimalHealth>,
    /// Drift detection summary (if run recently).
    pub drift: Option<DriftSummary>,
}

/// Health summary for a single spring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringHealth {
    /// Spring name (e.g. "hotSpring").
    pub name: String,
    /// Current version from manifest.
    pub version: String,
    /// Number of quantitative checks.
    pub checks: u64,
    /// Wave when last verified.
    pub wave_verified: u32,
    /// Whether this spring has known drift.
    pub drifted: bool,
}

/// Health summary for a single primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Primal name (e.g. "biomeOS").
    pub name: String,
    /// Current version from manifest.
    pub version: String,
    /// Number of quantitative checks.
    pub checks: u64,
    /// Wave when last verified.
    pub wave_verified: u32,
}

/// Summary of drift detection results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSummary {
    /// Total entries evaluated.
    pub total_entries: usize,
    /// Entries with confirmed drift.
    pub drifted: usize,
    /// Entries that could not be read.
    pub unreadable: usize,
}

/// The JSON-RPC method name for ecosystem health queries.
pub const METHOD_ECOSYSTEM_HEALTH: &str = "foundation.ecosystem_health";

impl EcosystemHealth {
    /// Build an ecosystem health response from a version manifest.
    #[must_use]
    pub fn from_manifest(manifest: &foundation_core::versions::VersionManifest) -> Self {
        let springs: Vec<SpringHealth> = manifest
            .springs
            .iter()
            .map(|(name, sv)| SpringHealth {
                name: name.clone(),
                version: sv.version.clone(),
                checks: sv.checks,
                wave_verified: sv.wave_verified,
                drifted: false,
            })
            .collect();

        let primals: Vec<PrimalHealth> = manifest
            .primals
            .iter()
            .map(|(name, pv)| PrimalHealth {
                name: name.clone(),
                version: pv.version.clone(),
                checks: pv.checks,
                wave_verified: pv.wave_verified,
            })
            .collect();

        let total_checks = springs.iter().map(|s| s.checks).sum::<u64>()
            + primals.iter().map(|p| p.checks).sum::<u64>();

        Self {
            status: DegradationLevel::Healthy,
            wave: manifest.meta.wave,
            last_validated: manifest.meta.last_synced.clone(),
            total_checks,
            springs,
            primals,
            drift: None,
        }
    }

    /// Enrich with drift detection results.
    #[must_use]
    pub fn with_drift(mut self, report: &foundation_core::versions::DriftReport) -> Self {
        for entry in &report.entries {
            if entry.version_drifted {
                if let Some(spring) = self.springs.iter_mut().find(|s| s.name == entry.name) {
                    spring.drifted = true;
                }
            }
        }

        if report.has_drift() {
            self.status = DegradationLevel::Degraded;
        }

        self.drift = Some(DriftSummary {
            total_entries: report.entries.len(),
            drifted: report.drifted,
            unreadable: report.unreadable,
        });

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation_core::versions::{PrimalVersion, SpringVersion, VersionManifest, VersionMeta};
    use std::collections::HashMap;

    fn sample_manifest() -> VersionManifest {
        let mut springs = HashMap::new();
        springs.insert(
            "hotSpring".to_string(),
            SpringVersion {
                version: "0.6.32".to_string(),
                workspace: "barracuda".to_string(),
                checks: 1234,
                wave_verified: 73,
            },
        );
        let mut primals = HashMap::new();
        primals.insert(
            "biomeOS".to_string(),
            PrimalVersion {
                version: "3.98".to_string(),
                checks: 595,
                wave_verified: 73,
            },
        );
        VersionManifest {
            meta: VersionMeta {
                last_synced: "2026-06-03".to_string(),
                wave: 73,
            },
            springs,
            primals,
        }
    }

    #[test]
    fn builds_from_manifest() {
        let health = EcosystemHealth::from_manifest(&sample_manifest());
        assert_eq!(health.wave, 73);
        assert_eq!(health.total_checks, 1829);
        assert_eq!(health.springs.len(), 1);
        assert_eq!(health.primals.len(), 1);
        assert_eq!(health.status, DegradationLevel::Healthy);
    }

    #[test]
    fn serializes_to_json() {
        let health = EcosystemHealth::from_manifest(&sample_manifest());
        let json = serde_json::to_string_pretty(&health).unwrap();
        assert!(json.contains("\"total_checks\": 1829"));
        assert!(json.contains("hotSpring"));
        assert!(json.contains("biomeOS"));
    }

    #[test]
    fn method_name_follows_convention() {
        assert!(METHOD_ECOSYSTEM_HEALTH.starts_with("foundation."));
    }
}
