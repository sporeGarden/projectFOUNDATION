// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transport layer — UDS and TCP connection management.
//!
//! Implements newline-delimited JSON-RPC framing over both Unix domain
//! sockets and TCP streams. UDS is preferred; TCP is dev/desktop fallback.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bytes::Bytes;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UnixStream};
use tokio::time::timeout;

use crate::error::IpcError;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

/// Default timeout for RPC calls.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum response size (64 KiB — matches bash `dd bs=65536`).
pub const MAX_RESPONSE_SIZE: usize = 65_536;

/// Error detail for unparseable JSON-RPC responses.
const MALFORMED_JSON_DETAIL: &str = "response is not valid JSON";

/// Abstraction over UDS and TCP transports.
#[derive(Debug, Clone)]
pub enum TransportLayer {
    /// Unix domain socket transport.
    Uds {
        /// Path to the socket file.
        path: PathBuf,
    },
    /// TCP transport (dev/desktop fallback).
    Tcp {
        /// Host address.
        host: String,
        /// Port number.
        port: u16,
    },
}

impl TransportLayer {
    /// Create a UDS transport, verifying the socket file exists.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::SocketNotFound`] if the path does not exist.
    pub fn uds(path: impl Into<PathBuf>) -> Result<Self, IpcError> {
        let path = path.into();
        if !path.exists() {
            return Err(IpcError::SocketNotFound { path });
        }
        Ok(Self::Uds { path })
    }

    /// Create a TCP transport.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Send a JSON-RPC request and receive the response.
    ///
    /// Uses newline-delimited JSON framing per the ecoPrimals IPC standard.
    ///
    /// # Errors
    ///
    /// Returns transport, timeout, or protocol errors.
    pub async fn call(
        &self,
        request: &JsonRpcRequest,
        call_timeout: Option<Duration>,
    ) -> Result<JsonRpcResponse, IpcError> {
        let effective_timeout = call_timeout.unwrap_or(DEFAULT_TIMEOUT);

        let wire = request.to_wire().map_err(|e| IpcError::Serialization {
            method: request.method.clone(),
            source: e,
        })?;

        let response_bytes = timeout(effective_timeout, self.send_recv(&wire))
            .await
            .map_err(|_| IpcError::Timeout {
                primal: String::new(),
                method: request.method.clone(),
                elapsed_ms: u64::try_from(effective_timeout.as_millis()).unwrap_or(u64::MAX),
            })??;

        JsonRpcResponse::from_bytes(&response_bytes).map_err(|_| IpcError::MalformedResponse {
            primal: String::new(),
            method: request.method.clone(),
            detail: String::from(MALFORMED_JSON_DETAIL),
        })
    }

    /// Low-level send/receive over the selected transport.
    async fn send_recv(&self, request_bytes: &[u8]) -> Result<Bytes, IpcError> {
        match self {
            Self::Uds { path } => Self::uds_send_recv(path, request_bytes).await,
            Self::Tcp { host, port } => Self::tcp_send_recv(host, *port, request_bytes).await,
        }
    }

    async fn uds_send_recv(path: &Path, request_bytes: &[u8]) -> Result<Bytes, IpcError> {
        let label = format!("UDS:{}", path.display());
        let io_err = |e: std::io::Error| IpcError::Io {
            transport: label.clone(),
            source: e,
        };

        let stream = UnixStream::connect(path).await.map_err(io_err)?;

        let (reader, mut writer) = stream.into_split();
        writer.write_all(request_bytes).await.map_err(io_err)?;
        writer.shutdown().await.map_err(io_err)?;

        let mut buf_reader = BufReader::new(reader);
        let mut line = Vec::with_capacity(MAX_RESPONSE_SIZE);
        buf_reader
            .read_until(b'\n', &mut line)
            .await
            .map_err(io_err)?;

        Ok(Bytes::from(line))
    }

    async fn tcp_send_recv(host: &str, port: u16, request_bytes: &[u8]) -> Result<Bytes, IpcError> {
        let addr = format!("{host}:{port}");
        let label = format!("TCP:{addr}");
        let io_err = |e: std::io::Error| IpcError::Io {
            transport: label.clone(),
            source: e,
        };

        let stream = TcpStream::connect(&addr).await.map_err(io_err)?;

        let (reader, mut writer) = stream.into_split();
        writer.write_all(request_bytes).await.map_err(io_err)?;
        writer.shutdown().await.map_err(io_err)?;

        let mut buf_reader = BufReader::new(reader);
        let mut line = Vec::with_capacity(MAX_RESPONSE_SIZE);
        buf_reader
            .read_until(b'\n', &mut line)
            .await
            .map_err(io_err)?;

        Ok(Bytes::from(line))
    }
}

impl std::fmt::Display for TransportLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uds { path } => write!(f, "UDS:{}", path.display()),
            Self::Tcp { host, port } => write!(f, "TCP:{host}:{port}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn uds_missing_socket_returns_error() {
        let result = TransportLayer::uds("/nonexistent/path.sock");
        assert!(result.is_err());
    }

    #[test]
    fn tcp_transport_display() {
        let t = TransportLayer::tcp("127.0.0.1", 9400);
        assert_eq!(t.to_string(), "TCP:127.0.0.1:9400");
    }

    #[tokio::test]
    async fn tcp_call_roundtrip() {
        use crate::protocol::JsonRpcRequest;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            assert!(request.contains("\"method\":\"health.liveness\""));
            let response = "{\"jsonrpc\":\"2.0\",\"result\":{\"alive\":true},\"id\":1}\n";
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let transport = TransportLayer::tcp("127.0.0.1", port);
        let request = JsonRpcRequest::new("health.liveness", Some(serde_json::json!({})), 1);
        let response = transport
            .call(&request, Some(Duration::from_secs(5)))
            .await
            .unwrap();

        let result = response.into_result().unwrap();
        assert_eq!(result["alive"], true);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn tcp_timeout_returns_error() {
        use crate::protocol::JsonRpcRequest;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let _server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        let transport = TransportLayer::tcp("127.0.0.1", port);
        let request = JsonRpcRequest::new("slow.op", None, 1);
        let result = transport
            .call(&request, Some(Duration::from_millis(50)))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn uds_call_roundtrip() {
        use crate::protocol::JsonRpcRequest;

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.unwrap();
            assert!(n > 0);
            let response = "{\"jsonrpc\":\"2.0\",\"result\":{\"ready\":true},\"id\":42}\n";
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let transport = TransportLayer::uds(sock_path.to_str().unwrap()).unwrap();
        let request = JsonRpcRequest::new("health.readiness", Some(serde_json::json!({})), 42);
        let response = transport
            .call(&request, Some(Duration::from_secs(5)))
            .await
            .unwrap();

        let result = response.into_result().unwrap();
        assert_eq!(result["ready"], true);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn tcp_connection_refused() {
        use crate::protocol::JsonRpcRequest;

        let transport = TransportLayer::tcp("127.0.0.1", 1);
        let request = JsonRpcRequest::new("noop", None, 1);
        let result = transport.call(&request, Some(Duration::from_secs(2))).await;
        assert!(result.is_err());
    }
}
