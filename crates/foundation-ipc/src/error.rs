// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC error types — typed failures replacing `|| true` silent swallowing.

use std::path::PathBuf;

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
        source: serde_json::Error,
    },
}

/// Degradation level for a primal that failed health checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationLevel {
    /// Primal is fully healthy.
    Healthy,
    /// Primal responded but reported non-ready state.
    Degraded,
    /// Primal is unreachable — operations will be skipped with warnings.
    Unreachable,
}
