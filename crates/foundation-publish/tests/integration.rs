// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests: registry → gallery → domain profile full pipeline.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

use foundation_publish::domain_profile::ProfileIndex;
use foundation_publish::gallery::{GalleryConfig, GalleryGenerator};
use foundation_publish::registry::SporeRegistry;

const REGISTRY_TOML: &str = r#"
[meta]
last_updated = "2026-05-30"
total_ingested = 2

[[pseudospore]]
name = "hotSpring-CompChem-GuideStone"
version = "1.6.1"
origin = "ecoPrimals/springs/hotSpring"
spring = "springs/hotSpring"
status = "COMPLETE"
modules_pass = 7
modules_total = 8
blake3 = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
description = "Computational chemistry validation"

[[pseudospore]]
name = "healthSpring-BTSP-Probe"
version = "0.1.0"
origin = "ecoPrimals/springs/healthSpring"
spring = "springs/healthSpring"
status = "PARTIAL"
modules_pass = 3
modules_total = 5
"#;

const DOMAIN_PROFILE: &str = r#"
[profile]
id = "clinical-pkpd"
version = "1.0.0"
tools = ["Rust", "Python"]

[translation]
enabled = true

[derivation]
tool = "plumed"

[audit]
config_fidelity = true
scientific_claims = false
"#;

#[test]
fn end_to_end_registry_to_gallery() {
    let dir = tempfile::tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    fs::write(&registry_path, REGISTRY_TOML).unwrap();

    let registry = SporeRegistry::from_file(&registry_path).unwrap();
    assert_eq!(registry.entries.len(), 2);

    let complete = registry.complete_entries();
    assert_eq!(complete.len(), 1);
    assert_eq!(complete[0].name, "hotSpring-CompChem-GuideStone");

    let config = GalleryConfig {
        output_dir: dir.path().join("gallery"),
        ..GalleryConfig::default()
    };
    let generator = GalleryGenerator::new(config);

    let pages = generator.generate_all(&complete).unwrap();
    assert_eq!(pages.len(), 1);
    assert!(pages[0].exists());

    let content = fs::read_to_string(&pages[0]).unwrap();
    // Verify Zola front matter
    assert!(content.starts_with("+++\n"));
    assert!(content.contains("title = \"hotSpring-CompChem-GuideStone v1.6.1\""));
    assert!(content.contains("spring = \"hotSpring\""));
    // Verify content sections
    assert!(content.contains("## Validation Status"));
    assert!(content.contains("7/8"));
    assert!(content.contains("## Access"));
    assert!(content.contains("## Provenance"));

    // Generate index
    let index_path = generator.generate_index(&complete).unwrap();
    let index_content = fs::read_to_string(index_path).unwrap();
    assert!(index_content.contains("pseudoSpore Gallery"));
    assert!(index_content.contains("hotSpring-CompChem-GuideStone"));
}

#[test]
fn domain_profile_scan_and_index() {
    let dir = tempfile::tempdir().unwrap();

    // Create nested spring structure with profile
    let spring_dir = dir.path().join("modules/compchem");
    fs::create_dir_all(&spring_dir).unwrap();
    fs::write(spring_dir.join("domain_profile.toml"), DOMAIN_PROFILE).unwrap();

    // Also create a spring without a profile
    let empty_spring = dir.path().join("modules/other");
    fs::create_dir_all(&empty_spring).unwrap();
    fs::write(empty_spring.join("README.md"), "# No profile here").unwrap();

    let index = ProfileIndex::scan_directory(dir.path(), "testSpring").unwrap();
    assert_eq!(index.profiles.len(), 1);

    let profile = &index.profiles[0];
    assert_eq!(profile.id, "clinical-pkpd");
    assert_eq!(profile.spring, "testSpring");
    assert_eq!(profile.version, "1.0.0");
    assert_eq!(profile.tools, vec!["Rust", "Python"]);
    assert!(profile.capabilities.translation);
    assert!(profile.capabilities.derivation);
    assert!(profile.capabilities.audit);
    assert!(profile.capabilities.figures); // defaults to true when section absent
}

#[test]
fn multi_spring_merge() {
    let dir = tempfile::tempdir().unwrap();

    // Spring A
    let spring_a = dir.path().join("spring_a");
    fs::create_dir_all(&spring_a).unwrap();
    fs::write(
        spring_a.join("domain_profile.toml"),
        "[profile]\nid = \"profile-a\"\nversion = \"1.0\"\n",
    )
    .unwrap();

    // Spring B
    let spring_b = dir.path().join("spring_b");
    fs::create_dir_all(&spring_b).unwrap();
    fs::write(
        spring_b.join("domain_profile.toml"),
        "[profile]\nid = \"profile-b\"\nversion = \"2.0\"\ntools = [\"breseq\"]\n",
    )
    .unwrap();

    let mut index = ProfileIndex::scan_directory(&spring_a, "springA").unwrap();
    let index_b = ProfileIndex::scan_directory(&spring_b, "springB").unwrap();
    index.merge(index_b);

    assert_eq!(index.profiles.len(), 2);

    let breseq_profiles = index.requiring_tool("breseq");
    assert_eq!(breseq_profiles.len(), 1);
    assert_eq!(breseq_profiles[0].spring, "springB");

    let spring_a_profiles = index.from_spring("springA");
    assert_eq!(spring_a_profiles.len(), 1);
}

#[test]
fn registry_from_file_handles_missing_gracefully() {
    let result = SporeRegistry::from_file(&PathBuf::from("/nonexistent/registry.toml"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("/nonexistent/registry.toml"));
}
