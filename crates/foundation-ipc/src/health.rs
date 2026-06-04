// SPDX-License-Identifier: AGPL-3.0-or-later
//! Health check operations — the mandatory triad: liveness, readiness, check.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::client::PrimalClient;
use crate::error::{DegradationLevel, IpcError};

/// Result of the health triad check for a single primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Primal name.
    pub primal: String,
    /// Whether `health.liveness` succeeded.
    pub alive: bool,
    /// Whether `health.readiness` succeeded.
    pub ready: bool,
    /// Optional version string from `health.check`.
    pub version: Option<String>,
    /// Typed degradation assessment.
    pub level: DegradationLevel,
}

/// Aggregated health of all primals required for a validation run.
#[derive(Debug, Clone, Default)]
pub struct HealthTriad {
    /// Individual primal health results.
    pub results: Vec<HealthStatus>,
}

impl HealthTriad {
    /// Create an empty health triad collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check health for a primal client, updating its degradation state.
    ///
    /// Performs the standard three-call health triad:
    /// 1. `health.liveness` — is the process alive?
    /// 2. `health.readiness` — is it ready to serve?
    /// 3. `health.check` — version and status metadata.
    ///
    /// Returns the index into `results` for the just-recorded status.
    /// Unlike bash `|| true`, failures are recorded with context rather than silenced.
    pub async fn check(&mut self, client: &mut PrimalClient) -> usize {
        let primal = client.name().to_owned();

        let alive = client
            .call_raw(crate::methods::health::LIVENESS, Some(json!({})))
            .await
            .is_ok();

        if !alive {
            client.mark_unreachable();
            self.results.push(HealthStatus {
                primal,
                alive: false,
                ready: false,
                version: None,
                level: DegradationLevel::Unreachable,
            });
            return self.results.len() - 1;
        }

        let ready = client
            .call_raw(crate::methods::health::READINESS, Some(json!({})))
            .await
            .is_ok();

        let version = client
            .call_raw(crate::methods::health::CHECK, Some(json!({})))
            .await
            .ok()
            .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from));

        let level = if ready {
            DegradationLevel::Healthy
        } else {
            client.mark_degraded();
            DegradationLevel::Degraded
        };

        self.results.push(HealthStatus {
            primal,
            alive,
            ready,
            version,
            level,
        });
        self.results.len() - 1
    }

    /// Check whether all required primals are at least alive.
    #[must_use]
    pub fn all_alive(&self) -> bool {
        self.results.iter().all(|r| r.alive)
    }

    /// Check whether all required primals are ready.
    #[must_use]
    pub fn all_ready(&self) -> bool {
        self.results.iter().all(|r| r.ready)
    }

    /// Get primals that are unreachable.
    #[must_use]
    pub fn unreachable_primals(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|r| !r.alive)
            .map(|r| r.primal.as_str())
            .collect()
    }

    /// Produce a human-readable summary for logging.
    #[must_use]
    pub fn summary(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for HealthTriad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.results.len();
        let alive = self.results.iter().filter(|r| r.alive).count();
        let ready = self.results.iter().filter(|r| r.ready).count();
        write!(f, "{ready}/{total} ready, {alive}/{total} alive")
    }
}

/// Check health for a set of primal clients.
///
/// # Errors
///
/// Returns an error only if a *required* primal (per the graph) is unreachable.
/// Optional primals that are unreachable produce warnings but not errors.
pub async fn check_required_primals(
    clients: &mut [PrimalClient],
    required: &[&str],
) -> Result<HealthTriad, IpcError> {
    let mut triad = HealthTriad::new();

    for client in clients.iter_mut() {
        triad.check(client).await;
    }

    let unreachable: Vec<&str> = triad
        .unreachable_primals()
        .into_iter()
        .filter(|p| required.contains(p))
        .collect();

    if let Some(first) = unreachable.first() {
        return Err(IpcError::Connection {
            primal: (*first).to_owned(),
            transport: String::from(foundation_core::primal_names::slugs::DISCOVERY),
            message: format!("required primal(s) unreachable: {}", unreachable.join(", ")),
        });
    }

    Ok(triad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_triad_empty() {
        let triad = HealthTriad::new();
        assert!(triad.all_alive());
        assert!(triad.all_ready());
        assert!(triad.unreachable_primals().is_empty());
        assert_eq!(triad.summary(), "0/0 ready, 0/0 alive");
    }

    #[test]
    fn health_triad_summary_with_results() {
        let triad = HealthTriad {
            results: vec![
                HealthStatus {
                    primal: String::from("nestgate"),
                    alive: true,
                    ready: true,
                    version: Some(String::from("1.0")),
                    level: DegradationLevel::Healthy,
                },
                HealthStatus {
                    primal: String::from("rhizocrypt"),
                    alive: true,
                    ready: false,
                    version: None,
                    level: DegradationLevel::Degraded,
                },
                HealthStatus {
                    primal: String::from("songbird"),
                    alive: false,
                    ready: false,
                    version: None,
                    level: DegradationLevel::Unreachable,
                },
            ],
        };

        assert!(!triad.all_alive());
        assert!(!triad.all_ready());
        assert_eq!(triad.unreachable_primals(), vec!["songbird"]);
        assert_eq!(triad.summary(), "1/3 ready, 2/3 alive");
    }

    #[test]
    fn health_status_serialization() {
        let status = HealthStatus {
            primal: String::from("nestgate"),
            alive: true,
            ready: true,
            version: Some(String::from("0.2.0")),
            level: DegradationLevel::Healthy,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"primal\":\"nestgate\""));
        assert!(json.contains("\"version\":\"0.2.0\""));
        assert!(json.contains("\"level\":\"healthy\""));
    }
}
