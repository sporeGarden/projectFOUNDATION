// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests using mock TCP and UDS JSON-RPC servers.
//!
//! Tests are serialized via `ENV_LOCK` because they mutate process-wide
//! environment variables for discovery override testing. The guard is held
//! across await points intentionally to prevent interleaving.
#![allow(
    unsafe_code,
    clippy::await_holding_lock,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::sync::Mutex;
use std::time::Duration;

use foundation_core::config::DiscoveryConfig;
use foundation_ipc::client::PrimalClient;
use foundation_ipc::health::HealthTriad;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Global lock to serialize tests that manipulate environment variables.
/// This prevents `NESTGATE_SOCKET` set in one test from leaking to others.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Spawn a mock TCP JSON-RPC server that serves canned responses.
///
/// Each response is served on a separate accepted connection (matching the
/// transport's connect-per-call pattern). Returns after all responses are served.
async fn spawn_mock_tcp(responses: Vec<&'static str>) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let handle = tokio::spawn(async move {
        for response in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 8192];
            let _ = stream.read(&mut buf).await.unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();
            stream.flush().await.unwrap();
        }
    });

    // Yield to ensure the listener is active before returning.
    tokio::task::yield_now().await;

    (port, handle)
}

#[tokio::test]
async fn health_triad_all_healthy() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let liveness = r#"{"jsonrpc":"2.0","result":{"status":"alive"},"id":1}"#;
    let readiness = r#"{"jsonrpc":"2.0","result":{"status":"ready"},"id":2}"#;
    let check = r#"{"jsonrpc":"2.0","result":{"version":"0.1.0","uptime":3600},"id":3}"#;

    let (port, server) = spawn_mock_tcp(vec![liveness, readiness, check]).await;

    let config_str =
        format!("[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = {port}\n");
    let config: DiscoveryConfig = toml::from_str(&config_str).unwrap();

    let mut client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_secs(5));

    let mut triad = HealthTriad::new();
    let idx = triad.check(&mut client).await;
    let status = &triad.results[idx];

    assert!(status.alive, "liveness check failed");
    assert!(status.ready, "readiness check failed");
    assert_eq!(status.version.as_deref(), Some("0.1.0"));
    assert_eq!(status.level, foundation_ipc::DegradationLevel::Healthy);
    assert!(triad.all_alive());
    assert!(triad.all_ready());

    server.abort();
}

#[tokio::test]
async fn health_triad_unreachable() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let config_str = "[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = 1\n";
    let config: DiscoveryConfig = toml::from_str(config_str).unwrap();

    let mut client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_millis(200));

    let mut triad = HealthTriad::new();
    let idx = triad.check(&mut client).await;
    let status = &triad.results[idx];

    assert!(!status.alive);
    assert!(!status.ready);
    assert_eq!(status.level, foundation_ipc::DegradationLevel::Unreachable);
    assert!(!triad.all_alive());
}

#[tokio::test]
async fn client_call_raw_success() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let response = r#"{"jsonrpc":"2.0","result":{"files":["a.dat","b.dat"]},"id":1}"#;
    let (port, server) = spawn_mock_tcp(vec![response]).await;

    let config_str =
        format!("[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = {port}\n");
    let config: DiscoveryConfig = toml::from_str(&config_str).unwrap();

    let client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_secs(5));

    let result = client
        .call_raw("storage.list", Some(serde_json::json!({"prefix": "/data"})))
        .await
        .unwrap();

    assert_eq!(result["files"].as_array().unwrap().len(), 2);

    server.abort();
}

#[tokio::test]
async fn client_call_raw_rpc_error() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let response =
        r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
    let (port, server) = spawn_mock_tcp(vec![response]).await;

    let config_str =
        format!("[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = {port}\n");
    let config: DiscoveryConfig = toml::from_str(&config_str).unwrap();

    let client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_secs(5));

    let err = client
        .call_raw("nonexistent.method", None)
        .await
        .unwrap_err();

    let err_str = err.to_string();
    assert!(
        err_str.contains("Method not found") || err_str.contains("-32601"),
        "unexpected error: {err_str}"
    );

    server.abort();
}

#[tokio::test]
async fn discovery_fails_for_unknown_primal() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let config_str = "[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = 9500\n";
    let config: DiscoveryConfig = toml::from_str(config_str).unwrap();

    let result = PrimalClient::discover("unknown_primal", &config);
    assert!(result.is_err());
}

#[tokio::test]
async fn uds_discovery_with_env_override() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("nestgate.sock");

    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap();
        let resp = r#"{"jsonrpc":"2.0","result":{"alive":true},"id":1}"#;
        stream.write_all(resp.as_bytes()).await.unwrap();
        stream.write_all(b"\n").await.unwrap();
        stream.flush().await.unwrap();
    });

    // Yield to let the UDS listener bind
    tokio::task::yield_now().await;

    unsafe { std::env::set_var("NESTGATE_SOCKET", sock_path.to_str().unwrap()) };

    let config: DiscoveryConfig = toml::from_str("[sockets]\n").unwrap();
    let client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_secs(5));

    let result = client
        .call_raw("health.liveness", Some(serde_json::json!({})))
        .await
        .unwrap();

    assert_eq!(result["alive"], true);

    unsafe { std::env::remove_var("NESTGATE_SOCKET") };
    server.abort();
}
