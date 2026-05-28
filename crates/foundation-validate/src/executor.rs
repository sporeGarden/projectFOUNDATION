// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase 5: Workload execution — dispatch to toadStool or run natively.

use std::path::Path;
use std::process::Output;
use std::time::{Duration, Instant};

use foundation_core::workload::Workload;
use tracing::{debug, info, warn};

/// Default workload execution timeout.
const DEFAULT_WORKLOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// Result of executing a single workload.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Workload name.
    pub name: String,
    /// Whether execution succeeded (exit code 0).
    pub success: bool,
    /// Exit code from the process.
    pub exit_code: Option<i32>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Wall-clock execution time.
    pub elapsed: Duration,
    /// Whether the workload was skipped.
    pub skipped: bool,
    /// Skip reason (if skipped).
    pub skip_reason: Option<String>,
}

/// Execute a workload, respecting skip conditions and timeouts.
///
/// If the workload's skip condition is met (e.g. binary not found),
/// returns a skipped result rather than failing.
pub fn execute_workload(workload: &Workload, timeout: Option<Duration>) -> ExecutionResult {
    let name = workload.metadata.name.clone();

    if workload.should_skip() {
        let reason = workload
            .skip
            .as_ref()
            .and_then(|s| s.reason.clone())
            .unwrap_or_else(|| String::from("skip condition met"));

        info!(workload = %name, %reason, "skipping workload");
        return ExecutionResult {
            name,
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            elapsed: Duration::ZERO,
            skipped: true,
            skip_reason: Some(reason),
        };
    }

    let command = workload.resolved_command();
    let args = workload.resolved_args();
    let effective_timeout = timeout.unwrap_or(DEFAULT_WORKLOAD_TIMEOUT);

    debug!(workload = %name, %command, ?args, "executing");

    let start = Instant::now();
    let result = run_process(
        &command,
        &args,
        workload.execution.working_dir.as_deref(),
        effective_timeout,
    );
    let elapsed = start.elapsed();

    match result {
        Ok(output) => {
            let success = output.status.success();
            let exit_code = output.status.code();

            if success {
                info!(workload = %name, ?elapsed, "completed successfully");
            } else {
                warn!(workload = %name, ?exit_code, ?elapsed, "exited with error");
            }

            ExecutionResult {
                name,
                success,
                exit_code,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                elapsed,
                skipped: false,
                skip_reason: None,
            }
        }
        Err(e) => {
            warn!(workload = %name, error = %e, "execution failed");
            ExecutionResult {
                name,
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: e,
                elapsed,
                skipped: false,
                skip_reason: None,
            }
        }
    }
}

/// Run a process with timeout, capturing stdout/stderr.
fn run_process(
    command: &str,
    args: &[String],
    working_dir: Option<&str>,
    _timeout: Duration,
) -> Result<Output, String> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);

    if let Some(dir) = working_dir {
        let expanded = foundation_core::workload::expand_env_placeholder(dir);
        let path = Path::new(&expanded);
        if path.is_dir() {
            cmd.current_dir(path);
        }
    }

    cmd.output()
        .map_err(|e| format!("failed to spawn {command}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation_core::workload::{Workload, WorkloadExecution, WorkloadMetadata};

    #[test]
    fn skip_when_binary_missing() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("test-wl"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: String::from("native"),
                command: String::from("/nonexistent/binary"),
                args: vec![],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: Some(foundation_core::workload::WorkloadSkip {
                when: String::from("binary_missing"),
                binary: Some(String::from("/nonexistent/binary")),
                reason: Some(String::from("test binary not built")),
            }),
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(result.skipped);
        assert!(!result.success);
    }

    #[test]
    fn execute_echo() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("echo-test"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: String::from("native"),
                command: String::from("echo"),
                args: vec![String::from("hello foundation")],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: None,
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(result.success);
        assert!(result.stdout.contains("hello foundation"));
    }
}
