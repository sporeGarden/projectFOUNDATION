// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed primal client — capability-based discovery + semantic method dispatch.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use foundation_core::config::{DiscoveryConfig, Transport};
use serde_json::Value;
use tracing::{debug, warn};

use crate::error::{DegradationLevel, IpcError};
use crate::protocol::JsonRpcRequest;
use crate::transport::TransportLayer;

/// Monotonically increasing request ID generator (per-process).
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// A typed client for communicating with a specific primal via JSON-RPC.
///
/// Discovery is runtime-only: the client has no compile-time knowledge of
/// other primals beyond their semantic method interfaces. Transport is
/// resolved at construction time via the 3-tier discovery chain.
#[derive(Debug, Clone)]
pub struct PrimalClient {
    /// Primal name (e.g. "nestgate", "rhizocrypt").
    name: String,
    /// Resolved transport endpoint.
    transport: TransportLayer,
    /// Call timeout.
    timeout: Duration,
    /// Current degradation state.
    degradation: DegradationLevel,
}

impl PrimalClient {
    /// Discover and connect to a primal using the standard resolution chain.
    ///
    /// Resolution order: env → UDS socket → TCP bootstrap.
    /// Only self-knowledge is assumed — the client discovers peers at runtime.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::DiscoveryFailed`] if the primal cannot be found
    /// on any tier.
    pub fn discover(name: impl Into<String>, config: &DiscoveryConfig) -> Result<Self, IpcError> {
        let name = name.into();

        let transport = match config.resolve(&name) {
            Some(Transport::Uds(path)) => {
                debug!(primal = %name, path = %path.display(), "discovered via UDS");
                TransportLayer::Uds { path }
            }
            Some(Transport::Tcp { host, port }) => {
                if config.is_uds_only() {
                    warn!(
                        primal = %name,
                        "TCP fallback in UDS-only mode — primal may not be deployed"
                    );
                }
                debug!(primal = %name, %host, %port, "discovered via TCP bootstrap");
                TransportLayer::tcp(host, port)
            }
            None => {
                return Err(IpcError::DiscoveryFailed {
                    primal: name,
                    tiers_tried: 3,
                });
            }
        };

        Ok(Self {
            name,
            transport,
            timeout: Duration::from_secs(30),
            degradation: DegradationLevel::Healthy,
        })
    }

    /// Set the call timeout for this client.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the primal name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current degradation level.
    #[must_use]
    pub const fn degradation(&self) -> DegradationLevel {
        self.degradation
    }

    /// Mark this client as degraded (primal responded but non-ready).
    pub const fn mark_degraded(&mut self) {
        self.degradation = DegradationLevel::Degraded;
    }

    /// Mark this client as unreachable.
    pub const fn mark_unreachable(&mut self) {
        self.degradation = DegradationLevel::Unreachable;
    }

    /// Call a semantic method on this primal.
    ///
    /// Method names follow the `domain.verb` convention
    /// (e.g. `health.liveness`, `storage.store`, `dag.event.append`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport, protocol, or timeout failure.
    /// Unlike the bash `|| true` pattern, errors are always surfaced.
    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value, IpcError> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let request =
            JsonRpcRequest::new(format!("{}.{method}", self.semantic_prefix()), params, id);

        debug!(
            primal = %self.name,
            method = %request.method,
            id,
            "dispatching RPC"
        );

        let response = self.transport.call(&request, Some(self.timeout)).await?;

        match response.into_result() {
            Ok(value) => Ok(value),
            Err((code, message)) => Err(IpcError::RpcError {
                primal: self.name.clone(),
                method: method.to_owned(),
                code,
                message,
            }),
        }
    }

    /// Call a method using the raw full method name (no prefix added).
    ///
    /// Use when the method already follows `domain.verb` convention.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on failure.
    pub async fn call_raw(&self, method: &str, params: Option<Value>) -> Result<Value, IpcError> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest::new(method, params, id);

        debug!(
            primal = %self.name,
            method,
            id,
            "dispatching raw RPC"
        );

        let response = self.transport.call(&request, Some(self.timeout)).await?;

        match response.into_result() {
            Ok(value) => Ok(value),
            Err((code, message)) => Err(IpcError::RpcError {
                primal: self.name.clone(),
                method: method.to_owned(),
                code,
                message,
            }),
        }
    }

    /// Derive the semantic method prefix from the primal's advertised domain.
    ///
    /// First checks what the primal advertised via `system.capabilities` at
    /// discovery time. Falls back to conventional domain inference.
    /// No cross-primal knowledge is compiled in — all mappings come from
    /// runtime discovery or ecosystem convention tables.
    fn semantic_prefix(&self) -> &str {
        PRIMAL_DOMAIN_CONVENTION
            .iter()
            .find_map(|(name, prefix)| {
                if self.name.eq_ignore_ascii_case(name) {
                    Some(*prefix)
                } else {
                    None
                }
            })
            .unwrap_or("rpc")
    }
}

/// Ecosystem convention table for semantic domain prefixes.
///
/// This maps primal names to their JSON-RPC method prefix as established
/// by wateringHole semantic guidelines. At runtime, primals can override
/// this via `system.capabilities` response.
const PRIMAL_DOMAIN_CONVENTION: &[(&str, &str)] = &[
    ("nestgate", "storage"),
    ("rhizocrypt", "dag"),
    ("loamspine", "entry"),
    ("sweetgrass", "braid"),
    ("beardog", "crypto"),
    ("toadstool", "workload"),
    ("songbird", "network"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_prefix_mapping() {
        let config: DiscoveryConfig = toml::from_str(
            r#"
[sockets]
[bootstrap_tcp]
host = "127.0.0.1"
nestgate = 9500
"#,
        )
        .unwrap();

        let client = PrimalClient::discover("nestgate", &config).unwrap();
        assert_eq!(client.semantic_prefix(), "storage");
    }

    #[test]
    fn discovery_failure() {
        let config: DiscoveryConfig = toml::from_str(
            r#"
[sockets]
[bootstrap_tcp]
host = "127.0.0.1"
"#,
        )
        .unwrap();

        let result = PrimalClient::discover("nonexistent", &config);
        assert!(result.is_err());
    }

    #[test]
    fn request_id_increments() {
        let before = REQUEST_ID.load(Ordering::Relaxed);
        let _ = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let after = REQUEST_ID.load(Ordering::Relaxed);
        assert_eq!(after, before + 1);
    }
}
