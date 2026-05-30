// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain profile indexer — reads and indexes `domain_profile.toml` files from springs.
//!
//! Foundation does not re-implement lithoSpore's full `DomainProfile` parser.
//! Instead it reads the `[profile]` header section for indexing, cataloging,
//! and sporePrint content generation. The full profile semantics remain owned
//! by `pseudospore-core` in lithoSpore.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use foundation_core::CoreError;

/// Lightweight representation of a `domain_profile.toml` header.
///
/// Only captures the fields foundation needs for indexing and gallery
/// generation — not the full derivation/audit/figure configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DomainProfileHeader {
    /// Profile section from the TOML.
    pub profile: ProfileSection,
    /// Whether a translation section exists.
    #[serde(default)]
    pub translation: Option<TranslationPresence>,
    /// Whether a derivation section exists.
    #[serde(default)]
    pub derivation: Option<DerivationPresence>,
    /// Whether a figures section exists.
    #[serde(default)]
    pub figures: Option<FiguresPresence>,
    /// Whether an audit section exists.
    #[serde(default)]
    pub audit: Option<AuditPresence>,
}

/// The `[profile]` section of a domain profile.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileSection {
    /// Profile identifier (e.g. `"md-metadynamics-carbohydrate"`).
    pub id: String,
    /// Semantic version of the profile.
    pub version: String,
    /// Required external tools.
    #[serde(default)]
    pub tools: Vec<String>,
}

/// Presence check for `[translation]` — only captures `enabled`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationPresence {
    /// Whether translation is active.
    #[serde(default)]
    pub enabled: bool,
}

/// Presence check for `[derivation]` — only captures the tool.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivationPresence {
    /// The tool used for derivation (e.g. `"plumed"`).
    #[serde(default)]
    pub tool: Option<String>,
}

/// Presence check for `[figures]` — only captures `enabled`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FiguresPresence {
    /// Whether figure generation is active (defaults to `true`).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Presence check for `[audit]`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditPresence {
    /// Whether config fidelity audits are enabled.
    #[serde(default)]
    pub config_fidelity: bool,
    /// Whether scientific claim validation is enabled.
    #[serde(default)]
    pub scientific_claims: bool,
}

const fn default_true() -> bool {
    true
}

impl DomainProfileHeader {
    /// Load a domain profile header from a TOML file.
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

    /// Whether the profile declares index translation.
    #[must_use]
    pub fn has_translation(&self) -> bool {
        self.translation.as_ref().is_some_and(|t| t.enabled)
    }

    /// Whether the profile declares derivation contracts.
    #[must_use]
    pub const fn has_derivation(&self) -> bool {
        self.derivation.is_some()
    }

    /// Whether the profile declares figure generation.
    #[must_use]
    pub fn has_figures(&self) -> bool {
        self.figures.as_ref().is_none_or(|f| f.enabled)
    }

    /// Whether the profile declares audit checks.
    #[must_use]
    pub const fn has_audit(&self) -> bool {
        self.audit.is_some()
    }
}

/// Index of discovered domain profiles across the ecosystem.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProfileIndex {
    /// Discovered profiles with their source path.
    pub profiles: Vec<IndexedProfile>,
}

/// A single indexed domain profile.
#[derive(Debug, Clone, Serialize)]
pub struct IndexedProfile {
    /// Path to the source `domain_profile.toml`.
    pub path: PathBuf,
    /// Spring or garden that owns this profile.
    pub spring: String,
    /// Profile identifier from the `[profile]` section.
    pub id: String,
    /// Profile version.
    pub version: String,
    /// Required tools.
    pub tools: Vec<String>,
    /// Capability flags for quick filtering.
    pub capabilities: ProfileCapabilities,
}

/// Boolean capability flags derived from the profile sections.
#[derive(Debug, Clone, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "domain capability matrix; each bool is independent"
)]
pub struct ProfileCapabilities {
    /// Whether the profile declares index translation.
    pub translation: bool,
    /// Whether the profile declares derivation contracts.
    pub derivation: bool,
    /// Whether the profile declares figure generation.
    pub figures: bool,
    /// Whether the profile declares audit checks.
    pub audit: bool,
}

impl ProfileIndex {
    /// Scan a directory tree for `domain_profile.toml` files and index them.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if directory traversal fails.
    pub fn scan_directory(root: &Path, spring_name: &str) -> Result<Self, CoreError> {
        let mut profiles = Vec::new();
        Self::scan_recursive(root, spring_name, &mut profiles)?;
        Ok(Self { profiles })
    }

    fn scan_recursive(
        dir: &Path,
        spring_name: &str,
        profiles: &mut Vec<IndexedProfile>,
    ) -> Result<(), CoreError> {
        let entries = std::fs::read_dir(dir).map_err(|e| CoreError::io(dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| CoreError::io(dir, e))?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden dirs and common non-profile locations
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    let _ = Self::scan_recursive(&path, spring_name, profiles);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some("domain_profile.toml") {
                if let Ok(header) = DomainProfileHeader::from_file(&path) {
                    profiles.push(IndexedProfile {
                        path: path.clone(),
                        spring: spring_name.to_string(),
                        id: header.profile.id.clone(),
                        version: header.profile.version.clone(),
                        tools: header.profile.tools.clone(),
                        capabilities: ProfileCapabilities {
                            translation: header.has_translation(),
                            derivation: header.has_derivation(),
                            figures: header.has_figures(),
                            audit: header.has_audit(),
                        },
                    });
                }
            }
        }
        Ok(())
    }

    /// Merge another index into this one (for multi-spring scanning).
    pub fn merge(&mut self, other: Self) {
        self.profiles.extend(other.profiles);
    }

    /// Get all profiles that require a specific tool.
    #[must_use]
    pub fn requiring_tool(&self, tool: &str) -> Vec<&IndexedProfile> {
        self.profiles
            .iter()
            .filter(|p| p.tools.iter().any(|t| t.eq_ignore_ascii_case(tool)))
            .collect()
    }

    /// Get all profiles from a specific spring.
    #[must_use]
    pub fn from_spring(&self, spring: &str) -> Vec<&IndexedProfile> {
        self.profiles
            .iter()
            .filter(|p| p.spring.eq_ignore_ascii_case(spring))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SAMPLE_PROFILE: &str = r#"
[profile]
id = "md-metadynamics-carbohydrate"
version = "1.0"
tools = ["gromacs", "plumed"]

[translation]
enabled = true
domain_frame = "pdb_serial"
computation_frame = "gromacs_line_index"
topology_format = "gro"

[derivation]
tool = "plumed"
find_paths = ["PATH"]

[figures]
enabled = true
generator = "python3"

[audit]
config_fidelity = true
scientific_claims = true
"#;

    #[test]
    fn parse_profile_header() {
        let header: DomainProfileHeader = toml::from_str(SAMPLE_PROFILE).unwrap();
        assert_eq!(header.profile.id, "md-metadynamics-carbohydrate");
        assert_eq!(header.profile.version, "1.0");
        assert_eq!(header.profile.tools, vec!["gromacs", "plumed"]);
    }

    #[test]
    fn capability_flags() {
        let header: DomainProfileHeader = toml::from_str(SAMPLE_PROFILE).unwrap();
        assert!(header.has_translation());
        assert!(header.has_derivation());
        assert!(header.has_figures());
        assert!(header.has_audit());
    }

    #[test]
    fn minimal_profile() {
        let minimal = r#"
[profile]
id = "simple-test"
version = "0.1"
"#;
        let header: DomainProfileHeader = toml::from_str(minimal).unwrap();
        assert_eq!(header.profile.id, "simple-test");
        assert!(!header.has_translation());
        assert!(!header.has_derivation());
        assert!(header.has_figures()); // defaults to true when absent
        assert!(!header.has_audit());
    }

    #[test]
    fn scan_directory_finds_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let spring_dir = dir.path().join("hotSpring");
        let module_dir = spring_dir.join("modules/compchem");
        fs::create_dir_all(&module_dir).unwrap();

        fs::write(module_dir.join("domain_profile.toml"), SAMPLE_PROFILE).unwrap();

        let index = ProfileIndex::scan_directory(dir.path(), "hotSpring").unwrap();
        assert_eq!(index.profiles.len(), 1);
        assert_eq!(index.profiles[0].id, "md-metadynamics-carbohydrate");
        assert_eq!(index.profiles[0].spring, "hotSpring");
        assert!(index.profiles[0].capabilities.translation);
    }

    #[test]
    fn index_query_methods() {
        let dir = tempfile::tempdir().unwrap();
        let mod_dir = dir.path().join("spring1");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("domain_profile.toml"), SAMPLE_PROFILE).unwrap();

        let index = ProfileIndex::scan_directory(dir.path(), "hotSpring").unwrap();

        let gromacs = index.requiring_tool("gromacs");
        assert_eq!(gromacs.len(), 1);

        let from_hot = index.from_spring("hotSpring");
        assert_eq!(from_hot.len(), 1);

        let from_wet = index.from_spring("wetSpring");
        assert!(from_wet.is_empty());
    }

    #[test]
    fn merge_indices() {
        let mut idx1 = ProfileIndex {
            profiles: vec![IndexedProfile {
                path: PathBuf::from("/a/domain_profile.toml"),
                spring: String::from("hotSpring"),
                id: String::from("profile-a"),
                version: String::from("1.0"),
                tools: vec![],
                capabilities: ProfileCapabilities {
                    translation: false,
                    derivation: false,
                    figures: true,
                    audit: false,
                },
            }],
        };
        let idx2 = ProfileIndex {
            profiles: vec![IndexedProfile {
                path: PathBuf::from("/b/domain_profile.toml"),
                spring: String::from("wetSpring"),
                id: String::from("profile-b"),
                version: String::from("0.1"),
                tools: vec![String::from("breseq")],
                capabilities: ProfileCapabilities {
                    translation: false,
                    derivation: true,
                    figures: false,
                    audit: true,
                },
            }],
        };

        idx1.merge(idx2);
        assert_eq!(idx1.profiles.len(), 2);
    }
}
