// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source fetcher — download external datasets with rate limiting and retry.

use std::path::{Path, PathBuf};
use std::time::Duration;

use foundation_core::CoreError;
use foundation_core::source::{Source, SourcesManifest};
use tracing::{debug, info, warn};

use crate::hasher;

/// Rate limit for NCBI Entrez API (per their terms of service).
const DEFAULT_NCBI_DELAY: Duration = Duration::from_millis(400);
/// Rate limit for `UniProt` REST API.
const DEFAULT_UNIPROT_DELAY: Duration = Duration::from_millis(500);
/// Default per-request HTTP timeout.
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
/// Maximum retries before declaring a source unreachable.
const DEFAULT_MAX_RETRIES: u32 = 3;
/// Minimum bytes to accept as a valid download (protects against error pages).
const DEFAULT_MIN_FILE_SIZE: u64 = 100;
/// Default delay for APIs without specific rate limits.
const DEFAULT_GENERIC_DELAY: Duration = Duration::from_millis(200);

/// Configuration for the fetch operation.
#[derive(Debug, Clone)]
pub struct FetchConfig {
    /// Base directory to store fetched data.
    pub data_dir: PathBuf,
    /// Delay between NCBI API calls (rate-limiting).
    pub ncbi_delay: Duration,
    /// Delay between `UniProt` API calls.
    pub uniprot_delay: Duration,
    /// Maximum number of retries per source.
    pub max_retries: u32,
    /// Timeout for individual HTTP requests.
    pub request_timeout: Duration,
    /// Minimum acceptable file size in bytes (protects against empty/error pages).
    pub min_file_size: u64,
    /// Whether to skip already-fetched files (resume mode).
    pub skip_existing: bool,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data/fetched"),
            ncbi_delay: DEFAULT_NCBI_DELAY,
            uniprot_delay: DEFAULT_UNIPROT_DELAY,
            max_retries: DEFAULT_MAX_RETRIES,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            min_file_size: DEFAULT_MIN_FILE_SIZE,
            skip_existing: true,
        }
    }
}

/// Result of fetching a single source.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Source ID.
    pub source_id: String,
    /// Whether the fetch succeeded.
    pub success: bool,
    /// Output file path (if successful).
    pub output_path: Option<PathBuf>,
    /// Computed BLAKE3 hash (if successful).
    pub blake3: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
    /// Whether this was skipped (already existed).
    pub skipped: bool,
}

/// Manifest-driven data fetcher.
pub struct SourceFetcher {
    config: FetchConfig,
}

impl SourceFetcher {
    /// Create a new fetcher with the given configuration.
    #[must_use]
    pub const fn new(config: FetchConfig) -> Self {
        Self { config }
    }

    /// Fetch all sources from a manifest.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if the data directory cannot be created.
    pub fn fetch_manifest(
        &self,
        manifest: &SourcesManifest,
    ) -> Result<Vec<FetchResult>, CoreError> {
        std::fs::create_dir_all(&self.config.data_dir)
            .map_err(|e| CoreError::io(&self.config.data_dir, e))?;

        let results: Vec<FetchResult> = manifest
            .sources
            .iter()
            .map(|source| self.fetch_source(source))
            .collect();

        let success_count = results.iter().filter(|r| r.success).count();
        let skip_count = results.iter().filter(|r| r.skipped).count();
        info!(
            total = results.len(),
            success = success_count,
            skipped = skip_count,
            "fetch complete"
        );

        Ok(results)
    }

    /// Fetch a single source with retry logic.
    fn fetch_source(&self, source: &Source) -> FetchResult {
        let output_path = self.output_path_for(source);

        if self.config.skip_existing && output_path.exists() {
            if let Ok(meta) = std::fs::metadata(&output_path) {
                if meta.len() >= self.config.min_file_size {
                    debug!(source_id = %source.id, "skipping — already fetched");
                    let blake3 = hasher::blake3_file(&output_path).ok();
                    return FetchResult {
                        source_id: source.id.clone(),
                        success: true,
                        output_path: Some(output_path),
                        blake3,
                        error: None,
                        skipped: true,
                    };
                }
            }
        }

        let Some(url) = &source.url else {
            return FetchResult {
                source_id: source.id.clone(),
                success: false,
                output_path: None,
                blake3: None,
                error: Some(String::from("no URL in source manifest")),
                skipped: false,
            };
        };

        match self.fetch_with_retry(url, &output_path) {
            Ok(blake3) => FetchResult {
                source_id: source.id.clone(),
                success: true,
                output_path: Some(output_path),
                blake3: Some(blake3),
                error: None,
                skipped: false,
            },
            Err(e) => FetchResult {
                source_id: source.id.clone(),
                success: false,
                output_path: None,
                blake3: None,
                error: Some(e),
                skipped: false,
            },
        }
    }

    /// Fetch a URL with configurable retry logic.
    fn fetch_with_retry(&self, url: &str, output: &Path) -> Result<String, String> {
        let delay = self.delay_for_url(url);

        for attempt in 1..=self.config.max_retries {
            match self.do_fetch(url, output) {
                Ok(()) => {
                    let hash =
                        hasher::blake3_file(output).map_err(|e| format!("hash failed: {e}"))?;
                    return Ok(hash);
                }
                Err(e) if attempt < self.config.max_retries => {
                    warn!(
                        attempt,
                        max = self.config.max_retries,
                        error = %e,
                        "fetch failed, retrying"
                    );
                    std::thread::sleep(delay * attempt);
                }
                Err(e) => {
                    return Err(format!(
                        "failed after {} attempts: {e}",
                        self.config.max_retries
                    ));
                }
            }
        }

        unreachable!("loop always returns on max_retries")
    }

    /// Perform a single HTTP GET and write to a file.
    fn do_fetch(&self, url: &str, output: &Path) -> Result<(), String> {
        let agent = ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .timeout_global(Some(self.config.request_timeout))
                .user_agent("projectFOUNDATION/0.1 (scientific-validation)")
                .build(),
        );

        let response = agent
            .get(url)
            .call()
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        let mut reader = response.into_body().into_reader();
        let mut file = std::fs::File::create(output)
            .map_err(|e| format!("cannot create {}: {e}", output.display()))?;

        std::io::copy(&mut reader, &mut file).map_err(|e| format!("write failed: {e}"))?;

        let meta = std::fs::metadata(output).map_err(|e| format!("stat failed: {e}"))?;
        if meta.len() < self.config.min_file_size {
            return Err(format!(
                "file too small ({} bytes, minimum {})",
                meta.len(),
                self.config.min_file_size
            ));
        }

        Ok(())
    }

    /// Determine rate-limit delay based on the URL's domain.
    fn delay_for_url(&self, url: &str) -> Duration {
        if url.contains("ncbi.nlm.nih.gov") {
            self.config.ncbi_delay
        } else if url.contains("uniprot.org") {
            self.config.uniprot_delay
        } else {
            DEFAULT_GENERIC_DELAY
        }
    }

    /// Derive the output file path for a source.
    fn output_path_for(&self, source: &Source) -> PathBuf {
        let filename = format!(
            "{}.{}",
            source.id,
            source.format.as_deref().unwrap_or("dat")
        );
        self.config.data_dir.join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = FetchConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.min_file_size, 100);
        assert!(config.skip_existing);
    }

    #[test]
    fn output_path_generation() {
        let config = FetchConfig {
            data_dir: PathBuf::from("/tmp/test-data"),
            ..FetchConfig::default()
        };
        let fetcher = SourceFetcher::new(config);
        let source = Source {
            id: String::from("nist_spectra"),
            database: String::from("NIST"),
            description: String::from("test"),
            accessions: vec![],
            url: None,
            format: Some(String::from("csv")),
            blake3: String::new(),
            retrieved: String::new(),
            paper: None,
            notes: None,
        };
        let path = fetcher.output_path_for(&source);
        assert_eq!(path, PathBuf::from("/tmp/test-data/nist_spectra.csv"));
    }

    #[test]
    fn delay_for_ncbi() {
        let config = FetchConfig::default();
        let fetcher = SourceFetcher::new(config.clone());
        let delay = fetcher.delay_for_url("https://eutils.ncbi.nlm.nih.gov/entrez");
        assert_eq!(delay, config.ncbi_delay);
    }

    #[test]
    fn delay_for_uniprot() {
        let config = FetchConfig::default();
        let fetcher = SourceFetcher::new(config.clone());
        let delay = fetcher.delay_for_url("https://rest.uniprot.org/uniprotkb/P12345");
        assert_eq!(delay, config.uniprot_delay);
    }

    #[test]
    fn delay_for_other_domains() {
        let config = FetchConfig::default();
        let fetcher = SourceFetcher::new(config);
        let delay = fetcher.delay_for_url("https://example.org/data");
        assert_eq!(delay, Duration::from_millis(200));
    }

    #[test]
    fn output_path_without_format_uses_dat_extension() {
        let dir = tempfile::tempdir().unwrap();
        let config = FetchConfig {
            data_dir: dir.path().to_path_buf(),
            ..FetchConfig::default()
        };
        let fetcher = SourceFetcher::new(config);
        let source = Source {
            id: String::from("literature_ref"),
            database: String::from("Literature"),
            description: String::from("test"),
            accessions: vec![],
            url: None,
            format: None,
            blake3: String::new(),
            retrieved: String::new(),
            paper: None,
            notes: None,
        };
        let path = fetcher.output_path_for(&source);
        assert_eq!(path, dir.path().join("literature_ref.dat"));
    }

    #[test]
    fn fetch_source_without_url_fails() {
        let dir = tempfile::tempdir().unwrap();
        let config = FetchConfig {
            data_dir: dir.path().to_path_buf(),
            skip_existing: false,
            ..FetchConfig::default()
        };
        let fetcher = SourceFetcher::new(config);
        let manifest = SourcesManifest {
            meta: foundation_core::source::SourcesMeta {
                thread: 1,
                thread_name: String::from("WCM"),
                expression: None,
                last_updated: None,
                total_sources: 1,
            },
            sources: vec![Source {
                id: String::from("no_url_source"),
                database: String::from("Literature"),
                description: String::from("reference only"),
                accessions: vec![],
                url: None,
                format: None,
                blake3: String::new(),
                retrieved: String::new(),
                paper: None,
                notes: None,
            }],
        };

        let results = fetcher.fetch_manifest(&manifest).unwrap();
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert!(!result.success);
        assert!(result.output_path.is_none());
        assert_eq!(result.error.as_deref(), Some("no URL in source manifest"));
    }
}
