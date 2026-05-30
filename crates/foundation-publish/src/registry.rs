// SPDX-License-Identifier: AGPL-3.0-or-later
//! pseudoSpore registry reader — parses lithoSpore's `pseudospores/registry.toml`.
//!
//! This module reads the upstream registry without owning it. lithoSpore is
//! the source of truth; foundation only consumes for publishing.

use std::path::Path;

use serde::Deserialize;

use foundation_core::CoreError;

/// Metadata section of the pseudoSpore registry.
#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMeta {
    /// When the registry was last updated.
    pub last_updated: String,
    /// Total ingested pseudoSpores.
    pub total_ingested: u32,
}

/// A single pseudoSpore entry in the registry.
#[derive(Debug, Clone, Deserialize)]
pub struct PseudoSporeEntry {
    /// Display name (e.g. `"hotSpring-CompChem-GuideStone"`).
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Origin repository path.
    pub origin: String,
    /// Spring that produced this spore.
    pub spring: String,
    /// Ingestion status.
    pub status: String,
    /// Number of passing modules.
    #[serde(default)]
    pub modules_pass: u32,
    /// Total modules in the spore.
    #[serde(default)]
    pub modules_total: u32,
    /// Optional domain profile name.
    #[serde(default)]
    pub domain_profile: Option<String>,
    /// Optional BLAKE3 hash of the artifact.
    #[serde(default)]
    pub blake3: Option<String>,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

impl PseudoSporeEntry {
    /// Generate a URL-safe slug from the name.
    #[must_use]
    pub fn slug(&self) -> String {
        self.name.to_lowercase().replace(' ', "-")
    }

    /// Whether all modules pass validation.
    #[must_use]
    pub const fn fully_validated(&self) -> bool {
        self.modules_pass == self.modules_total && self.modules_total > 0
    }

    /// Pass rate as a human-readable string (e.g. `"7/8"`).
    #[must_use]
    pub fn pass_rate(&self) -> String {
        format!("{}/{}", self.modules_pass, self.modules_total)
    }
}

/// The full pseudoSpore registry.
#[derive(Debug, Clone, Deserialize)]
pub struct SporeRegistry {
    /// Registry metadata.
    pub meta: RegistryMeta,
    /// Registered pseudoSpores.
    #[serde(default, rename = "pseudospore")]
    pub entries: Vec<PseudoSporeEntry>,
}

impl SporeRegistry {
    /// Load a registry from a TOML file path.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] on read failure or [`CoreError::TomlParse`] on parse error.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| CoreError::io(path, e))?;
        toml::from_str(&content).map_err(|e| CoreError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Find a pseudoSpore by name (case-insensitive).
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&PseudoSporeEntry> {
        self.entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Get all entries with status "COMPLETE".
    #[must_use]
    pub fn complete_entries(&self) -> Vec<&PseudoSporeEntry> {
        self.entries
            .iter()
            .filter(|e| e.status.eq_ignore_ascii_case("COMPLETE"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_REGISTRY: &str = r#"
[meta]
last_updated = "2026-05-24"
total_ingested = 2

[[pseudospore]]
name = "hotSpring-CompChem-GuideStone"
version = "1.6.1"
origin = "ecoPrimals/springs/hotSpring"
spring = "springs/hotSpring"
status = "COMPLETE"
modules_pass = 7
modules_total = 8
description = "Computational chemistry validation across Yukawa MD, lattice QCD, gradient flow"

[[pseudospore]]
name = "healthSpring-BTSP-Probe"
version = "0.1.0"
origin = "ecoPrimals/springs/healthSpring"
spring = "springs/healthSpring"
status = "PARTIAL"
modules_pass = 3
modules_total = 5
"#;

    #[test]
    fn parse_registry() {
        let registry: SporeRegistry = toml::from_str(SAMPLE_REGISTRY).unwrap();
        assert_eq!(registry.meta.total_ingested, 2);
        assert_eq!(registry.entries.len(), 2);
    }

    #[test]
    fn find_by_name() {
        let registry: SporeRegistry = toml::from_str(SAMPLE_REGISTRY).unwrap();
        let entry = registry
            .find_by_name("hotspring-compchem-guidestone")
            .unwrap();
        assert_eq!(entry.version, "1.6.1");
        assert!(registry.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn complete_entries() {
        let registry: SporeRegistry = toml::from_str(SAMPLE_REGISTRY).unwrap();
        let complete = registry.complete_entries();
        assert_eq!(complete.len(), 1);
        assert_eq!(complete[0].name, "hotSpring-CompChem-GuideStone");
    }

    #[test]
    fn slug_generation() {
        let registry: SporeRegistry = toml::from_str(SAMPLE_REGISTRY).unwrap();
        assert_eq!(registry.entries[0].slug(), "hotspring-compchem-guidestone");
    }

    #[test]
    fn pass_rate_display() {
        let registry: SporeRegistry = toml::from_str(SAMPLE_REGISTRY).unwrap();
        assert_eq!(registry.entries[0].pass_rate(), "7/8");
        assert!(!registry.entries[0].fully_validated());
    }

    #[test]
    fn fully_validated() {
        let mut entry = registry_entry();
        entry.modules_pass = 8;
        entry.modules_total = 8;
        assert!(entry.fully_validated());
    }

    fn registry_entry() -> PseudoSporeEntry {
        PseudoSporeEntry {
            name: String::from("test-spore"),
            version: String::from("1.0.0"),
            origin: String::from("test/origin"),
            spring: String::from("testSpring"),
            status: String::from("COMPLETE"),
            modules_pass: 5,
            modules_total: 5,
            domain_profile: None,
            blake3: None,
            description: None,
        }
    }
}
