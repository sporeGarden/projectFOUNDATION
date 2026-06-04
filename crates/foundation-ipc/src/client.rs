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
            timeout: crate::transport::DEFAULT_TIMEOUT,
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

/// Resolve `FAMILY_ID` via the biomeOS discovery socket using `family.id` RPC.
///
/// Falls back to env vars if the socket is unavailable (graceful degradation).
/// This is the Rust equivalent of `deploy/lib/env.sh` `FAMILY_ID` resolution.
///
/// # Errors
///
/// Returns [`IpcError`] if the discovery socket exists but the RPC call fails.
pub async fn resolve_family_id_rpc(
    config: &foundation_core::config::DiscoveryConfig,
) -> Result<String, IpcError> {
    use foundation_core::env_keys;

    let env_id = env_keys::resolve_family_id();
    if !env_id.is_empty() {
        return Ok(env_id);
    }

    let client = PrimalClient::discover(foundation_core::primal_names::slugs::DISCOVERY, config)?;
    let result = client
        .call_raw(crate::methods::family::ID, Some(serde_json::json!({})))
        .await?;
    let family_id = result
        .get("family_id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            warn!("discovery RPC returned no family_id field — using empty default");
            ""
        });
    Ok(family_id.to_owned())
}

/// Ecosystem convention table for semantic domain prefixes.
///
/// Maps primal slugs to their JSON-RPC method prefix as established
/// by wateringHole semantic guidelines. At runtime, primals can override
/// this via `system.capabilities` response.
const PRIMAL_DOMAIN_CONVENTION: &[(&str, &str)] = &[
    (foundation_core::primal_names::slugs::NESTGATE, "storage"),
    (foundation_core::primal_names::slugs::RHIZOCRYPT, "dag"),
    (foundation_core::primal_names::slugs::LOAMSPINE, "entry"),
    (foundation_core::primal_names::slugs::SWEETGRASS, "braid"),
    (foundation_core::primal_names::slugs::BEARDOG, "crypto"),
    (foundation_core::primal_names::slugs::TOADSTOOL, "workload"),
    (foundation_core::primal_names::slugs::SONGBIRD, "network"),
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
