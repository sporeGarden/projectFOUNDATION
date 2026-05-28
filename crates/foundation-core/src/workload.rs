// SPDX-License-Identifier: AGPL-3.0-or-later
//! Workload definitions for toadStool dispatch.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A toadStool workload definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    /// Workload metadata.
    pub metadata: WorkloadMetadata,
    /// Execution configuration.
    pub execution: WorkloadExecution,
    /// Resource constraints.
    #[serde(default)]
    pub resources: Option<WorkloadResources>,
    /// Security settings.
    #[serde(default)]
    pub security: Option<WorkloadSecurity>,
    /// Skip conditions.
    #[serde(default)]
    pub skip: Option<WorkloadSkip>,
    /// Provenance chain configuration.
    #[serde(default)]
    pub provenance: Option<WorkloadProvenance>,
}

/// Workload identity and classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadMetadata {
    /// Unique workload name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Semantic version.
    #[serde(default)]
    pub version: Option<String>,
    /// Thread ID (e.g. "02").
    pub thread: String,
    /// Thread name.
    #[serde(default)]
    pub thread_name: Option<String>,
    /// Primary spring.
    #[serde(default)]
    pub spring: Option<String>,
}

/// How the workload is executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadExecution {
    /// Execution type: "native", "wasm", "container".
    #[serde(rename = "type")]
    pub exec_type: String,
    /// Command path (may contain env var placeholders).
    pub command: String,
    /// Command arguments.
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory.
    #[serde(default)]
    pub working_dir: Option<String>,
}

/// Resource constraints for the workload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResources {
    /// Maximum memory in bytes.
    #[serde(default)]
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU percentage.
    #[serde(default)]
    pub max_cpu_percent: Option<f64>,
}

/// Security isolation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadSecurity {
    /// Isolation level: "None", "Standard", "Strict".
    #[serde(default = "default_isolation")]
    pub isolation_level: String,
    /// Directories the workload may access.
    #[serde(default)]
    pub trusted_directories: Vec<String>,
}

fn default_isolation() -> String {
    String::from("Standard")
}

/// Conditions under which the workload should be skipped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadSkip {
    /// Skip condition type (e.g. `binary_missing`).
    pub when: String,
    /// Path to the binary to check.
    #[serde(default)]
    pub binary: Option<String>,
    /// Human-readable skip reason.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Provenance chain configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProvenance {
    /// DAG chain primal.
    #[serde(default)]
    pub chain: Option<String>,
    /// Attribution primal.
    #[serde(default)]
    pub attestation: Option<String>,
    /// Ledger primal.
    #[serde(default)]
    pub spine: Option<String>,
    /// Whether all three provenance primals are required.
    #[serde(default)]
    pub requires_trio: bool,
}

impl Workload {
    /// Load a workload from a TOML file.
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

    /// Whether this workload should be skipped based on its skip condition.
    #[must_use]
    pub fn should_skip(&self) -> bool {
        let Some(skip) = &self.skip else {
            return false;
        };

        match skip.when.as_str() {
            "binary_missing" => skip
                .binary
                .as_ref()
                .is_some_and(|binary| !Path::new(&expand_env_placeholder(binary)).exists()),
            _ => false,
        }
    }

    /// Expand environment variable placeholders in the command path.
    #[must_use]
    pub fn resolved_command(&self) -> String {
        expand_env_placeholder(&self.execution.command)
    }

    /// Expand environment variable placeholders in arguments.
    #[must_use]
    pub fn resolved_args(&self) -> Vec<String> {
        self.execution
            .args
            .iter()
            .map(|a| expand_env_placeholder(a))
            .collect()
    }
}

/// Expand `${VAR}` and `${VAR:-default}` patterns using the process environment.
///
/// Handles nested defaults like `${A:-${B:-fallback}}` via brace-depth counting.
#[must_use]
pub fn expand_env_placeholder(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut var_expr = String::new();
            let mut depth = 1u32;
            for c in chars.by_ref() {
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                var_expr.push(c);
            }

            if let Some((var_name, default)) = var_expr.split_once(":-") {
                match std::env::var(var_name) {
                    Ok(val) if !val.is_empty() => result.push_str(&val),
                    _ => result.push_str(&expand_env_placeholder(default)),
                }
            } else if let Ok(val) = std::env::var(&var_expr) {
                result.push_str(&val);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;

    #[test]
    fn expand_simple_var() {
        // SAFETY: test-only env manipulation, serial test execution
        unsafe { std::env::set_var("TEST_FOUNDATION_VAR", "/opt/springs") };
        let result = expand_env_placeholder("${TEST_FOUNDATION_VAR}/hotSpring");
        assert_eq!(result, "/opt/springs/hotSpring");
        unsafe { std::env::remove_var("TEST_FOUNDATION_VAR") };
    }

    #[test]
    fn expand_with_default() {
        // SAFETY: test-only env manipulation
        unsafe { std::env::remove_var("NONEXISTENT_VAR_XYZ") };
        let result = expand_env_placeholder("${NONEXISTENT_VAR_XYZ:-/fallback}/bin");
        assert_eq!(result, "/fallback/bin");
    }

    #[test]
    fn expand_nested_default() {
        // SAFETY: test-only env manipulation, serial test execution
        unsafe { std::env::set_var("TEST_ECO_ROOT", "/eco") };
        unsafe { std::env::remove_var("TEST_SPRINGS_ROOT_MISSING") };
        let result = expand_env_placeholder(
            "${TEST_SPRINGS_ROOT_MISSING:-${TEST_ECO_ROOT}/springs}/hotSpring",
        );
        assert_eq!(result, "/eco/springs/hotSpring");
        unsafe { std::env::remove_var("TEST_ECO_ROOT") };
    }

    #[test]
    fn parse_workload_toml() {
        let toml_str = r#"
[metadata]
name = "hs-sarkas-md"
description = "Sarkas Yukawa OCP"
version = "0.1.0"
thread = "02"
spring = "hotSpring"

[execution]
type = "native"
command = "/opt/hotspring"
args = ["validate", "--scenario", "sarkas-yukawa-md"]

[resources]
max_memory_bytes = 8589934592
max_cpu_percent = 90.0

[security]
isolation_level = "Standard"
trusted_directories = ["/opt/springs", "/opt/eco"]

[skip]
when = "binary_missing"
binary = "/opt/hotspring"
reason = "binary not built"

[provenance]
chain = "rhizoCrypt"
attestation = "sweetGrass"
spine = "loamSpine"
requires_trio = true
"#;
        let wl: Workload = toml::from_str(toml_str).unwrap();
        assert_eq!(wl.metadata.name, "hs-sarkas-md");
        assert_eq!(wl.execution.exec_type, "native");
        assert!(wl.provenance.as_ref().is_some_and(|p| p.requires_trio));
    }
}
