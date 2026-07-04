// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Core domain types for projectFOUNDATION.
//!
//! Defines the shared vocabulary: threads, targets, sources, tolerances,
//! workloads, and discovery configuration. All types are deserializable
//! from the TOML manifests in `data/`, `workloads/`, and `deploy/`.

pub mod config;
pub mod env_keys;
pub mod error;
pub mod paths;
pub mod primal_names;
pub mod source;
pub mod target;
pub mod thread;
pub mod time;
pub mod versions;
pub mod workload;

pub use config::DiscoveryConfig;
pub use error::CoreError;
pub use source::Source;
pub use target::{Target, Tolerance};
pub use thread::Thread;
pub use versions::VersionManifest;
pub use workload::Workload;
