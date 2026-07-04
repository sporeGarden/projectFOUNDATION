// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation target types — expected outcomes for scientific reproducibility.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Default explanation when a target has no numeric tolerance defined.
const QUALITATIVE_FALLBACK: &str = "no numeric tolerance defined";

/// How close a measured value must be to the expected value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tolerance {
    /// Absolute numeric tolerance (e.g. ±0.5).
    Absolute(f64),
    /// Structured tolerance with explicit type.
    Typed(TypedTolerance),
}

/// Explicitly typed tolerance for disambiguation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TypedTolerance {
    /// Absolute difference.
    #[serde(rename = "absolute")]
    Absolute {
        /// Maximum allowed absolute difference.
        value: f64,
    },
    /// Percentage difference.
    #[serde(rename = "percentage")]
    Percentage {
        /// Maximum allowed percentage difference.
        value: f64,
    },
    /// Qualitative (pass/fail description, no numeric comparison).
    #[serde(rename = "qualitative")]
    Qualitative {
        /// Description of what constitutes passing.
        description: String,
    },
}

impl Tolerance {
    /// Check whether an observed value passes this tolerance against an expected value.
    ///
    /// Returns `true` if the observed value is within tolerance of expected.
    /// For qualitative tolerances, always returns `true` (manual review required).
    #[must_use]
    pub fn passes(&self, expected: f64, observed: f64) -> bool {
        match self {
            Self::Absolute(tol) => (observed - expected).abs() <= *tol,
            Self::Typed(TypedTolerance::Absolute { value }) => {
                (observed - expected).abs() <= *value
            }
            Self::Typed(TypedTolerance::Percentage { value }) => {
                if expected.abs() < f64::EPSILON {
                    observed.abs() < *value
                } else {
                    ((observed - expected) / expected).abs() * 100.0 <= *value
                }
            }
            Self::Typed(TypedTolerance::Qualitative { .. }) => true,
        }
    }
}

/// A single validation target from a thread's target manifest.
///
/// Handles both schema variants found in the wild:
/// - `tolerance = 0.5` (absolute)
/// - `tolerance_pct = 15` (percentage)
/// - Both absent (qualitative target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    /// Unique identifier within the manifest.
    pub id: String,
    /// Associated baseCamp paper ID.
    pub paper: String,
    /// Human-readable description.
    pub description: String,
    /// Expected numeric value (if quantitative).
    #[serde(default)]
    pub expected_value: Option<f64>,
    /// Expected string value (if qualitative).
    #[serde(default)]
    pub expected_string: Option<String>,
    /// Unit of measurement.
    #[serde(default)]
    pub unit: Option<String>,
    /// Absolute tolerance (±value).
    #[serde(default)]
    pub tolerance: Option<f64>,
    /// Percentage tolerance (±value%).
    #[serde(default)]
    pub tolerance_pct: Option<f64>,
    /// Data provenance source description.
    #[serde(default)]
    pub source: Option<String>,
    /// Validating spring.
    pub spring: String,
    /// Spring path for result lookup.
    #[serde(default)]
    pub spring_path: Option<String>,
    /// BLAKE3 hash of the result artifact (empty until validated).
    #[serde(default)]
    pub blake3: String,
    /// Whether this target has been validated.
    #[serde(default)]
    pub validated: bool,
    /// Additional notes.
    #[serde(default)]
    pub notes: Option<String>,
}

/// Metadata header of a targets manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetsMeta {
    /// Thread number.
    pub thread: u32,
    /// Thread name.
    pub thread_name: String,
    /// Path to expression doc.
    #[serde(default)]
    pub expression: Option<String>,
    /// Last update date.
    #[serde(default)]
    pub last_updated: Option<String>,
    /// Declared target count.
    pub total_targets: u32,
    /// Count of validated targets (declared in manifest).
    #[serde(default)]
    pub validated_count: Option<u32>,
    /// Count of pending targets.
    #[serde(default)]
    pub pending_count: Option<u32>,
    /// Braid/evidence narrative.
    #[serde(default)]
    pub braid_evidence: Option<String>,
    /// Upstream spring name.
    #[serde(default)]
    pub upstream_spring: Option<String>,
    /// Upstream spring status.
    #[serde(default)]
    pub upstream_status: Option<String>,
}

/// Complete targets manifest file.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetsManifest {
    /// File metadata.
    pub meta: TargetsMeta,
    /// All validation targets.
    pub targets: Vec<Target>,
}

impl TargetsManifest {
    /// Load a targets manifest from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] on read failure, [`CoreError::TomlParse`] on parse failure,
    /// or [`CoreError::Validation`] if declared count mismatches actual targets.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| CoreError::io(path, e))?;
        let manifest: Self = toml::from_str(&content).map_err(|e| CoreError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })?;

        let declared = manifest.meta.total_targets as usize;
        let actual = manifest.targets.len();
        if declared != actual {
            return Err(CoreError::Validation {
                manifest: path.to_path_buf(),
                message: format!(
                    "meta.total_targets={declared} but found {actual} [[targets]] entries"
                ),
            });
        }

        Ok(manifest)
    }

    /// Count how many targets have been validated.
    #[must_use]
    pub fn validated_count(&self) -> usize {
        self.targets.iter().filter(|t| t.validated).count()
    }

    /// Count how many targets have BLAKE3 anchors.
    #[must_use]
    pub fn hashed_count(&self) -> usize {
        self.targets.iter().filter(|t| !t.blake3.is_empty()).count()
    }
}

/// Result of comparing an observed value against a target.
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResult {
    /// Target that was checked.
    pub target_id: String,
    /// Whether the comparison passed.
    pub passed: bool,
    /// Observed value.
    pub observed: Option<f64>,
    /// Expected value.
    pub expected: Option<f64>,
    /// Absolute difference (if numeric).
    pub delta: Option<f64>,
    /// Human-readable explanation.
    pub explanation: String,
}

impl Target {
    /// Resolve the effective tolerance from the two optional fields.
    ///
    /// Priority: `tolerance` (absolute) > `tolerance_pct` (percentage) > qualitative.
    #[must_use]
    pub fn resolved_tolerance(&self) -> Tolerance {
        self.tolerance.map_or_else(
            || {
                self.tolerance_pct.map_or_else(
                    || {
                        Tolerance::Typed(TypedTolerance::Qualitative {
                            description: String::from(QUALITATIVE_FALLBACK),
                        })
                    },
                    |pct| Tolerance::Typed(TypedTolerance::Percentage { value: pct }),
                )
            },
            Tolerance::Absolute,
        )
    }

    /// Compare an observed numeric value against this target's tolerance.
    #[must_use]
    pub fn compare(&self, observed: f64) -> ComparisonResult {
        let expected = self.expected_value.unwrap_or(0.0);
        let tol = self.resolved_tolerance();
        let passed = tol.passes(expected, observed);
        let delta = (observed - expected).abs();

        ComparisonResult {
            target_id: self.id.clone(),
            passed,
            observed: Some(observed),
            expected: Some(expected),
            delta: Some(delta),
            explanation: if passed {
                format!("PASS: δ={delta:.6} within tolerance")
            } else {
                format!("FAIL: δ={delta:.6} exceeds tolerance")
            },
        }
    }

    /// Whether this target is quantitative (has numeric expected value + tolerance).
    #[must_use]
    pub const fn is_quantitative(&self) -> bool {
        self.expected_value.is_some() && (self.tolerance.is_some() || self.tolerance_pct.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_tolerance_passes() {
        let tol = Tolerance::Absolute(0.5);
        assert!(tol.passes(100.0, 100.3));
        assert!(tol.passes(100.0, 99.5));
        assert!(!tol.passes(100.0, 100.6));
    }

    #[test]
    fn percentage_tolerance_passes() {
        let tol = Tolerance::Typed(TypedTolerance::Percentage { value: 5.0 });
        assert!(tol.passes(100.0, 104.0));
        assert!(!tol.passes(100.0, 106.0));
    }

    #[test]
    fn percentage_tolerance_zero_expected() {
        let tol = Tolerance::Typed(TypedTolerance::Percentage { value: 0.5 });
        assert!(tol.passes(0.0, 0.001));
        assert!(!tol.passes(0.0, 1.0));
    }

    #[test]
    fn qualitative_always_passes() {
        let tol = Tolerance::Typed(TypedTolerance::Qualitative {
            description: String::from("manual check"),
        });
        assert!(tol.passes(0.0, 999.0));
    }

    #[test]
    fn target_compare_pass() {
        let target = Target {
            id: String::from("test_target"),
            paper: String::from("07"),
            description: String::from("test"),
            expected_value: Some(1.0),
            expected_string: None,
            unit: Some(String::from("m/s")),
            tolerance: Some(0.1),
            tolerance_pct: None,
            source: None,
            spring: String::from("hotSpring"),
            spring_path: None,
            blake3: String::new(),
            validated: false,
            notes: None,
        };

        let result = target.compare(1.05);
        assert!(result.passed);
        assert!(result.delta.is_some_and(|d| d < 0.1));
    }

    #[test]
    fn target_compare_fail() {
        let target = Target {
            id: String::from("test_target"),
            paper: String::from("07"),
            description: String::from("test"),
            expected_value: Some(0.0),
            expected_string: None,
            unit: None,
            tolerance: Some(0.5),
            tolerance_pct: None,
            source: None,
            spring: String::from("hotSpring"),
            spring_path: None,
            blake3: String::new(),
            validated: false,
            notes: None,
        };

        let result = target.compare(0.6);
        assert!(!result.passed);
    }

    #[test]
    fn target_compare_with_percentage_tolerance() {
        let target = Target {
            id: String::from("pct_target"),
            paper: String::from("B1"),
            description: String::from("rate"),
            expected_value: Some(8.9e-11),
            expected_string: None,
            unit: Some(String::from("per_bp_per_generation")),
            tolerance: None,
            tolerance_pct: Some(15.0),
            source: None,
            spring: String::from("groundSpring"),
            spring_path: None,
            blake3: String::new(),
            validated: false,
            notes: None,
        };

        // 10% off should pass with 15% tolerance
        let result = target.compare(8.9e-11 * 1.10);
        assert!(result.passed);

        // 20% off should fail
        let result = target.compare(8.9e-11 * 1.20);
        assert!(!result.passed);
    }

    #[test]
    fn target_is_quantitative() {
        let quant = Target {
            id: String::from("q"),
            paper: String::from("07"),
            description: String::from("quant"),
            expected_value: Some(1.0),
            expected_string: None,
            unit: None,
            tolerance: Some(0.1),
            tolerance_pct: None,
            source: None,
            spring: String::from("hotSpring"),
            spring_path: None,
            blake3: String::new(),
            validated: false,
            notes: None,
        };
        assert!(quant.is_quantitative());

        let qual = Target {
            id: String::from("q"),
            paper: String::from("07"),
            description: String::from("qual"),
            expected_value: None,
            expected_string: Some(String::from("pass")),
            unit: None,
            tolerance: None,
            tolerance_pct: None,
            source: None,
            spring: String::from("hotSpring"),
            spring_path: None,
            blake3: String::new(),
            validated: false,
            notes: None,
        };
        assert!(!qual.is_quantitative());
    }

    #[test]
    fn parse_targets_manifest() {
        let toml_str = r#"
[meta]
thread = 2
thread_name = "Plasma Physics"
last_updated = "2026-05-11"
total_targets = 1

[[targets]]
id = "sarkas_energy_drift_max"
paper = "07"
description = "Maximum energy drift"
expected_value = 0.0
unit = "percent"
tolerance = 0.5
source = "hotSpring"
spring = "hotSpring"
blake3 = ""
validated = true
"#;
        let manifest: TargetsManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.targets.len(), 1);
        assert_eq!(manifest.validated_count(), 1);
        assert_eq!(manifest.hashed_count(), 0);
    }
}
