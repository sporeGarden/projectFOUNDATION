// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gallery page generator — creates Zola-compatible Markdown for sporePrint.
//!
//! Generates `/lab/spores/{slug}/` pages for each pseudoSpore in the registry.
//! Pages include computation receipts, module status, provenance info, and
//! download links for lithoSpore packages.

use std::fmt::Write as _;
use std::path::PathBuf;

use foundation_core::CoreError;
use tracing::info;

use crate::registry::PseudoSporeEntry;

/// Configuration for gallery generation.
#[derive(Debug, Clone)]
pub struct GalleryConfig {
    /// Output directory for generated gallery pages.
    pub output_dir: PathBuf,
    /// Base URL for lithoSpore downloads.
    pub download_base_url: String,
    /// Base URL for Forgejo source links.
    pub forgejo_base_url: String,
}

impl Default for GalleryConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from(foundation_core::paths::conventions::SPOREPRINT_SPORES),
            download_base_url: String::from(crate::urls::DOWNLOAD_BASE),
            forgejo_base_url: String::from(crate::urls::FORGEJO_BASE),
        }
    }
}

/// Generates sporePrint gallery pages from pseudoSpore registry entries.
pub struct GalleryGenerator {
    config: GalleryConfig,
}

impl GalleryGenerator {
    /// Create a new generator with the given configuration.
    #[must_use]
    pub const fn new(config: GalleryConfig) -> Self {
        Self { config }
    }

    /// Generate gallery pages for all entries.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if the output directory cannot be created
    /// or pages cannot be written.
    pub fn generate_all(&self, entries: &[&PseudoSporeEntry]) -> Result<Vec<PathBuf>, CoreError> {
        std::fs::create_dir_all(&self.config.output_dir)
            .map_err(|e| CoreError::io(&self.config.output_dir, e))?;

        let mut paths = Vec::with_capacity(entries.len());
        for entry in entries {
            let path = self.generate_page(entry)?;
            paths.push(path);
        }

        info!(count = paths.len(), "gallery pages generated");
        Ok(paths)
    }

    /// Generate a single gallery page for a pseudoSpore.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if the page cannot be written.
    pub fn generate_page(&self, entry: &PseudoSporeEntry) -> Result<PathBuf, CoreError> {
        let slug = entry.slug();
        let filename = format!("{slug}.md");
        let output_path = self.config.output_dir.join(&filename);
        let content = self.render_page(entry);

        std::fs::write(&output_path, content).map_err(|e| CoreError::io(&output_path, e))?;

        info!(slug = %slug, path = %output_path.display(), "generated gallery page");
        Ok(output_path)
    }

    /// Render the full Markdown content for a gallery page.
    #[must_use]
    pub fn render_page(&self, entry: &PseudoSporeEntry) -> String {
        let slug = entry.slug();
        let mut page = String::with_capacity(2048);

        // Zola front matter
        let _ = writeln!(page, "+++");
        let _ = writeln!(page, "title = \"{} v{}\"", entry.name, entry.version);
        let _ = writeln!(page, "date = {}", current_date());
        let _ = writeln!(page, "template = \"page.html\"");
        let _ = writeln!(page, "[extra]");
        let _ = writeln!(page, "entity = \"pseudospore-{slug}\"");
        let _ = writeln!(page, "tier = \"spore\"");
        let _ = writeln!(page, "spring = \"{}\"", extract_spring_name(&entry.spring));
        let _ = writeln!(page, "+++");
        let _ = writeln!(page);

        // Title and summary
        let _ = writeln!(page, "# {} v{}", entry.name, entry.version);
        let _ = writeln!(page);

        if let Some(desc) = &entry.description {
            let _ = writeln!(page, "{desc}");
            let _ = writeln!(page);
        }

        // Status table
        let _ = writeln!(page, "## Validation Status");
        let _ = writeln!(page);
        let _ = writeln!(page, "| Metric | Value |");
        let _ = writeln!(page, "|--------|------:|");
        let _ = writeln!(page, "| Modules validated | {} |", entry.pass_rate());
        let _ = writeln!(page, "| Status | {} |", entry.status);
        let _ = writeln!(page, "| Origin | `{}` |", entry.origin);
        let _ = writeln!(page, "| Spring | `{}` |", entry.spring);
        if let Some(blake3) = &entry.blake3 {
            let _ = writeln!(
                page,
                "| BLAKE3 | `{}`... |",
                &blake3[..16.min(blake3.len())]
            );
        }
        let _ = writeln!(page);

        // Actions
        let _ = writeln!(page, "## Access");
        let _ = writeln!(page);
        let _ = writeln!(
            page,
            "- [Download lithoSpore package]({}/{slug}/lithoSpore.tar.gz)",
            self.config.download_base_url
        );
        let _ = writeln!(
            page,
            "- [View source on Forgejo]({}/{})",
            self.config.forgejo_base_url, entry.origin
        );
        let _ = writeln!(page);

        // Provenance
        let _ = writeln!(page, "## Provenance");
        let _ = writeln!(page);
        let _ = writeln!(
            page,
            "This pseudoSpore carries a sweetGrass W3C PROV-O braid. Validation"
        );
        let _ = writeln!(
            page,
            "receipts are content-addressed via BLAKE3 and registered in NestGate."
        );
        let _ = writeln!(page);
        let _ = writeln!(
            page,
            "To reproduce locally: `litho fetch --from primals.eco/lab/spores/{slug}`"
        );

        page
    }

    /// Generate a section index page (`_index.md`) for the spores gallery.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if the file cannot be written.
    pub fn generate_index(&self, entries: &[&PseudoSporeEntry]) -> Result<PathBuf, CoreError> {
        let index_path = self.config.output_dir.join("_index.md");
        let mut page = String::with_capacity(1024);

        let _ = writeln!(page, "+++");
        let _ = writeln!(page, "title = \"pseudoSpore Gallery\"");
        let _ = writeln!(page, "template = \"section.html\"");
        let _ = writeln!(page, "sort_by = \"date\"");
        let _ = writeln!(page, "+++");
        let _ = writeln!(page);
        let _ = writeln!(page, "# pseudoSpore Gallery");
        let _ = writeln!(page);
        let _ = writeln!(
            page,
            "Hosted computation artifacts from the ecoPrimals spring network."
        );
        let _ = writeln!(
            page,
            "Each pseudoSpore is a self-contained reproduction chassis — validated"
        );
        let _ = writeln!(page, "science you can verify locally via lithoSpore.");
        let _ = writeln!(page);
        let _ = writeln!(page, "| Spore | Version | Modules | Status |");
        let _ = writeln!(page, "|-------|---------|---------|--------|");

        for entry in entries {
            let slug = entry.slug();
            let _ = writeln!(
                page,
                "| [{name}](@/lab/spores/{slug}.md) | {version} | {rate} | {status} |",
                name = entry.name,
                slug = slug,
                version = entry.version,
                rate = entry.pass_rate(),
                status = entry.status,
            );
        }

        std::fs::write(&index_path, page).map_err(|e| CoreError::io(&index_path, e))?;
        info!(path = %index_path.display(), "gallery index generated");
        Ok(index_path)
    }
}

/// Extract the spring short name from a path like `"springs/hotSpring"`.
fn extract_spring_name(spring_path: &str) -> &str {
    spring_path.rsplit('/').next().unwrap_or(spring_path)
}

/// Get current date as `YYYY-MM-DD` for Zola front matter.
fn current_date() -> String {
    use std::time::SystemTime;

    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days = secs / 86_400;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}")
}

/// Generate a gallery page for a pseudoSpore, returning the rendered content
/// without writing to disk. Useful for preview or testing.
#[must_use]
pub fn render_preview(entry: &PseudoSporeEntry, config: &GalleryConfig) -> String {
    let generator = GalleryGenerator::new(config.clone());
    generator.render_page(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::PseudoSporeEntry;

    fn sample_entry() -> PseudoSporeEntry {
        PseudoSporeEntry {
            name: String::from("hotSpring-CompChem-GuideStone"),
            version: String::from("1.6.1"),
            origin: String::from("ecoPrimals/springs/hotSpring"),
            spring: String::from("springs/hotSpring"),
            status: String::from("COMPLETE"),
            modules_pass: 7,
            modules_total: 8,
            domain_profile: None,
            blake3: Some(String::from("abcdef0123456789abcdef0123456789")),
            description: Some(String::from(
                "Computational chemistry validation across Yukawa MD, lattice QCD, gradient flow",
            )),
        }
    }

    #[test]
    fn render_page_contains_front_matter() {
        let entry = sample_entry();
        let config = GalleryConfig::default();
        let content = render_preview(&entry, &config);

        assert!(content.starts_with("+++\n"));
        assert!(content.contains("title = \"hotSpring-CompChem-GuideStone v1.6.1\""));
        assert!(content.contains("entity = \"pseudospore-hotspring-compchem-guidestone\""));
        assert!(content.contains("spring = \"hotSpring\""));
    }

    #[test]
    fn render_page_contains_validation_table() {
        let entry = sample_entry();
        let config = GalleryConfig::default();
        let content = render_preview(&entry, &config);

        assert!(content.contains("| Modules validated | 7/8 |"));
        assert!(content.contains("| Status | COMPLETE |"));
        assert!(content.contains("| BLAKE3 | `abcdef0123456789`... |"));
    }

    #[test]
    fn render_page_contains_links() {
        let entry = sample_entry();
        let config = GalleryConfig::default();
        let content = render_preview(&entry, &config);

        assert!(content.contains("Download lithoSpore package"));
        assert!(content.contains("lithoSpore.tar.gz"));
        assert!(content.contains("View source on Forgejo"));
    }

    #[test]
    fn generate_all_creates_files() {
        let dir = tempfile::tempdir().unwrap();
        let config = GalleryConfig {
            output_dir: dir.path().to_path_buf(),
            ..GalleryConfig::default()
        };
        let generator = GalleryGenerator::new(config);
        let entry = sample_entry();
        let entries: Vec<&PseudoSporeEntry> = vec![&entry];

        let paths = generator.generate_all(&entries).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].exists());

        let content = std::fs::read_to_string(&paths[0]).unwrap();
        assert!(content.contains("hotSpring-CompChem-GuideStone"));
    }

    #[test]
    fn generate_index_creates_table() {
        let dir = tempfile::tempdir().unwrap();
        let config = GalleryConfig {
            output_dir: dir.path().to_path_buf(),
            ..GalleryConfig::default()
        };
        let generator = GalleryGenerator::new(config);
        let entry = sample_entry();
        let entries: Vec<&PseudoSporeEntry> = vec![&entry];

        let path = generator.generate_index(&entries).unwrap();
        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("pseudoSpore Gallery"));
        assert!(content.contains("hotSpring-CompChem-GuideStone"));
        assert!(content.contains("1.6.1"));
    }

    #[test]
    fn extract_spring_name_from_path() {
        assert_eq!(extract_spring_name("springs/hotSpring"), "hotSpring");
        assert_eq!(extract_spring_name("hotSpring"), "hotSpring");
        assert_eq!(
            extract_spring_name("ecoPrimals/springs/wetSpring"),
            "wetSpring"
        );
    }

    #[test]
    fn current_date_format() {
        let date = current_date();
        assert_eq!(date.len(), 10);
        assert!(date.contains('-'));
        assert!(date.starts_with("202"));
    }
}
