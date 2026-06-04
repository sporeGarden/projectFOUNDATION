// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Manifest-driven data fetch with BLAKE3 content addressing.
//!
//! Replaces `deploy/fetch_sources.sh` — reads source manifests from
//! `data/sources/*.toml`, fetches external data (NCBI, `UniProt`, KEGG,
//! literature DOIs), computes BLAKE3 hashes, and verifies integrity.

pub mod fetcher;
pub mod hasher;
pub mod registry;

pub use fetcher::{
    DEFAULT_FILE_EXTENSION, FetchConfig, FetchError, FetchResult, SourceFetcher, domains,
};
pub use hasher::blake3_file;
pub use registry::ArtifactRegistry;
