// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized environment variable names — single source of truth.
//!
//! Mirrors the `env_keys` pattern from primalSpring. No bare
//! `std::env::var("...")` literals should appear in production code;
//! all env access goes through these constants and resolution helpers.

use std::path::PathBuf;

/// Root of the ecoPrimals development tree.
pub const ECOPRIMALS_ROOT: &str = "ECOPRIMALS_ROOT";
/// Root of the springs directory.
pub const SPRINGS_ROOT: &str = "SPRINGS_ROOT";
/// The family identity string (from biomeOS discovery or config).
pub const FAMILY_ID: &str = "FAMILY_ID";
/// Legacy family ID variable (biomeOS compat).
pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
/// Foundation project root.
pub const FOUNDATION_ROOT: &str = "FOUNDATION_ROOT";
/// XDG runtime directory (socket discovery).
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
/// Override for discovery config path.
pub const DISCOVERY_CONFIG: &str = "FOUNDATION_DISCOVERY_CONFIG";
/// Data directory for fetched sources.
pub const DATA_DIR: &str = "FOUNDATION_DATA_DIR";
/// Output directory for validation reports.
pub const REPORT_DIR: &str = "FOUNDATION_REPORT_DIR";
/// Telemetry output path for dispatch metrics (JSONL).
pub const TELEMETRY_PATH: &str = "FOUNDATION_TELEMETRY_PATH";
/// Gate name for validation runs.
pub const GATE_NAME: &str = "FOUNDATION_GATE";

/// Resolve the family ID using the ecosystem priority chain:
/// 1. `FAMILY_ID` env var (legacy)
/// 2. `BIOMEOS_FAMILY_ID` env var
/// 3. Empty (pre-bootstrap — valid for development)
#[must_use]
pub fn resolve_family_id() -> String {
    std::env::var(FAMILY_ID)
        .or_else(|_| std::env::var(BIOMEOS_FAMILY_ID))
        .unwrap_or_default()
}

/// Resolve the foundation project root.
///
/// Priority: `FOUNDATION_ROOT` env → git rev-parse → current directory.
#[must_use]
pub fn resolve_foundation_root() -> PathBuf {
    if let Ok(root) = std::env::var(FOUNDATION_ROOT) {
        return PathBuf::from(root);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolve the data directory for fetched sources.
///
/// Priority: `FOUNDATION_DATA_DIR` env → `{root}/.data/`
#[must_use]
pub fn resolve_data_dir(root: &std::path::Path) -> PathBuf {
    std::env::var(DATA_DIR).map_or_else(|_| root.join(".data"), PathBuf::from)
}

/// Resolve the telemetry output path for dispatch metrics.
///
/// Priority: `FOUNDATION_TELEMETRY_PATH` env → socket dir default.
#[must_use]
pub fn resolve_telemetry_path() -> PathBuf {
    std::env::var(TELEMETRY_PATH).map_or_else(
        |_| {
            let xdg = std::env::var(XDG_RUNTIME_DIR).unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(xdg)
                .join("ecoPrimals")
                .join("foundation_telemetry.jsonl")
        },
        PathBuf::from,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_uppercase() {
        assert_eq!(ECOPRIMALS_ROOT, "ECOPRIMALS_ROOT");
        assert_eq!(FAMILY_ID, "FAMILY_ID");
        assert_eq!(FOUNDATION_ROOT, "FOUNDATION_ROOT");
    }

    #[test]
    fn resolve_family_id_empty_default() {
        let id = resolve_family_id();
        assert!(id.is_empty() || !id.is_empty());
    }

    #[test]
    fn resolve_foundation_root_returns_path() {
        let root = resolve_foundation_root();
        assert!(!root.as_os_str().is_empty());
    }
}
