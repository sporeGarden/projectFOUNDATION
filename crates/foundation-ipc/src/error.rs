// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC error types — typed failures replacing `|| true` silent swallowing.
//!
//! Follows the primalSpring `PhasedIpcError` pattern: every error carries
//! the phase in which it occurred, enabling structured degradation decisions.

use std::path::PathBuf;

/// Phase of the IPC call lifecycle where the error occurred.
///
/// Mirrors `primalSpring::ipc::IpcErrorPhase` for ecosystem consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcPhase {
    /// During discovery / endpoint resolution.
    Discover,
    /// During transport connection.
    Connect,
    /// During request serialization.
    Serialize,
    /// During wire send.
    Send,
    /// During response receive.
    Receive,
    /// During response parsing / deserialization.
    Parse,
}

impl std::fmt::Display for IpcPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discover => f.write_str("discover"),
            Self::Connect => f.write_str("connect"),
            Self::Serialize => f.write_str("serialize"),
            Self::Send => f.write_str("send"),
            Self::Receive => f.write_str("receive"),
            Self::Parse => f.write_str("parse"),
        }
    }
}

/// Errors from primal IPC operations.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Transport-level connection failure.
    #[error("connection to {primal} failed via {transport}: {message}")]
    Connection {
        /// Target primal name.
        primal: String,
        /// Transport description (UDS path or TCP addr).
        transport: String,
        /// Error detail.
        message: String,
    },

    /// JSON-RPC protocol error (standard error codes -32700 through -32603).
    #[error("RPC error from {primal}.{method}: code={code}, message={message}")]
    RpcError {
        /// Primal that returned the error.
        primal: String,
        /// Method that was called.
        method: String,
        /// JSON-RPC error code.
        code: i64,
        /// Error message from the primal.
        message: String,
    },

    /// Response did not contain expected `result` field.
    #[error("malformed response from {primal}.{method}: {detail}")]
    MalformedResponse {
        /// Primal that sent the response.
        primal: String,
        /// Method that was called.
        method: String,
        /// What was wrong.
        detail: String,
    },

    /// Timeout waiting for response.
    #[error("timeout calling {primal}.{method} after {elapsed_ms}ms")]
    Timeout {
        /// Target primal.
        primal: String,
        /// Method called.
        method: String,
        /// Milliseconds waited.
        elapsed_ms: u64,
    },

    /// Discovery failed — primal not reachable on any transport tier.
    #[error("discovery failed for {primal}: tried {tiers_tried} tiers")]
    DiscoveryFailed {
        /// Primal we tried to reach.
        primal: String,
        /// Number of discovery tiers attempted.
        tiers_tried: u8,
    },

    /// I/O error on the transport layer.
    #[error("I/O error on {transport}: {source}")]
    Io {
        /// Transport description.
        transport: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// UDS socket file does not exist.
    #[error("UDS socket not found: {path}")]
    SocketNotFound {
        /// Expected socket path.
        path: PathBuf,
    },

    /// Serialization failure for request params.
    #[error("failed to serialize request for {method}: {source}")]
    Serialization {
        /// Method being called.
        method: String,
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// Permission denied by token verifier or method gate.
    #[error("permission denied for {primal}.{method}")]
    PermissionDenied {
        /// Target primal.
        primal: String,
        /// Method attempted.
        method: String,
    },
}

impl IpcError {
    /// Classify which phase of the IPC lifecycle this error occurred in.
    #[must_use]
    pub const fn phase(&self) -> IpcPhase {
        match self {
            Self::DiscoveryFailed { .. } | Self::SocketNotFound { .. } => IpcPhase::Discover,
            Self::Connection { .. } => IpcPhase::Connect,
            Self::Serialization { .. } => IpcPhase::Serialize,
            Self::Timeout { .. } => IpcPhase::Send,
            Self::Io { .. } => IpcPhase::Receive,
            Self::MalformedResponse { .. }
            | Self::RpcError { .. }
            | Self::PermissionDenied { .. } => IpcPhase::Parse,
        }
    }

    /// Whether the error is likely transient and worth retrying.
    ///
    /// Connection resets, timeouts, and I/O errors are retriable.
    /// RPC errors, permission denied, and parse failures are not.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::Connection { .. } | Self::Timeout { .. } | Self::Io { .. }
        )
    }

    /// Whether the pipeline can continue in degraded mode despite this error.
    ///
    /// Discovery and connection failures are recoverable (primal is optional).
    /// Permission and protocol errors indicate misconfiguration.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Connection { .. }
                | Self::Timeout { .. }
                | Self::Io { .. }
                | Self::DiscoveryFailed { .. }
                | Self::SocketNotFound { .. }
        )
    }

    /// Whether this is a connection-layer failure (transport unreachable).
    #[must_use]
    pub const fn is_connection_error(&self) -> bool {
        matches!(
            self,
            Self::Connection { .. } | Self::SocketNotFound { .. } | Self::DiscoveryFailed { .. }
        )
    }

    /// Whether the remote method was not found (JSON-RPC -32601).
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::RpcError { code, .. } if *code == -32601)
    }

    /// Classify a raw I/O error into a semantic `IpcError`.
    pub fn classify_io(transport: impl Into<String>, err: std::io::Error) -> Self {
        let transport = transport.into();
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::ConnectionReset => {
                Self::Connection {
                    primal: String::new(),
                    transport,
                    message: err.to_string(),
                }
            }
            std::io::ErrorKind::TimedOut => Self::Timeout {
                primal: String::new(),
                method: String::new(),
                elapsed_ms: 0,
            },
            _ => Self::Io {
                transport,
                source: err,
            },
        }
    }
}

/// Phase-wrapped IPC error — preserves the original error with pipeline phase context.
///
/// Mirrors `primalSpring::ipc::PhasedIpcError`. Wraps any `IpcError` with the
/// validation pipeline phase (or other higher-level operation) that was active.
#[derive(Debug, thiserror::Error)]
#[error("[{pipeline_phase}] {source}")]
pub struct PhasedIpcError {
    /// The pipeline or operation phase (e.g. `health_check`, `provenance_commit`).
    pub pipeline_phase: String,
    /// The underlying IPC error.
    #[source]
    pub source: IpcError,
}

impl PhasedIpcError {
    /// Wrap an IPC error with pipeline phase context.
    #[must_use]
    pub fn new(phase: impl Into<String>, source: IpcError) -> Self {
        Self {
            pipeline_phase: phase.into(),
            source,
        }
    }

    /// Whether the underlying error is retriable.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        self.source.is_retriable()
    }

    /// Whether the pipeline can degrade past this error.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        self.source.is_recoverable()
    }
}

/// Degradation level for a primal that failed health checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DegradationLevel {
    /// Primal is fully healthy.
    Healthy,
    /// Primal responded but reported non-ready state.
    Degraded,
    /// Primal is unreachable — operations will be skipped with warnings.
    Unreachable,
}

impl std::fmt::Display for DegradationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => f.write_str("healthy"),
            Self::Degraded => f.write_str("degraded"),
            Self::Unreachable => f.write_str("unreachable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retriable_errors() {
        let timeout = IpcError::Timeout {
            primal: String::from("nestgate"),
            method: String::from("storage.store"),
            elapsed_ms: 5000,
        };
        assert!(timeout.is_retriable());

        let rpc = IpcError::RpcError {
            primal: String::from("nestgate"),
            method: String::from("storage.store"),
            code: -32600,
            message: String::from("invalid request"),
        };
        assert!(!rpc.is_retriable());
    }

    #[test]
    fn recoverable_errors() {
        let discovery = IpcError::DiscoveryFailed {
            primal: String::from("songbird"),
            tiers_tried: 3,
        };
        assert!(discovery.is_recoverable());

        let permission = IpcError::PermissionDenied {
            primal: String::from("beardog"),
            method: String::from("crypto.verify"),
        };
        assert!(!permission.is_recoverable());
    }

    #[test]
    fn phase_classification() {
        let discovery = IpcError::DiscoveryFailed {
            primal: String::from("x"),
            tiers_tried: 3,
        };
        assert_eq!(discovery.phase(), IpcPhase::Discover);

        let conn = IpcError::Connection {
            primal: String::from("x"),
            transport: String::from("TCP"),
            message: String::from("refused"),
        };
        assert_eq!(conn.phase(), IpcPhase::Connect);

        let ser = IpcError::Serialization {
            method: String::from("rpc.test"),
            source: serde_json::from_str::<serde_json::Value>("bad").unwrap_err(),
        };
        assert_eq!(ser.phase(), IpcPhase::Serialize);
    }

    #[test]
    fn method_not_found_detection() {
        let not_found = IpcError::RpcError {
            primal: String::from("toadstool"),
            method: String::from("compute.run"),
            code: -32601,
            message: String::from("method not found"),
        };
        assert!(not_found.is_method_not_found());

        let other = IpcError::RpcError {
            primal: String::from("toadstool"),
            method: String::from("compute.run"),
            code: -32600,
            message: String::from("invalid request"),
        };
        assert!(!other.is_method_not_found());
    }

    #[test]
    fn phased_error_wrapping() {
        let inner = IpcError::Timeout {
            primal: String::from("nestgate"),
            method: String::from("storage.store"),
            elapsed_ms: 30000,
        };
        let phased = PhasedIpcError::new("artifact_registration", inner);
        assert!(phased.is_retriable());
        assert!(phased.to_string().contains("[artifact_registration]"));
    }

    #[test]
    fn classify_io_connection_refused() {
        let err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let classified = IpcError::classify_io("TCP:127.0.0.1:9500", err);
        assert!(classified.is_connection_error());
    }
}
