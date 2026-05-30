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
                stdout: String::from_utf8(output.stdout)
                    .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
                stderr: String::from_utf8(output.stderr)
                    .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
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
///
/// Uses `wait_with_output` in a thread with a timeout mechanism.
/// On timeout, the child process is killed.
fn run_process(
    command: &str,
    args: &[String],
    working_dir: Option<&str>,
    timeout: Duration,
) -> Result<Output, String> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);

    if let Some(dir) = working_dir {
        let expanded = foundation_core::workload::expand_env_placeholder(dir);
        let path = Path::new(expanded.as_ref());
        if path.is_dir() {
            cmd.current_dir(path);
        }
    }

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn {command}: {e}"))?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|e| e.to_string()),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("timed out after {}s", timeout.as_secs()));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("error waiting for {command}: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation_core::workload::{
        ExecType, SkipCondition, Workload, WorkloadExecution, WorkloadMetadata, WorkloadSkip,
    };

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
                exec_type: ExecType::Native,
                command: String::from("/nonexistent/binary"),
                args: vec![],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: Some(WorkloadSkip {
                when: SkipCondition::BinaryMissing,
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
                exec_type: ExecType::Native,
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

    #[test]
    fn timeout_kills_process() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("sleeper"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: ExecType::Native,
                command: String::from("sleep"),
                args: vec![String::from("60")],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: None,
            provenance: None,
        };

        let result = execute_workload(&workload, Some(Duration::from_millis(200)));
        assert!(!result.success);
        assert!(result.stderr.contains("timed out"));
    }

    #[test]
    fn nonexistent_command_fails_gracefully() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("missing-cmd"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: ExecType::Native,
                command: String::from("/nonexistent/binary/that/will/never/exist"),
                args: vec![],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: None,
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(!result.success);
        assert!(!result.skipped);
        assert!(result.stderr.contains("failed to spawn"));
    }

    #[test]
    fn captures_exit_code() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("exit-42"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: ExecType::Native,
                command: String::from("bash"),
                args: vec![String::from("-c"), String::from("exit 42")],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: None,
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(42));
    }

    #[test]
    fn captures_stderr() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("stderr-test"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: ExecType::Native,
                command: String::from("bash"),
                args: vec![
                    String::from("-c"),
                    String::from("echo error_msg >&2; exit 1"),
                ],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: None,
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(!result.success);
        assert!(result.stderr.contains("error_msg"));
    }

    #[test]
    fn working_dir_is_honored() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("pwd-test"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: ExecType::Native,
                command: String::from("pwd"),
                args: vec![],
                working_dir: Some(String::from("/tmp")),
            },
            resources: None,
            security: None,
            skip: None,
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(result.success);
        assert!(result.stdout.trim().starts_with("/tmp"));
    }

    #[test]
    fn skip_with_default_reason() {
        let workload = Workload {
            metadata: WorkloadMetadata {
                name: String::from("default-skip"),
                description: None,
                version: None,
                thread: String::from("01"),
                thread_name: None,
                spring: None,
            },
            execution: WorkloadExecution {
                exec_type: ExecType::Native,
                command: String::from("/nonexistent"),
                args: vec![],
                working_dir: None,
            },
            resources: None,
            security: None,
            skip: Some(WorkloadSkip {
                when: SkipCondition::BinaryMissing,
                binary: Some(String::from("/nonexistent")),
                reason: None,
            }),
            provenance: None,
        };

        let result = execute_workload(&workload, None);
        assert!(result.skipped);
        assert_eq!(result.skip_reason.as_deref(), Some("skip condition met"));
    }
}
