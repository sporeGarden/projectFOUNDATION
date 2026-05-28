// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests using mock TCP and UDS JSON-RPC servers.
#![allow(unsafe_code)]

use std::time::Duration;

use foundation_core::config::DiscoveryConfig;
use foundation_ipc::client::PrimalClient;
use foundation_ipc::health::HealthTriad;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Spawn a mock TCP JSON-RPC server that echoes back canned responses.
async fn spawn_mock_server(responses: Vec<&'static str>) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let handle = tokio::spawn(async move {
        for response in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 8192];
            let _ = stream.read(&mut buf).await.unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();
        }
    });

    (port, handle)
}

#[tokio::test]
async fn health_triad_all_healthy() {
    let liveness = r#"{"jsonrpc":"2.0","result":{"status":"alive"},"id":1}"#;
    let readiness = r#"{"jsonrpc":"2.0","result":{"status":"ready"},"id":2}"#;
    let check = r#"{"jsonrpc":"2.0","result":{"version":"0.1.0","uptime":3600},"id":3}"#;

    let (port, server) = spawn_mock_server(vec![liveness, readiness, check]).await;

    let config_str =
        format!("[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = {port}\n");
    let config: DiscoveryConfig = toml::from_str(&config_str).unwrap();

    let mut client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_secs(5));

    let mut triad = HealthTriad::new();
    let status = triad.check(&mut client).await;

    assert!(status.alive);
    assert!(status.ready);
    assert_eq!(status.version.as_deref(), Some("0.1.0"));
    assert_eq!(status.level, "healthy");
    assert!(triad.all_alive());
    assert!(triad.all_ready());

    server.abort();
}

#[tokio::test]
async fn health_triad_unreachable() {
    let config_str = "[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = 1\n";
    let config: DiscoveryConfig = toml::from_str(config_str).unwrap();

    let mut client = PrimalClient::discover("nestgate", &config)
        .unwrap()
        .with_timeout(Duration::from_millis(200));

    let mut triad = HealthTriad::new();
    let status = triad.check(&mut client).await;

    assert!(!status.alive);
    assert!(!status.ready);
    assert_eq!(status.level, "unreachable");
    assert!(!triad.all_alive());
}

#[tokio::test]
async fn client_call_raw_success() {
    let response = r#"{"jsonrpc":"2.0","result":{"files":["a.dat","b.dat"]},"id":1}"#;
    let (port, server) = spawn_mock_server(vec![response]).await;

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
    let response =
        r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
    let (port, server) = spawn_mock_server(vec![response]).await;

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
    assert!(err_str.contains("Method not found") || err_str.contains("-32601"));

    server.abort();
}

#[tokio::test]
async fn discovery_fails_for_unknown_primal() {
    let config_str = "[sockets]\n[bootstrap_tcp]\nhost = \"127.0.0.1\"\nnestgate = 9500\n";
    let config: DiscoveryConfig = toml::from_str(config_str).unwrap();

    let result = PrimalClient::discover("unknown_primal", &config);
    assert!(result.is_err());
}

#[tokio::test]
async fn uds_discovery_with_env_override() {
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
    });

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
