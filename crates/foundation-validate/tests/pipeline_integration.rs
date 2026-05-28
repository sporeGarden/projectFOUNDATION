// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for the validation pipeline with fixture data.

use std::path::PathBuf;

use foundation_validate::pipeline::{PipelineConfig, ValidationPipeline};

fn create_fixture_project(dir: &std::path::Path) {
    let deploy = dir.join("deploy");
    std::fs::create_dir_all(&deploy).unwrap();
    std::fs::write(
        deploy.join("discovery_defaults.toml"),
        r#"
[metadata]
vps_standard = "uds_only"

[sockets]
nestgate = "${XDG_RUNTIME_DIR:-/run/user/1000}/ecoPrimals/nestgate.sock"

[bootstrap_tcp]
host = "127.0.0.1"
nestgate = 9500
"#,
    )
    .unwrap();

    let lineage = dir.join("lineage");
    std::fs::create_dir_all(&lineage).unwrap();
    std::fs::write(
        lineage.join("THREAD_INDEX.toml"),
        r#"
[meta]
version = "1.0"
generated = "2026-05-28"
total_threads = 1

[[threads]]
id = 1
short = "test_thread"
name = "Test Thread"
domain = "testing"
expression = "expressions/TEST.md"
data_sources = "data/sources/test_thread.toml"
data_targets = "data/targets/test_thread_targets.toml"
"#,
    )
    .unwrap();

    let targets_dir = dir.join("data/targets");
    std::fs::create_dir_all(&targets_dir).unwrap();
    std::fs::write(
        targets_dir.join("test_thread_targets.toml"),
        r#"
[meta]
thread = 1
thread_name = "Test Thread"
total_targets = 2

[[targets]]
id = "accuracy_test"
paper = "01"
description = "Accuracy within tolerance"
expected_value = 1.0
tolerance = 0.1
spring = "testSpring"
blake3 = ""
validated = true

[[targets]]
id = "rate_test"
paper = "01"
description = "Rate with percentage tolerance"
expected_value = 0.5
tolerance_pct = 10.0
spring = "testSpring"
blake3 = ""
validated = false
"#,
    )
    .unwrap();

    let workloads_dir = dir.join("workloads/test_thread");
    std::fs::create_dir_all(&workloads_dir).unwrap();
    std::fs::write(
        workloads_dir.join("echo_workload.toml"),
        r#"
[metadata]
name = "echo_test"
description = "Simple echo workload for testing"
thread = "01"

[execution]
type = "native"
command = "echo"
args = ["42.0"]
"#,
    )
    .unwrap();

    let data_dir = dir.join("data/fetched");
    std::fs::create_dir_all(&data_dir).unwrap();

    let sources_dir = dir.join("data/sources");
    std::fs::create_dir_all(&sources_dir).unwrap();
    std::fs::write(
        sources_dir.join("test_thread.toml"),
        r#"
[meta]
thread = 1
thread_name = "Test Thread"
total_sources = 1

[[sources]]
id = "test_data"
database = "Synthetic"
description = "Test fixture data"
blake3 = ""
retrieved = ""
"#,
    )
    .unwrap();

    let output = dir.join("validation");
    std::fs::create_dir_all(output).unwrap();
}

#[tokio::test]
async fn pipeline_runs_with_fixtures() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    create_fixture_project(&root);

    let config = PipelineConfig {
        project_root: root.clone(),
        discovery_config_path: root.join("deploy/discovery_defaults.toml"),
        thread_index_path: root.join("lineage/THREAD_INDEX.toml"),
        thread_filter: None,
        skip_fetch: true,
        data_dir: root.join("data/fetched"),
        output_dir: root.join("validation"),
        gate_name: String::from("test-gate"),
    };

    let pipeline = ValidationPipeline::new(config);
    let result = pipeline.run().await.unwrap();

    assert_eq!(result.execution_results.len(), 1);
    assert!(result.execution_results[0].success);
    assert_eq!(result.execution_results[0].name, "echo_test");
    assert!(result.report_path.is_some());
    assert!(result.report_path.unwrap().exists());
}

#[tokio::test]
async fn pipeline_runs_with_thread_filter() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    create_fixture_project(&root);

    let config = PipelineConfig {
        project_root: root.clone(),
        discovery_config_path: root.join("deploy/discovery_defaults.toml"),
        thread_index_path: root.join("lineage/THREAD_INDEX.toml"),
        thread_filter: Some(String::from("nonexistent_thread")),
        skip_fetch: true,
        data_dir: root.join("data/fetched"),
        output_dir: root.join("validation"),
        gate_name: String::from("test-gate"),
    };

    let pipeline = ValidationPipeline::new(config);
    let result = pipeline.run().await.unwrap();

    assert!(result.execution_results.is_empty());
    assert!(result.overall_pass);
}

#[tokio::test]
async fn pipeline_produces_comparison_report() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    create_fixture_project(&root);

    let config = PipelineConfig {
        project_root: root.clone(),
        discovery_config_path: root.join("deploy/discovery_defaults.toml"),
        thread_index_path: root.join("lineage/THREAD_INDEX.toml"),
        thread_filter: Some(String::from("test_thread")),
        skip_fetch: true,
        data_dir: root.join("data/fetched"),
        output_dir: root.join("validation"),
        gate_name: String::from("test-gate"),
    };

    let pipeline = ValidationPipeline::new(config);
    let result = pipeline.run().await.unwrap();

    assert!(result.comparison.is_some());
    let comparison = result.comparison.unwrap();
    assert_eq!(comparison.results.len(), 2);
}
