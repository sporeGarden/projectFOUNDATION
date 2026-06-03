// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized URL constants for sporePrint gallery generation.
//!
//! These are the base URLs used when generating download links and repository
//! references in gallery pages. They can be overridden via `GalleryConfig`.

/// Base URL for pseudoSpore artifact downloads.
pub const DOWNLOAD_BASE: &str = "https://primals.eco/lab/spores";

/// Base URL for Forgejo repository links.
pub const FORGEJO_BASE: &str = "https://git.primals.eco";
