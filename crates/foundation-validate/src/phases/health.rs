// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase 1: Health checks — discover and probe validation primals.
//!
//! Gracefully degrades: unreachable primals produce warnings, not errors.
//! The pipeline continues regardless of health status.

use foundation_core::config::DiscoveryConfig;
use foundation_core::primal_names;
use foundation_ipc::PrimalClient;
use foundation_ipc::health::HealthTriad;
use tracing::{info, warn};

/// Result of the health check phase.
#[derive(Debug, Clone)]
pub struct HealthPhaseResult {
    /// Human-readable summary (e.g. `"3/3 healthy"`).
    pub summary: String,
    /// Whether all primals responded healthy.
    pub all_healthy: bool,
    /// Primals that could not be reached.
    pub unreachable: Vec<String>,
}

/// Execute Phase 1: discover and probe health of validation primals.
///
/// Never fails — unreachable primals are recorded as degraded.
pub async fn run(config: &DiscoveryConfig) -> HealthPhaseResult {
    let mut triad = HealthTriad::new();
    let mut clients: Vec<PrimalClient> = Vec::new();
    let mut unreachable = Vec::new();

    for &primal in primal_names::VALIDATION_PRIMALS {
        match PrimalClient::discover(primal, config) {
            Ok(client) => clients.push(client),
            Err(e) => {
                warn!(primal, error = %e, "discovery failed — skipping health check");
                unreachable.push(primal.to_owned());
            }
        }
    }

    for client in &mut clients {
        triad.check(client).await;
    }

    let summary = triad.summary();
    // all_healthy requires both: triad reports ready AND no discovery failures
    let all_healthy = triad.all_ready() && unreachable.is_empty();

    if all_healthy {
        info!(summary = %summary, "all primals healthy");
    } else {
        unreachable.extend(triad.unreachable_primals().iter().map(|s| (*s).to_owned()));
        warn!(
            summary = %summary,
            unreachable = ?unreachable,
            "operating in degraded mode"
        );
    }

    HealthPhaseResult {
        summary,
        all_healthy,
        unreachable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn health_phase_degrades_when_all_unreachable() {
        let config = DiscoveryConfig {
            metadata: None,
            sockets: HashMap::new(),
            bootstrap_tcp: None,
        };

        let result = run(&config).await;
        // All primals fail discovery → not healthy, captured in unreachable
        assert!(!result.all_healthy);
        assert!(!result.unreachable.is_empty());
        assert!(!result.summary.is_empty());
    }
}
