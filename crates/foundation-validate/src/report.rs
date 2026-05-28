// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase 8: Validation report generation.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use chrono::Utc;
use foundation_core::CoreError;

use crate::compare::ComparisonReport;
use crate::executor::ExecutionResult;
use crate::pipeline::ValidationResult;

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
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
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
        fetch_results: &[(String, bool)],
    ) -> Result<(), CoreError> {
        let mut md = String::with_capacity(4096);

        md.push_str("# Validation Report\n\n");
        let _ = writeln!(
            md,
            "**Generated:** {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
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
            let fetched = fetch_results.iter().filter(|(_, ok)| *ok).count();
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
    let _ = writeln!(
        md,
        "**Generated:** {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
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
pub fn write_provenance_toml(
    run_dir: &Path,
    dag_session_id: Option<&str>,
    spine_entry_id: Option<&str>,
    braid_id: Option<&str>,
    merkle_root: Option<&str>,
) -> Result<(), CoreError> {
    let path = run_dir.join("provenance.toml");

    let mut content = String::from("# SPDX-License-Identifier: AGPL-3.0-or-later\n");
    let _ = writeln!(
        content,
        "generated = \"{}\"",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    );
    content.push_str("\n[trio]\n");

    if let Some(id) = dag_session_id {
        let _ = writeln!(content, "dag_session_id = \"{id}\"");
    }
    if let Some(id) = spine_entry_id {
        let _ = writeln!(content, "spine_entry_id = \"{id}\"");
    }
    if let Some(id) = braid_id {
        let _ = writeln!(content, "braid_id = \"{id}\"");
    }
    if let Some(root) = merkle_root {
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
        write_provenance_toml(
            dir.path(),
            Some("session-123"),
            Some("entry-456"),
            Some("braid-789"),
            Some("abcdef0123456789"),
        )
        .unwrap();

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
}
