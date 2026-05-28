// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase 6: Target comparison — typed tolerance checking.
//!
//! Replaces the bash regex-grep comparison in `target_compare.sh` with
//! proper numeric parsing and typed `Tolerance` enum dispatch.

use foundation_core::target::{ComparisonResult, TargetsManifest};
use tracing::{debug, info, warn};

/// Results of comparing all targets against observed workload outputs.
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    /// Individual target results.
    pub results: Vec<ComparisonResult>,
    /// Count of passing targets.
    pub passed: usize,
    /// Count of failing targets.
    pub failed: usize,
    /// Count of targets skipped (no observed value available).
    pub skipped: usize,
}

impl ComparisonReport {
    /// Overall pass rate as a fraction [0.0, 1.0].
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "tolerance percentages are approximate by nature"
    )]
    pub fn pass_rate(&self) -> f64 {
        let evaluated = self.passed + self.failed;
        if evaluated == 0 {
            return 0.0;
        }
        self.passed as f64 / evaluated as f64
    }

    /// Whether all evaluated targets passed.
    #[must_use]
    pub const fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Compare observed workload outputs against a targets manifest.
///
/// `observations` is a slice of `(target_id, observed_value)` pairs.
/// Targets without a corresponding observation are marked as skipped.
#[must_use]
pub fn compare_targets(
    manifest: &TargetsManifest,
    observations: &[(String, f64)],
) -> ComparisonReport {
    let mut results = Vec::with_capacity(manifest.targets.len());
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for target in &manifest.targets {
        if let Some(observed) = find_observation(observations, &target.id) {
            let result = target.compare(observed);
            if result.passed {
                debug!(target = %target.id, "PASS");
                passed += 1;
            } else {
                warn!(
                    target = %target.id,
                    observed,
                    expected = ?target.expected_value,
                    "FAIL"
                );
                failed += 1;
            }
            results.push(result);
        } else {
            debug!(target = %target.id, "skipped — no observation");
            skipped += 1;
            results.push(ComparisonResult {
                target_id: target.id.clone(),
                passed: false,
                observed: None,
                expected: target.expected_value,
                delta: None,
                explanation: String::from("no observed value"),
            });
        }
    }

    info!(passed, failed, skipped, "comparison complete");
    ComparisonReport {
        results,
        passed,
        failed,
        skipped,
    }
}

fn find_observation(observations: &[(String, f64)], target_id: &str) -> Option<f64> {
    observations
        .iter()
        .find(|(id, _)| id == target_id)
        .map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation_core::target::Target;

    fn sample_manifest() -> TargetsManifest {
        TargetsManifest {
            meta: foundation_core::target::TargetsMeta {
                thread: 2,
                thread_name: String::from("Plasma"),
                expression: None,
                last_updated: None,
                total_targets: 3,
                validated_count: None,
                pending_count: None,
                braid_evidence: None,
                upstream_spring: None,
                upstream_status: None,
            },
            targets: vec![
                Target {
                    id: String::from("energy_drift"),
                    paper: String::from("07"),
                    description: String::from("Energy drift"),
                    expected_value: Some(0.0),
                    expected_string: None,
                    unit: Some(String::from("percent")),
                    tolerance: Some(0.5),
                    tolerance_pct: None,
                    source: None,
                    spring: String::from("hotSpring"),
                    spring_path: None,
                    blake3: String::new(),
                    validated: false,
                    notes: None,
                },
                Target {
                    id: String::from("rdf_convergence"),
                    paper: String::from("07"),
                    description: String::from("RDF tail"),
                    expected_value: Some(0.0),
                    expected_string: None,
                    unit: None,
                    tolerance: Some(0.02),
                    tolerance_pct: None,
                    source: None,
                    spring: String::from("hotSpring"),
                    spring_path: None,
                    blake3: String::new(),
                    validated: false,
                    notes: None,
                },
                Target {
                    id: String::from("diffusion_k0_g10"),
                    paper: String::from("07"),
                    description: String::from("D* k=0 G=10"),
                    expected_value: Some(0.1253),
                    expected_string: None,
                    unit: None,
                    tolerance: Some(0.05),
                    tolerance_pct: None,
                    source: None,
                    spring: String::from("hotSpring"),
                    spring_path: None,
                    blake3: String::new(),
                    validated: false,
                    notes: None,
                },
            ],
        }
    }

    #[test]
    fn all_pass() {
        let manifest = sample_manifest();
        let observations = vec![
            (String::from("energy_drift"), 0.001),
            (String::from("rdf_convergence"), 0.001),
            (String::from("diffusion_k0_g10"), 0.13),
        ];
        let report = compare_targets(&manifest, &observations);
        assert!(report.all_passed());
        assert_eq!(report.passed, 3);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn one_fail() {
        let manifest = sample_manifest();
        let observations = vec![
            (String::from("energy_drift"), 0.001),
            (String::from("rdf_convergence"), 0.05), // exceeds 0.02 tolerance
            (String::from("diffusion_k0_g10"), 0.13),
        ];
        let report = compare_targets(&manifest, &observations);
        assert!(!report.all_passed());
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn missing_observations() {
        let manifest = sample_manifest();
        let observations = vec![(String::from("energy_drift"), 0.001)];
        let report = compare_targets(&manifest, &observations);
        assert_eq!(report.passed, 1);
        assert_eq!(report.skipped, 2);
    }

    #[test]
    fn pass_rate_calculation() {
        let manifest = sample_manifest();
        let observations = vec![
            (String::from("energy_drift"), 0.001),
            (String::from("rdf_convergence"), 0.05), // fail
        ];
        let report = compare_targets(&manifest, &observations);
        assert!((report.pass_rate() - 0.5).abs() < f64::EPSILON);
    }
}
