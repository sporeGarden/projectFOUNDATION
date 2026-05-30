// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! 8-phase scientific validation pipeline for projectFOUNDATION.
//!
//! Replaces `deploy/foundation_validate.sh` with typed, testable Rust.
//! Each phase is independently testable and produces structured output
//! rather than shell exit codes.

pub mod compare;
pub mod executor;
pub mod phases;
pub mod pipeline;
pub mod report;

pub use compare::{Observation, compare_targets};
pub use pipeline::{PipelineConfig, ValidationPipeline, ValidationResult};
pub use report::{FetchStatus, ProvenanceIds, ReportWriter};
