// SPDX-License-Identifier: AGPL-3.0-or-later
//! Project-relative path conventions.
//!
//! Centralizes the well-known directory and file paths relative to the project
//! root. These conventions are shared across the CLI, validation pipeline, and
//! fetch infrastructure — changing them here updates all consumers.

use std::path::{Path, PathBuf};

/// Well-known relative paths within a projectFOUNDATION workspace.
pub mod conventions {
    /// Thread index manifest.
    pub const THREAD_INDEX: &str = "lineage/THREAD_INDEX.toml";
    /// Spring version manifest for drift detection.
    pub const SPRING_VERSIONS: &str = "lineage/SPRING_VERSIONS.toml";
    /// Discovery defaults bootstrap configuration.
    pub const DISCOVERY_DEFAULTS: &str = "deploy/discovery_defaults.toml";
    /// Default data directory (for fetched sources).
    pub const DATA_FETCHED: &str = "data/fetched";
    /// Workloads directory (toadStool-executable definitions).
    pub const WORKLOADS: &str = "workloads";
    /// Validation output directory.
    pub const VALIDATION: &str = "validation";
    /// sporePrint gallery output directory.
    pub const SPOREPRINT_SPORES: &str = "sporeprint/spores";
    /// Validation report filename.
    pub const VALIDATION_REPORT: &str = "VALIDATION_REPORT.md";
    /// Provenance manifest filename.
    pub const PROVENANCE_TOML: &str = "provenance.toml";
}

/// Resolve a project-relative path against a root directory.
#[must_use]
pub fn resolve(root: &Path, relative: &str) -> PathBuf {
    root.join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_joins_correctly() {
        let root = Path::new("/home/user/project");
        assert_eq!(
            resolve(root, conventions::THREAD_INDEX),
            PathBuf::from("/home/user/project/lineage/THREAD_INDEX.toml")
        );
    }

    #[test]
    fn conventions_are_relative() {
        for path in [
            conventions::THREAD_INDEX,
            conventions::SPRING_VERSIONS,
            conventions::DISCOVERY_DEFAULTS,
            conventions::DATA_FETCHED,
            conventions::WORKLOADS,
            conventions::VALIDATION,
            conventions::SPOREPRINT_SPORES,
        ] {
            assert!(
                !path.starts_with('/'),
                "convention path '{path}' must be relative"
            );
        }
    }
}
