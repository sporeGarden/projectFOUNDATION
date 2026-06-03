// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! JSON-RPC 2.0 IPC clients for projectFOUNDATION primal communication.
//!
//! Implements UDS-first, capability-based discovery following the ecoPrimals
//! 5-tier resolution chain. Each primal client is typed with domain-specific
//! methods using semantic `domain.verb` naming.

pub mod client;
pub mod dashboard;
pub mod error;
pub mod health;
pub mod protocol;
pub mod provenance;
pub mod transport;

pub use client::PrimalClient;
pub use dashboard::EcosystemHealth;
pub use error::{DegradationLevel, IpcError, IpcPhase, PhasedIpcError};
pub use health::{HealthStatus, HealthTriad};
pub use provenance::ProvenanceSession;
pub use transport::TransportLayer;
