// SPDX-License-Identifier: AGPL-3.0-or-later
//! Spring and primal version manifest for drift detection.
//!
//! Parses `lineage/SPRING_VERSIONS.toml` and compares against actual
//! Cargo.toml versions to detect when lineage counts may be stale.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Top-level version manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    /// Sync metadata.
    pub meta: VersionMeta,
    /// Spring versions (keyed by spring name).
    #[serde(default)]
    pub springs: HashMap<String, SpringVersion>,
    /// Primal versions (keyed by primal name).
    #[serde(default)]
    pub primals: HashMap<String, PrimalVersion>,
}

/// Metadata about when the manifest was last synced.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionMeta {
    /// ISO date string of last sync.
    pub last_synced: String,
    /// Wave number at last sync.
    pub wave: u32,
}

/// Version record for a spring.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpringVersion {
    /// Version string (semver or V-tag).
    pub version: String,
    /// Workspace subdirectory (e.g. "barracuda", ".", "ecoPrimal").
    pub workspace: String,
    /// Last-known quantitative check count.
    pub checks: u64,
    /// Wave when this was last verified.
    pub wave_verified: u32,
}

/// Version record for a primal.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrimalVersion {
    /// Version string.
    pub version: String,
    /// Last-known test/check count.
    pub checks: u64,
    /// Wave when this was last verified.
    pub wave_verified: u32,
}

/// Result of comparing a manifest entry against the actual version.
#[derive(Debug, Clone, Serialize)]
pub struct DriftEntry {
    /// Name of the spring or primal.
    pub name: String,
    /// Whether this is a spring or primal.
    pub kind: DriftKind,
    /// Expected version from manifest.
    pub manifest_version: String,
    /// Actual version found on disk (if readable).
    pub actual_version: Option<String>,
    /// Whether versions differ.
    pub version_drifted: bool,
    /// Last-verified check count.
    pub manifest_checks: u64,
    /// Wave when last verified.
    pub wave_verified: u32,
}

/// Classification of the entry.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DriftKind {
    /// A spring (validation substrate).
    Spring,
    /// A primal (infrastructure component).
    Primal,
}

/// Full drift report — machine-readable output.
#[derive(Debug, Clone, Serialize)]
pub struct DriftReport {
    /// Wave of the manifest being checked.
    pub manifest_wave: u32,
    /// Total entries checked.
    pub total_checked: usize,
    /// Entries where version drifted.
    pub drifted: usize,
    /// Entries where version could not be read.
    pub unreadable: usize,
    /// Individual drift entries.
    pub entries: Vec<DriftEntry>,
}

impl VersionManifest {
    /// Load from a TOML file path.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] if the file cannot be read or parsed.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| CoreError::io(path, e))?;
        toml::from_str(&content).map_err(|e| CoreError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })
    }
}

impl DriftReport {
    /// Whether any drift was detected.
    #[must_use]
    pub const fn has_drift(&self) -> bool {
        self.drifted > 0
    }

    /// Summary line for logging.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}/{} checked, {} drifted, {} unreadable",
            self.total_checked,
            self.entries.len(),
            self.drifted,
            self.unreadable
        )
    }
}

/// Read the `version` field from a Cargo.toml at the given path.
///
/// Returns `None` if the file can't be read or doesn't contain a version.
#[must_use]
pub fn read_cargo_version(cargo_toml_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(cargo_toml_path).ok()?;
    let table: toml::Table = toml::from_str(&content).ok()?;

    table
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Check a version manifest against actual spring/primal directories.
///
/// `eco_root` is the path to the `ecoPrimals/` directory containing
/// `springs/` and `primals/` subdirectories.
#[must_use]
pub fn check_drift(manifest: &VersionManifest, eco_root: &Path) -> DriftReport {
    let mut entries = Vec::new();

    for (name, sv) in &manifest.springs {
        let cargo_path = eco_root
            .join("springs")
            .join(name)
            .join(&sv.workspace)
            .join("Cargo.toml");

        let actual_version = read_cargo_version(&cargo_path);
        let version_drifted = actual_version
            .as_ref()
            .is_some_and(|actual| !sv.version.contains(actual) && *actual != sv.version);

        entries.push(DriftEntry {
            name: name.clone(),
            kind: DriftKind::Spring,
            manifest_version: sv.version.clone(),
            actual_version,
            version_drifted,
            manifest_checks: sv.checks,
            wave_verified: sv.wave_verified,
        });
    }

    for (name, pv) in &manifest.primals {
        let cargo_path = eco_root.join("primals").join(name).join("Cargo.toml");

        let actual_version = read_cargo_version(&cargo_path);
        let version_drifted = actual_version
            .as_ref()
            .is_some_and(|actual| !pv.version.contains(actual) && *actual != pv.version);

        entries.push(DriftEntry {
            name: name.clone(),
            kind: DriftKind::Primal,
            manifest_version: pv.version.clone(),
            actual_version,
            version_drifted,
            manifest_checks: pv.checks,
            wave_verified: pv.wave_verified,
        });
    }

    let drifted = entries.iter().filter(|e| e.version_drifted).count();
    let unreadable = entries
        .iter()
        .filter(|e| e.actual_version.is_none())
        .count();
    let total_checked = entries.len() - unreadable;

    DriftReport {
        manifest_wave: manifest.meta.wave,
        total_checked,
        drifted,
        unreadable,
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const SAMPLE_MANIFEST: &str = r#"
[meta]
last_synced = "2026-06-03"
wave = 73

[springs.hotSpring]
version = "0.6.32"
workspace = "barracuda"
checks = 1234
wave_verified = 73

[springs.airSpring]
version = "0.8.7"
workspace = "barracuda"
checks = 902
wave_verified = 73

[primals.biomeOS]
version = "3.98"
checks = 595
wave_verified = 73
"#;

    #[test]
    fn parse_version_manifest() {
        let manifest: VersionManifest = toml::from_str(SAMPLE_MANIFEST).unwrap();
        assert_eq!(manifest.meta.wave, 73);
        assert_eq!(manifest.springs.len(), 2);
        assert_eq!(manifest.primals.len(), 1);
        assert_eq!(manifest.springs["hotSpring"].checks, 1234);
        assert_eq!(manifest.primals["biomeOS"].version, "3.98");
    }

    #[test]
    fn drift_report_with_no_filesystem() {
        let manifest: VersionManifest = toml::from_str(SAMPLE_MANIFEST).unwrap();
        let report = check_drift(&manifest, Path::new("/nonexistent"));
        assert_eq!(report.entries.len(), 3);
        assert_eq!(report.unreadable, 3);
        assert_eq!(report.drifted, 0);
        assert!(!report.has_drift());
    }

    #[test]
    fn drift_report_summary() {
        let report = DriftReport {
            manifest_wave: 73,
            total_checked: 5,
            drifted: 2,
            unreadable: 1,
            entries: Vec::new(),
        };
        assert!(report.has_drift());
        assert!(report.summary().contains("2 drifted"));
    }

    #[test]
    fn read_cargo_version_nonexistent() {
        assert_eq!(read_cargo_version(&PathBuf::from("/no/such/file")), None);
    }
}
