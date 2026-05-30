// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase 8: Validation report generation.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use foundation_core::CoreError;

/// Format current UTC time as `YYYYMMDD-HHMMSS`.
fn timestamp_compact() -> String {
    format_system_time(SystemTime::now(), "%Y%m%d-%H%M%S")
}

/// Format current UTC time as `YYYY-MM-DD HH:MM:SS UTC`.
fn timestamp_display() -> String {
    format_system_time(SystemTime::now(), "%Y-%m-%d %H:%M:%S UTC")
}

/// Format current UTC time as ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`).
fn timestamp_iso() -> String {
    format_system_time(SystemTime::now(), "%Y-%m-%dT%H:%M:%SZ")
}

/// Minimal UTC time formatter using UNIX epoch math — no external crate.
fn format_system_time(time: SystemTime, fmt: &str) -> String {
    let secs = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days = secs / 86_400;
    let time_of_day = secs % 86_400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Civil date from days since epoch (algorithm from Howard Hinnant)
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

    fmt.replace("%Y", &format!("{y:04}"))
        .replace("%m", &format!("{m:02}"))
        .replace("%d", &format!("{d:02}"))
        .replace("%H", &format!("{hours:02}"))
        .replace("%M", &format!("{minutes:02}"))
        .replace("%S", &format!("{seconds:02}"))
}

use crate::compare::ComparisonReport;
use crate::executor::ExecutionResult;
use crate::pipeline::ValidationResult;

/// Typed fetch availability result — replaces raw `(String, bool)` tuples.
#[derive(Debug, Clone)]
pub struct FetchStatus {
    /// Source identifier from the sources manifest.
    pub source_id: String,
    /// Whether the source data is locally available.
    pub available: bool,
}

/// Provenance session IDs for a completed validation run.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceIds<'a> {
    /// DAG session ID from `rhizoCrypt`.
    pub dag_session_id: Option<&'a str>,
    /// Spine entry ID from `loamSpine`.
    pub spine_entry_id: Option<&'a str>,
    /// Braid ID from `sweetGrass`.
    pub braid_id: Option<&'a str>,
    /// Merkle root hash.
    pub merkle_root: Option<&'a str>,
}

/// Generates structured Markdown validation reports.
pub struct ReportWriter {
    gate_name: String,
}

impl ReportWriter {
    /// Create a report writer for the given gate.
    #[must_use]
    pub fn new(gate_name: &str) -> Self {
        Self {
            gate_name: gate_name.to_owned(),
        }
    }

    /// Write a validation report for a completed pipeline run.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] if the report cannot be written.
    pub fn write(&self, result: &ValidationResult) -> Result<PathBuf, CoreError> {
        let timestamp = timestamp_compact();
        let run_dir = PathBuf::from(format!("run-{timestamp}"));
        std::fs::create_dir_all(&run_dir).map_err(|e| CoreError::io(&run_dir, e))?;

        let report_path = run_dir.join("VALIDATION_REPORT.md");
        let content = render(result);
        std::fs::write(&report_path, content).map_err(|e| CoreError::io(&report_path, e))?;

        Ok(report_path)
    }

    /// Write a focused report to a specific path.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] on write failure.
    pub fn write_report(
        &self,
        path: &Path,
        comparison: Option<&ComparisonReport>,
        execution_results: &[ExecutionResult],
        fetch_results: &[FetchStatus],
    ) -> Result<(), CoreError> {
        let mut md = String::with_capacity(4096);

        md.push_str("# Validation Report\n\n");
        let _ = writeln!(md, "**Generated:** {}", timestamp_display());
        let _ = writeln!(md, "**Gate:** {}", self.gate_name);
        let _ = writeln!(
            md,
            "**Overall:** {}\n",
            if comparison.is_none_or(ComparisonReport::all_passed) {
                "PASS"
            } else {
                "FAIL"
            }
        );

        if !fetch_results.is_empty() {
            md.push_str("## Phase 3: Fetch\n\n");
            let fetched = fetch_results.iter().filter(|f| f.available).count();
            let _ = writeln!(md, "{fetched}/{} sources available\n", fetch_results.len());
        }

        md.push_str("## Phase 5: Workload Execution\n\n");
        render_executions(&mut md, execution_results);

        if let Some(comparison) = comparison {
            md.push_str("## Phase 6: Target Comparison\n\n");
            render_comparison(&mut md, comparison);
        }

        std::fs::write(path, md).map_err(|e| CoreError::io(path, e))
    }
}

fn render(result: &ValidationResult) -> String {
    let mut md = String::with_capacity(4096);

    md.push_str("# Validation Report\n\n");
    let _ = writeln!(md, "**Generated:** {}", timestamp_display());
    let _ = writeln!(md, "**Duration:** {:.2}s", result.elapsed_secs);
    let _ = writeln!(
        md,
        "**Overall:** {}\n",
        if result.overall_pass { "PASS" } else { "FAIL" }
    );

    md.push_str("## Phase 1: Health\n\n");
    md.push_str(&result.health_summary);
    md.push_str("\n\n");

    md.push_str("## Phase 5: Workload Execution\n\n");
    render_executions(&mut md, &result.execution_results);

    if let Some(comparison) = &result.comparison {
        md.push_str("## Phase 6: Target Comparison\n\n");
        render_comparison(&mut md, comparison);
    }

    md.push_str("## Phase 7: Provenance\n\n");
    md.push_str(&result.provenance_summary);
    md.push('\n');

    md
}

fn render_executions(md: &mut String, results: &[ExecutionResult]) {
    if results.is_empty() {
        md.push_str("No workloads executed.\n\n");
        return;
    }

    md.push_str("| Workload | Status | Time |\n");
    md.push_str("|----------|--------|------|\n");
    for r in results {
        let status = if r.skipped {
            "SKIP"
        } else if r.success {
            "PASS"
        } else {
            "FAIL"
        };
        let _ = writeln!(
            md,
            "| {} | {status} | {:.2}s |",
            r.name,
            r.elapsed.as_secs_f64()
        );
    }
    md.push('\n');
}

fn render_comparison(md: &mut String, report: &ComparisonReport) {
    let _ = writeln!(
        md,
        "**Results:** {}/{} passed ({:.0}%)\n",
        report.passed,
        report.passed + report.failed,
        report.pass_rate() * 100.0
    );

    md.push_str("| Target | Status | Delta |\n");
    md.push_str("|--------|--------|-------|\n");
    for r in &report.results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        let delta = r
            .delta
            .map_or_else(|| String::from("—"), |d| format!("{d:.6}"));
        let _ = writeln!(md, "| {} | {status} | {delta} |", r.target_id);
    }
    md.push('\n');
}

/// Write a minimal provenance TOML alongside the report.
///
/// # Errors
///
/// Returns [`CoreError::Io`] on write failure.
pub fn write_provenance_toml(run_dir: &Path, ids: &ProvenanceIds<'_>) -> Result<(), CoreError> {
    let path = run_dir.join("provenance.toml");

    let mut content = String::from("# SPDX-License-Identifier: AGPL-3.0-or-later\n");
    let _ = writeln!(content, "generated = \"{}\"", timestamp_iso());
    content.push_str("\n[trio]\n");

    if let Some(id) = ids.dag_session_id {
        let _ = writeln!(content, "dag_session_id = \"{id}\"");
    }
    if let Some(id) = ids.spine_entry_id {
        let _ = writeln!(content, "spine_entry_id = \"{id}\"");
    }
    if let Some(id) = ids.braid_id {
        let _ = writeln!(content, "braid_id = \"{id}\"");
    }
    if let Some(root) = ids.merkle_root {
        let _ = writeln!(content, "merkle_root = \"{root}\"");
    }

    std::fs::write(&path, content).map_err(|e| CoreError::io(&path, e))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use foundation_core::target::ComparisonResult;

    use crate::compare::ComparisonReport;
    use crate::executor::ExecutionResult;
    use crate::pipeline::ValidationResult;

    #[test]
    fn write_report_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let writer = ReportWriter::new("test-gate");
        let report_path = dir.path().join("report.md");

        let comparison = Some(ComparisonReport {
            results: vec![],
            passed: 0,
            failed: 0,
            skipped: 0,
        });

        writer
            .write_report(&report_path, comparison.as_ref(), &[], &[])
            .unwrap();
        assert!(report_path.exists());
        let content = std::fs::read_to_string(&report_path).unwrap();
        assert!(content.contains("PASS"));
        assert!(content.contains("test-gate"));
    }

    #[test]
    fn write_provenance_toml_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let ids = super::ProvenanceIds {
            dag_session_id: Some("session-123"),
            spine_entry_id: Some("entry-456"),
            braid_id: Some("braid-789"),
            merkle_root: Some("abcdef0123456789"),
        };
        write_provenance_toml(dir.path(), &ids).unwrap();

        let path = dir.path().join("provenance.toml");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("session-123"));
        assert!(content.contains("braid-789"));
    }

    #[test]
    fn render_includes_execution_and_comparison() {
        let result = ValidationResult {
            health_summary: String::from("primals reachable"),
            fetch_results: vec![],
            artifacts_registered: 0,
            execution_results: vec![
                ExecutionResult {
                    name: String::from("plasma_bench"),
                    success: true,
                    exit_code: Some(0),
                    stdout: String::new(),
                    stderr: String::new(),
                    elapsed: Duration::from_secs_f64(2.25),
                    skipped: false,
                    skip_reason: None,
                },
                ExecutionResult {
                    name: String::from("wcm_validate"),
                    success: false,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: String::from("timeout"),
                    elapsed: Duration::from_secs_f64(0.5),
                    skipped: false,
                    skip_reason: None,
                },
                ExecutionResult {
                    name: String::from("optional_check"),
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    elapsed: Duration::ZERO,
                    skipped: true,
                    skip_reason: Some(String::from("binary not found")),
                },
            ],
            comparison: Some(ComparisonReport {
                results: vec![
                    ComparisonResult {
                        target_id: String::from("energy_drift"),
                        passed: true,
                        observed: Some(1.0e-6),
                        expected: Some(1.0e-5),
                        delta: Some(9.0e-6),
                        explanation: String::from("within tolerance"),
                    },
                    ComparisonResult {
                        target_id: String::from("charge_neutrality"),
                        passed: false,
                        observed: Some(0.02),
                        expected: Some(0.0),
                        delta: Some(0.02),
                        explanation: String::from("exceeds max"),
                    },
                ],
                passed: 1,
                failed: 1,
                skipped: 0,
            }),
            provenance_summary: String::from("committed"),
            report_path: None,
            elapsed_secs: 12.0,
            overall_pass: false,
        };

        let content = render(&result);
        assert!(content.contains("FAIL"));
        assert!(content.contains("## Phase 5: Workload Execution"));
        assert!(content.contains("| plasma_bench | PASS | 2.25s |"));
        assert!(content.contains("| wcm_validate | FAIL | 0.50s |"));
        assert!(content.contains("| optional_check | SKIP | 0.00s |"));
        assert!(content.contains("## Phase 6: Target Comparison"));
        assert!(content.contains("**Results:** 1/2 passed (50%)"));
        assert!(content.contains("| energy_drift | PASS |"));
        assert!(content.contains("| charge_neutrality | FAIL |"));
    }

    #[test]
    fn timestamp_format_correctness() {
        use std::time::{Duration as StdDuration, UNIX_EPOCH};

        // 2026-05-28 12:00:00 UTC = 1779969600 seconds since epoch
        let fixed_time = UNIX_EPOCH + StdDuration::from_secs(1_779_969_600);
        let compact = format_system_time(fixed_time, "%Y%m%d-%H%M%S");
        assert_eq!(compact, "20260528-120000");

        let display = format_system_time(fixed_time, "%Y-%m-%d %H:%M:%S UTC");
        assert_eq!(display, "2026-05-28 12:00:00 UTC");

        let iso = format_system_time(fixed_time, "%Y-%m-%dT%H:%M:%SZ");
        assert_eq!(iso, "2026-05-28T12:00:00Z");
    }

    #[test]
    fn timestamp_functions_produce_valid_output() {
        let compact = timestamp_compact();
        assert_eq!(compact.len(), 15); // YYYYMMDD-HHMMSS

        let display = timestamp_display();
        assert!(display.ends_with(" UTC"));
        assert!(display.len() >= 23);

        let iso = timestamp_iso();
        assert!(iso.ends_with('Z'));
        assert!(iso.contains('T'));
    }
}
