// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! sporePrint content generation from pseudoSpore registry and validation evidence.
//!
//! This crate bridges the gap between lithoSpore's pseudoSpore artifacts and
//! sporePrint's Zola static site. It reads registry data, generates Markdown
//! gallery pages with Zola front matter, and writes them to the `sporeprint/`
//! directory for auto-merge into the sporePrint site.

pub mod domain_profile;
pub mod gallery;
pub mod registry;

pub use domain_profile::{DomainProfileHeader, ProfileIndex};
pub use gallery::GalleryGenerator;
pub use registry::{PseudoSporeEntry, SporeRegistry};
