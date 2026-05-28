// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal identity constants — typed names for logging and identity.
//!
//! Follows the primalSpring `primal_names` pattern: typed slug constants
//! used for logging context and identity resolution. Production routing
//! uses capability-based discovery, not primal names.

/// Canonical primal slug identifiers (lowercase, used for discovery keys and logging).
pub mod slugs {
    /// Storage primal.
    pub const NESTGATE: &str = "nestgate";
    /// DAG/provenance primal.
    pub const RHIZOCRYPT: &str = "rhizocrypt";
    /// Ledger/spine primal.
    pub const LOAMSPINE: &str = "loamspine";
    /// Attribution/braid primal.
    pub const SWEETGRASS: &str = "sweetgrass";
    /// Crypto/auth primal.
    pub const BEARDOG: &str = "beardog";
    /// Compute/workload primal.
    pub const TOADSTOOL: &str = "toadstool";
    /// Network/relay primal.
    pub const SONGBIRD: &str = "songbird";
    /// GPU/math primal.
    pub const BARRACUDA: &str = "barracuda";
    /// OS/gateway primal.
    pub const BIOMEOS: &str = "biomeos";
    /// Inference primal.
    pub const SQUIRREL: &str = "squirrel";
}

/// Display-friendly names for UI/logging output.
pub mod display {
    /// Get the display name for a primal slug.
    #[must_use]
    pub fn for_slug(slug: &str) -> &str {
        DISPLAY_TABLE
            .iter()
            .find_map(|(s, d)| if *s == slug { Some(*d) } else { None })
            .unwrap_or(slug)
    }

    const DISPLAY_TABLE: &[(&str, &str)] = &[
        ("nestgate", "NestGate"),
        ("rhizocrypt", "rhizoCrypt"),
        ("loamspine", "loamSpine"),
        ("sweetgrass", "sweetGrass"),
        ("beardog", "BearDog"),
        ("toadstool", "toadStool"),
        ("songbird", "Songbird"),
        ("barracuda", "barraCuda"),
        ("biomeos", "biomeOS"),
        ("squirrel", "squirrel"),
    ];
}

/// Primals required for the foundation validation pipeline.
pub const VALIDATION_PRIMALS: &[&str] = &[
    slugs::NESTGATE,
    slugs::RHIZOCRYPT,
    slugs::LOAMSPINE,
    slugs::SWEETGRASS,
    slugs::TOADSTOOL,
];

/// Provenance trio primals (DAG + ledger + braid).
pub const PROVENANCE_PRIMALS: &[&str] = &[slugs::RHIZOCRYPT, slugs::LOAMSPINE, slugs::SWEETGRASS];

/// Check whether a slug is a known primal.
#[must_use]
pub fn is_known(slug: &str) -> bool {
    ALL_SLUGS.contains(&slug)
}

/// All known primal slugs.
const ALL_SLUGS: &[&str] = &[
    slugs::NESTGATE,
    slugs::RHIZOCRYPT,
    slugs::LOAMSPINE,
    slugs::SWEETGRASS,
    slugs::BEARDOG,
    slugs::TOADSTOOL,
    slugs::SONGBIRD,
    slugs::BARRACUDA,
    slugs::BIOMEOS,
    slugs::SQUIRREL,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_primals() {
        assert!(is_known("nestgate"));
        assert!(is_known("beardog"));
        assert!(!is_known("fakePrimal"));
    }

    #[test]
    fn display_names() {
        assert_eq!(display::for_slug("nestgate"), "NestGate");
        assert_eq!(display::for_slug("beardog"), "BearDog");
        assert_eq!(display::for_slug("unknown"), "unknown");
    }

    #[test]
    fn validation_primals_count() {
        assert_eq!(VALIDATION_PRIMALS.len(), 5);
    }

    #[test]
    fn provenance_trio() {
        assert_eq!(PROVENANCE_PRIMALS.len(), 3);
        for p in PROVENANCE_PRIMALS {
            assert!(is_known(p));
        }
    }
}
