// SPDX-License-Identifier: AGPL-3.0-or-later
//! Orchestrates the 8-phase validation pipeline.

use std::path::{Path, PathBuf};
use std::time::Instant;

use foundation_core::config::DiscoveryConfig;
use foundation_core::source::SourcesManifest;
use foundation_core::target::TargetsManifest;
use foundation_core::thread::ThreadIndex;
use foundation_core::workload::Workload;
use foundation_fetch::registry::ArtifactRegistry;
use foundation_ipc::PrimalClient;
use foundation_ipc::health::HealthTriad;
use tracing::{info, warn};

use crate::compare::{ComparisonReport, compare_targets};
use crate::executor::{ExecutionResult, execute_workload};
use crate::report::ReportWriter;

/// Configuration for a validation pipeline run.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Project root directory.
    pub project_root: PathBuf,
    /// Path to `discovery_defaults.toml`.
    pub discovery_config_path: PathBuf,
    /// Path to `THREAD_INDEX.toml`.
    pub thread_index_path: PathBuf,
    /// Optional: restrict to a single thread (by short name or ID).
    pub thread_filter: Option<String>,
    /// Whether to skip the fetch phase.
    pub skip_fetch: bool,
    /// Data directory for fetched sources.
    pub data_dir: PathBuf,
    /// Output directory for validation reports.
    pub output_dir: PathBuf,
    /// Gate name for provenance records.
    pub gate_name: String,
}

impl PipelineConfig {
    /// Create a config from a project root with standard paths.
    ///
    /// Uses `env_keys` for resolution rather than bare env var names.
    #[must_use]
    pub fn from_project_root(root: PathBuf) -> Self {
        use foundation_core::env_keys;

        Self {
            discovery_config_path: root.join("deploy/discovery_defaults.toml"),
            thread_index_path: root.join("lineage/THREAD_INDEX.toml"),
            data_dir: env_keys::resolve_data_dir(&root),
            output_dir: root.join("validation"),
            gate_name: std::env::var(env_keys::GATE_NAME)
                .unwrap_or_else(|_| String::from("irongate")),
            thread_filter: None,
            skip_fetch: false,
            project_root: root,
        }
    }
}

/// Outcome of a complete validation pipeline run.
#[derive(Debug)]
pub struct ValidationResult {
    /// Phase 1: Health check results.
    pub health_summary: String,
    /// Phase 3: Fetch results (`source_id`, success).
    pub fetch_results: Vec<(String, bool)>,
    /// Phase 4: Artifact count registered.
    pub artifacts_registered: usize,
    /// Phase 5: Workload execution results.
    pub execution_results: Vec<ExecutionResult>,
    /// Phase 6: Target comparison report.
    pub comparison: Option<ComparisonReport>,
    /// Phase 7: Provenance commit summary.
    pub provenance_summary: String,
    /// Phase 8: Report file path.
    pub report_path: Option<PathBuf>,
    /// Total wall-clock time.
    pub elapsed_secs: f64,
    /// Whether the overall run is considered `PASS`.
    pub overall_pass: bool,
}

/// The 8-phase validation pipeline.
pub struct ValidationPipeline {
    config: PipelineConfig,
}

impl ValidationPipeline {
    /// Create a new pipeline with the given configuration.
    #[must_use]
    pub const fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Execute the full 8-phase pipeline.
    ///
    /// # Errors
    ///
    /// Returns errors only for unrecoverable failures (missing config files,
    /// required primals unreachable). Individual phase degradation is recorded
    /// in the result rather than aborting.
    pub async fn run(&self) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let start = Instant::now();

        let discovery_config = DiscoveryConfig::from_file(&self.config.discovery_config_path)?;
        let thread_index = ThreadIndex::from_file(&self.config.thread_index_path)?;

        info!(
            gate = %self.config.gate_name,
            filter = ?self.config.thread_filter,
            "starting validation pipeline"
        );

        // Phase 1: Health checks (graceful degradation — never aborts pipeline)
        let health_summary = self.run_health_phase(&discovery_config).await;

        // Phase 2: Provenance session (graceful degradation)
        let provenance_summary = self.run_provenance_open_phase(&discovery_config).await;

        // Determine which threads to process
        let threads = resolve_threads(&thread_index, self.config.thread_filter.as_deref());

        // Phase 3: Fetch sources (checks data availability)
        let fetch_results = self.run_fetch_phase(&threads);

        // Phase 4: Register artifacts via BLAKE3 scan
        let artifacts_registered = self.run_registry_phase();

        // Phase 5: Execute workloads
        let execution_results = self.run_execution_phase(&threads);

        // Phase 6: Compare targets
        let comparison = self.run_comparison_phase(&threads, &execution_results);

        // Phase 7: Provenance commit (graceful degradation)
        let provenance_summary = self
            .run_provenance_commit_phase(&discovery_config, &provenance_summary)
            .await;

        // Phase 8: Write report
        let report_path =
            self.write_report(comparison.as_ref(), &execution_results, &fetch_results);

        let elapsed_secs = start.elapsed().as_secs_f64();
        // Only pass if: (a) there were no threads to evaluate, OR (b) comparison ran and all passed.
        // A missing comparison when threads were selected is NOT a pass — it means targets failed to load.
        let overall_pass = if threads.is_empty() {
            true
        } else {
            comparison
                .as_ref()
                .is_some_and(ComparisonReport::all_passed)
        };

        info!(elapsed_secs, overall_pass, "pipeline complete");

        Ok(ValidationResult {
            health_summary,
            fetch_results,
            artifacts_registered,
            execution_results,
            comparison,
            provenance_summary,
            report_path,
            elapsed_secs,
            overall_pass,
        })
    }

    fn run_fetch_phase(&self, threads: &[&foundation_core::thread::Thread]) -> Vec<(String, bool)> {
        if self.config.skip_fetch {
            info!("fetch phase skipped by config");
            return Vec::new();
        }

        let mut results = Vec::new();

        for thread in threads {
            let source_file = self.config.project_root.join(&thread.data_sources);
            if !source_file.exists() {
                warn!(
                    thread = %thread.short,
                    path = %source_file.display(),
                    "sources manifest not found"
                );
                continue;
            }
            match SourcesManifest::from_file(&source_file) {
                Ok(manifest) => {
                    for source in &manifest.sources {
                        let exists = self.config.data_dir.join(&source.id).exists();
                        results.push((source.id.clone(), exists));
                    }
                }
                Err(e) => {
                    warn!(file = %source_file.display(), error = %e, "failed to parse sources");
                }
            }
        }
        results
    }

    fn run_registry_phase(&self) -> usize {
        match ArtifactRegistry::scan(&self.config.data_dir) {
            Ok(registry) => {
                let count = registry.count();
                info!(count, "artifacts registered via BLAKE3 scan");
                count
            }
            Err(e) => {
                warn!(error = %e, "artifact registry scan failed");
                0
            }
        }
    }

    fn run_execution_phase(
        &self,
        threads: &[&foundation_core::thread::Thread],
    ) -> Vec<ExecutionResult> {
        let mut results = Vec::new();
        let workloads_dir = self.config.project_root.join("workloads");

        for thread in threads {
            let thread_dir = workloads_dir.join(format!("thread{:02}_{}", thread.id, thread.short));
            if !thread_dir.is_dir() {
                continue;
            }
            let workload_files = list_toml_files(&thread_dir);
            for wf in workload_files {
                match Workload::from_file(&wf) {
                    Ok(workload) => {
                        let result = execute_workload(&workload, None);
                        results.push(result);
                    }
                    Err(e) => {
                        warn!(file = %wf.display(), error = %e, "failed to parse workload");
                    }
                }
            }
        }
        results
    }

    fn run_comparison_phase(
        &self,
        threads: &[&foundation_core::thread::Thread],
        execution_results: &[ExecutionResult],
    ) -> Option<ComparisonReport> {
        let mut all_observations: Vec<(String, f64)> = Vec::new();
        let mut combined_manifest: Option<TargetsManifest> = None;

        // Gather observations from successful executions keyed by workload name.
        // Workload names should align with target IDs by convention.
        for result in execution_results {
            if result.success && !result.stdout.is_empty() {
                if let Some(obs) = parse_numeric_output(&result.stdout) {
                    all_observations.push((result.name.clone(), obs));
                }
            }
        }

        for thread in threads {
            let targets_file = self.config.project_root.join(&thread.data_targets);
            if !targets_file.exists() {
                warn!(
                    thread = %thread.short,
                    path = %targets_file.display(),
                    "targets manifest not found"
                );
                continue;
            }
            match TargetsManifest::from_file(&targets_file) {
                Ok(manifest) => {
                    if let Some(ref mut combined) = combined_manifest {
                        combined.targets.extend(manifest.targets);
                    } else {
                        combined_manifest = Some(manifest);
                    }
                }
                Err(e) => {
                    warn!(file = %targets_file.display(), error = %e, "failed to parse targets");
                }
            }
        }

        combined_manifest.map(|m| compare_targets(&m, &all_observations))
    }

    fn write_report(
        &self,
        comparison: Option<&ComparisonReport>,
        execution_results: &[ExecutionResult],
        fetch_results: &[(String, bool)],
    ) -> Option<PathBuf> {
        let _ = std::fs::create_dir_all(&self.config.output_dir);
        let report_path = self.config.output_dir.join("validation_report.md");
        let writer = ReportWriter::new(&self.config.gate_name);

        match writer.write_report(&report_path, comparison, execution_results, fetch_results) {
            Ok(()) => {
                info!(path = %report_path.display(), "report written");
                Some(report_path)
            }
            Err(e) => {
                warn!(error = %e, "failed to write report");
                None
            }
        }
    }

    /// Phase 1: Health checks — discover and probe validation primals.
    ///
    /// Gracefully degrades: unreachable primals produce warnings, not errors.
    async fn run_health_phase(&self, config: &DiscoveryConfig) -> String {
        use foundation_core::primal_names;

        let mut triad = HealthTriad::new();
        let mut clients: Vec<PrimalClient> = Vec::new();

        for &primal in primal_names::VALIDATION_PRIMALS {
            match PrimalClient::discover(primal, config) {
                Ok(client) => clients.push(client),
                Err(e) => {
                    warn!(primal, error = %e, "discovery failed — skipping health check");
                }
            }
        }

        for client in &mut clients {
            triad.check(client).await;
        }

        let summary = triad.summary();
        if triad.all_ready() {
            info!(summary = %summary, "all primals healthy");
        } else {
            warn!(
                summary = %summary,
                unreachable = ?triad.unreachable_primals(),
                "operating in degraded mode"
            );
        }
        summary
    }

    /// Phase 2: Open provenance session (pre-run registration).
    ///
    /// Gracefully degrades if provenance primals are unreachable.
    async fn run_provenance_open_phase(&self, config: &DiscoveryConfig) -> String {
        use foundation_core::primal_names;

        let rhizocrypt = PrimalClient::discover(primal_names::slugs::RHIZOCRYPT, config);
        match rhizocrypt {
            Ok(client) => {
                match client
                    .call_raw(
                        "dag.session.create",
                        Some(serde_json::json!({
                            "gate": self.config.gate_name,
                            "purpose": "validation"
                        })),
                    )
                    .await
                {
                    Ok(resp) => {
                        let session_id = resp
                            .get("session_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        info!(session_id, "provenance session opened");
                        format!("session:{session_id}")
                    }
                    Err(e) => {
                        warn!(error = %e, "provenance session open failed — degraded");
                        String::from("degraded: rhizoCrypt RPC failed")
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "rhizoCrypt unreachable — provenance degraded");
                String::from("degraded: rhizoCrypt unreachable")
            }
        }
    }

    /// Phase 7: Commit provenance (post-run finalization).
    ///
    /// Gracefully degrades if the session was never opened.
    async fn run_provenance_commit_phase(
        &self,
        config: &DiscoveryConfig,
        session_info: &str,
    ) -> String {
        use foundation_core::primal_names;

        if !session_info.starts_with("session:") {
            return session_info.to_owned();
        }

        let session_id = &session_info["session:".len()..];

        match PrimalClient::discover(primal_names::slugs::RHIZOCRYPT, config) {
            Ok(client) => {
                match client
                    .call_raw(
                        "dag.session.commit",
                        Some(serde_json::json!({
                            "session_id": session_id,
                            "gate": self.config.gate_name,
                        })),
                    )
                    .await
                {
                    Ok(_) => {
                        info!(session_id, "provenance committed");
                        format!("committed:{session_id}")
                    }
                    Err(e) => {
                        warn!(session_id, error = %e, "provenance commit failed");
                        format!("commit_failed:{session_id}")
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "rhizoCrypt unreachable for commit");
                format!("degraded: commit skipped ({session_id})")
            }
        }
    }

    /// Get the pipeline configuration.
    #[must_use]
    pub const fn config(&self) -> &PipelineConfig {
        &self.config
    }
}

/// Resolve which threads to process, optionally filtered.
///
/// Returns references into the `ThreadIndex` — avoids cloning and preserves
/// access to `data_sources`, `data_targets`, and other fields.
fn resolve_threads<'a>(
    index: &'a ThreadIndex,
    filter: Option<&str>,
) -> Vec<&'a foundation_core::thread::Thread> {
    filter.map_or_else(
        || index.threads.iter().collect(),
        |f| {
            index.find_by_short(f).map_or_else(
                || {
                    warn!(filter = f, "thread filter matched no thread");
                    Vec::new()
                },
                |thread| vec![thread],
            )
        },
    )
}

/// List all .toml files in a directory (non-recursive).
fn list_toml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
        .collect()
}

/// Attempt to parse the last line of stdout as a numeric observation.
fn parse_numeric_output(stdout: &str) -> Option<f64> {
    stdout
        .lines()
        .rev()
        .find_map(|line| line.trim().parse::<f64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_project_root() {
        let config = PipelineConfig::from_project_root(PathBuf::from("/tmp/project"));
        assert_eq!(
            config.discovery_config_path,
            PathBuf::from("/tmp/project/deploy/discovery_defaults.toml")
        );
        assert_eq!(
            config.thread_index_path,
            PathBuf::from("/tmp/project/lineage/THREAD_INDEX.toml")
        );
    }

    #[tokio::test]
    async fn run_fails_when_discovery_config_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let config = PipelineConfig {
            project_root: root.clone(),
            discovery_config_path: root.join("deploy/missing_discovery.toml"),
            thread_index_path: root.join("lineage/THREAD_INDEX.toml"),
            thread_filter: None,
            skip_fetch: true,
            data_dir: root.join("data/fetched"),
            output_dir: root.join("validation"),
            gate_name: String::from("test-gate"),
        };
        let pipeline = ValidationPipeline::new(config);
        let err = pipeline.run().await.unwrap_err();
        assert!(err.to_string().contains("missing_discovery.toml"));
    }

    #[tokio::test]
    async fn run_fails_when_thread_index_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("deploy")).unwrap();
        std::fs::write(
            root.join("deploy/discovery_defaults.toml"),
            "[metadata]\nvps_standard = \"uds_only\"\n",
        )
        .unwrap();

        let config = PipelineConfig {
            project_root: root.clone(),
            discovery_config_path: root.join("deploy/discovery_defaults.toml"),
            thread_index_path: root.join("lineage/missing_index.toml"),
            thread_filter: None,
            skip_fetch: true,
            data_dir: root.join("data/fetched"),
            output_dir: root.join("validation"),
            gate_name: String::from("test-gate"),
        };
        let pipeline = ValidationPipeline::new(config);
        let err = pipeline.run().await.unwrap_err();
        assert!(err.to_string().contains("missing_index.toml"));
    }

    #[test]
    fn parse_numeric_output_last_line() {
        assert_eq!(parse_numeric_output("3.15"), Some(3.15));
        assert_eq!(parse_numeric_output("info: run\n42.0\n"), Some(42.0));
        assert_eq!(parse_numeric_output("text only"), None);
        assert_eq!(parse_numeric_output(""), None);
    }

    #[test]
    fn parse_numeric_output_ignores_non_numeric_trailing() {
        let output = "computation done\n0.125\nDONE";
        assert_eq!(parse_numeric_output(output), Some(0.125));
    }

    #[test]
    fn list_toml_files_nonexistent_dir() {
        let files = list_toml_files(Path::new("/nonexistent/path"));
        assert!(files.is_empty());
    }
}
