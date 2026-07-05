// SPDX-License-Identifier: AGPL-3.0-or-later
//! Data source types — external datasets referenced by validation targets.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A single external data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Unique identifier within the manifest.
    pub id: String,
    /// Database or repository name (e.g. "NCBI SRA", "NIST ASD", "Literature").
    pub database: String,
    /// Human-readable description.
    pub description: String,
    /// Accession numbers or DOIs.
    #[serde(default)]
    pub accessions: Vec<String>,
    /// Canonical URL for retrieval.
    #[serde(default)]
    pub url: Option<String>,
    /// Expected data format.
    #[serde(default)]
    pub format: Option<String>,
    /// BLAKE3 content hash (empty until fetched and verified).
    #[serde(default)]
    pub blake3: String,
    /// ISO date when data was last retrieved.
    #[serde(default)]
    pub retrieved: String,
    /// Associated baseCamp paper ID.
    #[serde(default)]
    pub paper: Option<String>,
    /// Additional notes.
    #[serde(default)]
    pub notes: Option<String>,
}

impl Source {
    /// Whether this source has been content-addressed (BLAKE3 hash populated).
    #[must_use]
    pub fn is_hashed(&self) -> bool {
        !self.blake3.is_empty()
    }

    /// Whether this source has been successfully retrieved.
    #[must_use]
    pub fn is_retrieved(&self) -> bool {
        !self.retrieved.is_empty()
    }
}

/// Metadata header of a sources manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct SourcesMeta {
    /// Thread number.
    pub thread: u32,
    /// Thread name.
    pub thread_name: String,
    /// Path to expression doc.
    #[serde(default)]
    pub expression: Option<String>,
    /// Last update date.
    #[serde(default)]
    pub last_updated: Option<String>,
    /// Declared source count.
    pub total_sources: u32,
}

/// Complete sources manifest file.
#[derive(Debug, Clone, Deserialize)]
pub struct SourcesManifest {
    /// File metadata.
    pub meta: SourcesMeta,
    /// All data sources.
    pub sources: Vec<Source>,
}

impl SourcesManifest {
    /// Load a sources manifest from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] on read failure, [`CoreError::TomlParse`] on parse failure,
    /// or [`CoreError::Validation`] if declared count mismatches actual entries.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| CoreError::io(path, e))?;
        let manifest: Self = toml::from_str(&content).map_err(|e| CoreError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })?;

        let declared = manifest.meta.total_sources as usize;
        let actual = manifest.sources.len();
        if declared != actual {
            return Err(CoreError::Validation {
                manifest: path.to_path_buf(),
                message: format!(
                    "meta.total_sources={declared} but found {actual} [[sources]] entries"
                ),
            });
        }

        Ok(manifest)
    }

    /// Count sources with BLAKE3 hashes.
    #[must_use]
    pub fn hashed_count(&self) -> usize {
        self.sources.iter().filter(|s| s.is_hashed()).count()
    }

    /// Count sources that have been retrieved.
    #[must_use]
    pub fn retrieved_count(&self) -> usize {
        self.sources.iter().filter(|s| s.is_retrieved()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sources_manifest() {
        let toml_str = r#"
[meta]
thread = 2
thread_name = "Plasma Physics"
last_updated = "2026-05-06"
total_sources = 2

[[sources]]
id = "murillo_md_transport"
database = "Literature"
description = "Murillo group MD transport coefficients"
accessions = ["10.1103/PhysRevLett.84.6026"]
url = "https://doi.org/10.1103/PhysRevLett.84.6026"
format = "csv"
blake3 = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
retrieved = "2026-05-01"
paper = "07"

[[sources]]
id = "nist_atomic_spectra"
database = "NIST ASD"
description = "NIST Atomic Spectra Database"
accessions = []
url = "https://physics.nist.gov/PhysRefData/ASD/lines_form.html"
format = "csv"
blake3 = ""
retrieved = ""
paper = "07"
"#;
        let manifest: SourcesManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.sources.len(), 2);
        assert_eq!(manifest.hashed_count(), 1);
        assert_eq!(manifest.retrieved_count(), 1);
        assert!(manifest.sources[0].is_hashed());
        assert!(!manifest.sources[1].is_hashed());
    }

    #[test]
    fn count_mismatch_returns_validation_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(
            &path,
            r#"
[meta]
thread = 1
thread_name = "Test"
total_sources = 5

[[sources]]
id = "only_one"
database = "Test"
description = "test"
"#,
        )
        .unwrap();

        let err = SourcesManifest::from_file(&path).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("total_sources=5"));
        assert!(msg.contains("found 1"));
    }

    #[test]
    fn from_file_missing_path() {
        let result = SourcesManifest::from_file(Path::new("/nonexistent/sources.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn source_default_fields() {
        let toml_str = r#"
[meta]
thread = 1
thread_name = "Test"
total_sources = 1

[[sources]]
id = "minimal"
database = "Test"
description = "bare minimum fields"
"#;
        let manifest: SourcesManifest = toml::from_str(toml_str).unwrap();
        let s = &manifest.sources[0];
        assert!(!s.is_hashed());
        assert!(!s.is_retrieved());
        assert!(s.url.is_none());
        assert!(s.format.is_none());
        assert!(s.paper.is_none());
        assert!(s.accessions.is_empty());
    }
}
