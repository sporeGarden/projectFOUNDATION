// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase implementations extracted from the pipeline orchestrator.
//!
//! Each phase is a self-contained function that takes typed inputs and
//! returns structured outputs. The pipeline orchestrator composes them.

pub mod health;
pub mod provenance;
