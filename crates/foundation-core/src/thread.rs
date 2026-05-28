// SPDX-License-Identifier: AGPL-3.0-or-later
//! Thread domain model — one of the 10 scientific lineage threads.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A scientific domain thread in the unified lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Numeric identifier (1–10).
    pub id: u32,
    /// Human-readable name.
    pub name: String,
    /// Short slug for file matching (e.g. "wcm", "plasma").
    pub short: String,
    /// Associated baseCamp paper IDs.
    #[serde(default)]
    pub basecamp_papers: Vec<String>,
    /// Springs that validate this thread.
    #[serde(default)]
    pub springs: Vec<String>,
    /// External contacts.
    #[serde(default)]
    pub contacts: Vec<String>,
    /// External lineage descriptions.
    #[serde(default)]
    pub external_lineages: Vec<String>,
    /// Path to the expression document.
    pub expression: String,
    /// ML expression path (thread 5 only).
    #[serde(default)]
    pub ml_expression: Option<String>,
    /// Path to the data sources manifest.
    pub data_sources: String,
    /// ML data sources (thread 5 only).
    #[serde(default)]
    pub ml_data_sources: Option<String>,
    /// Path to the validation targets manifest.
    pub data_targets: String,
    /// ML targets (thread 5 only).
    #[serde(default)]
    pub ml_data_targets: Option<String>,
    /// Whether this is a cross-cutting thread.
    #[serde(default)]
    pub cross_cutting: bool,
    /// Current status.
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    String::from("active")
}

/// Metadata section of the thread index TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadIndexMeta {
    /// Schema version.
    pub version: String,
    /// Date generated.
    pub generated: String,
    /// Total thread count.
    pub total_threads: u32,
    /// Total baseCamp papers.
    #[serde(default)]
    pub total_basecamp_papers: u32,
    /// Total springs.
    #[serde(default)]
    pub total_springs: u32,
    /// Total contacts.
    #[serde(default)]
    pub total_contacts: u32,
}

/// The full thread index file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadIndex {
    /// Metadata about the index.
    pub meta: ThreadIndexMeta,
    /// All domain threads.
    pub threads: Vec<Thread>,
}

impl ThreadIndex {
    /// Load the thread index from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if the file cannot be read, or
    /// [`CoreError::TomlParse`] if parsing fails.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| CoreError::io(path, e))?;
        let index: Self = toml::from_str(&content).map_err(|e| CoreError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(index)
    }

    /// Look up a thread by numeric ID.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&Thread> {
        self.threads.iter().find(|t| t.id == id)
    }

    /// Look up a thread by short slug.
    #[must_use]
    pub fn find_by_short(&self, short: &str) -> Option<&Thread> {
        self.threads
            .iter()
            .find(|t| t.short.eq_ignore_ascii_case(short))
    }

    /// Resolve the data targets path for a thread relative to a base directory.
    #[must_use]
    pub fn targets_path(&self, thread_id: u32, base: &Path) -> Option<PathBuf> {
        self.get(thread_id).map(|t| base.join(&t.data_targets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_INDEX: &str = r#"
[meta]
version = "1.1.0"
generated = "2026-05-27"
total_threads = 2
total_basecamp_papers = 10
total_springs = 3
total_contacts = 4

[[threads]]
id = 1
name = "Whole-Cell Modeling"
short = "wcm"
basecamp_papers = ["01", "03"]
springs = ["hotSpring", "wetSpring"]
contacts = ["ABG community"]
external_lineages = []
expression = "expressions/ABG_WHOLE_CELL_REBUILD.md"
data_sources = "data/sources/thread01_wcm.toml"
data_targets = "data/targets/thread01_wcm_targets.toml"
status = "active"

[[threads]]
id = 2
name = "Plasma Physics"
short = "plasma"
basecamp_papers = ["07"]
springs = ["hotSpring"]
contacts = ["Murillo"]
external_lineages = []
expression = "expressions/PLASMA_QCD_SOVEREIGN_GPU.md"
data_sources = "data/sources/thread02_plasma.toml"
data_targets = "data/targets/thread02_plasma_targets.toml"
status = "active"
"#;

    #[test]
    fn parse_thread_index() {
        let index: ThreadIndex = toml::from_str(SAMPLE_INDEX).unwrap();
        assert_eq!(index.meta.total_threads, 2);
        assert_eq!(index.threads.len(), 2);
        assert_eq!(index.threads[0].short, "wcm");
    }

    #[test]
    fn find_by_short() {
        let index: ThreadIndex = toml::from_str(SAMPLE_INDEX).unwrap();
        let plasma = index.find_by_short("plasma").unwrap();
        assert_eq!(plasma.id, 2);
        assert!(index.find_by_short("nonexistent").is_none());
    }

    #[test]
    fn get_by_id() {
        let index: ThreadIndex = toml::from_str(SAMPLE_INDEX).unwrap();
        assert_eq!(index.get(1).unwrap().name, "Whole-Cell Modeling");
        assert!(index.get(99).is_none());
    }

    #[test]
    fn targets_path_resolves_relative_to_base() {
        let index: ThreadIndex = toml::from_str(SAMPLE_INDEX).unwrap();
        let base = Path::new("/project/root");
        let path = index.targets_path(2, base).unwrap();
        assert_eq!(
            path,
            PathBuf::from("/project/root/data/targets/thread02_plasma_targets.toml")
        );
    }

    #[test]
    fn targets_path_missing_thread_returns_none() {
        let index: ThreadIndex = toml::from_str(SAMPLE_INDEX).unwrap();
        assert!(index.targets_path(99, Path::new("/base")).is_none());
    }
}
