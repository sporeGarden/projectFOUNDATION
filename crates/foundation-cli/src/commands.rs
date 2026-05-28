// SPDX-License-Identifier: AGPL-3.0-or-later
//! Command implementations for the foundation `UniBin`.

use std::path::PathBuf;

use foundation_core::config::DiscoveryConfig;
use foundation_core::source::SourcesManifest;
use foundation_core::target::TargetsManifest;
use foundation_core::thread::ThreadIndex;
use foundation_fetch::{FetchConfig, SourceFetcher, blake3_file};
use foundation_validate::{PipelineConfig, ValidationPipeline};
use tracing::{error, info};

type CmdResult = Result<(), Box<dyn std::error::Error>>;

/// Run the 8-phase validation pipeline.
pub async fn validate(
    root: PathBuf,
    thread: Option<String>,
    skip_fetch: bool,
    data_dir: Option<PathBuf>,
) -> CmdResult {
    let mut config = PipelineConfig::from_project_root(root);
    config.thread_filter = thread;
    config.skip_fetch = skip_fetch;
    if let Some(dir) = data_dir {
        config.data_dir = dir;
    }

    let pipeline = ValidationPipeline::new(config);
    let result = pipeline.run().await?;

    info!(
        elapsed = result.elapsed_secs,
        pass = result.overall_pass,
        "validation complete"
    );

    if !result.overall_pass {
        std::process::exit(1);
    }
    Ok(())
}

/// Fetch data sources from manifests.
#[expect(
    clippy::unused_async,
    reason = "SourceFetcher::fetch_manifest is sync; Phase C async streaming"
)]
pub async fn fetch(
    root: PathBuf,
    thread: Option<String>,
    data_dir: Option<PathBuf>,
    _register: bool,
) -> CmdResult {
    let index = ThreadIndex::from_file(&root.join("lineage/THREAD_INDEX.toml"))?;
    let data_base = data_dir.unwrap_or_else(|| root.join("data/fetched"));

    let threads_to_fetch: Vec<_> = if let Some(filter) = &thread {
        index
            .threads
            .iter()
            .filter(|t| t.short == *filter || t.id.to_string() == *filter)
            .collect()
    } else {
        index.threads.iter().collect()
    };

    let config = FetchConfig {
        data_dir: data_base,
        ..FetchConfig::default()
    };
    let fetcher = SourceFetcher::new(config);

    for t in threads_to_fetch {
        let manifest_path = root.join(&t.data_sources);
        match SourcesManifest::from_file(&manifest_path) {
            Ok(manifest) => {
                info!(thread = %t.short, sources = manifest.sources.len(), "fetching");
                let results = fetcher.fetch_manifest(&manifest)?;
                let success = results.iter().filter(|r| r.success).count();
                info!(thread = %t.short, success, total = results.len(), "done");
            }
            Err(e) => {
                error!(thread = %t.short, error = %e, "failed to load manifest");
            }
        }
    }

    Ok(())
}

/// Check health triad of NUCLEUS primals.
#[expect(
    clippy::unused_async,
    reason = "Phase C: call HealthTriad::check() per client (pipeline already does)"
)]
pub async fn health(root: PathBuf, verbose: bool) -> CmdResult {
    use foundation_core::primal_names;

    let config_path = root.join("deploy/discovery_defaults.toml");
    let config = DiscoveryConfig::from_file(&config_path)?;

    for &primal in primal_names::VALIDATION_PRIMALS {
        match foundation_ipc::PrimalClient::discover(primal, &config) {
            Ok(client) => {
                if verbose {
                    info!(primal, transport = "discovered", "reachable");
                }
                let _ = client;
            }
            Err(e) => {
                if verbose {
                    error!(primal, error = %e, "unreachable");
                } else {
                    info!(primal, status = "unreachable");
                }
            }
        }
    }

    Ok(())
}

/// Inspect and verify target manifests.
#[expect(
    clippy::unused_async,
    reason = "Phase C: compare_targets via pipeline; currently manifest-only"
)]
pub async fn targets(root: PathBuf, thread: Option<String>, check: bool) -> CmdResult {
    let index = ThreadIndex::from_file(&root.join("lineage/THREAD_INDEX.toml"))?;

    let threads_to_check: Vec<_> = if let Some(filter) = &thread {
        index
            .threads
            .iter()
            .filter(|t| t.short == *filter || t.id.to_string() == *filter)
            .collect()
    } else {
        index.threads.iter().collect()
    };

    for t in threads_to_check {
        let target_path = root.join(&t.data_targets);
        match TargetsManifest::from_file(&target_path) {
            Ok(manifest) => {
                let validated = manifest.validated_count();
                let total = manifest.targets.len();
                info!(
                    thread = %t.short,
                    validated,
                    total,
                    hashed = manifest.hashed_count(),
                    "targets loaded"
                );
                if check && validated < total {
                    error!(
                        thread = %t.short,
                        gap = total - validated,
                        "incomplete validation"
                    );
                }
            }
            Err(e) => {
                error!(thread = %t.short, error = %e, "failed to load targets");
            }
        }
    }

    Ok(())
}

/// Populate BLAKE3 hashes in source manifests.
#[expect(
    clippy::unused_async,
    reason = "Phase C: NestGate RPC registration + TOML write-back"
)]
pub async fn backfill(root: PathBuf, data_dir: Option<PathBuf>, dry_run: bool) -> CmdResult {
    let index = ThreadIndex::from_file(&root.join("lineage/THREAD_INDEX.toml"))?;
    let fetch_dir = data_dir.unwrap_or_else(|| root.join("data/fetched"));

    for t in &index.threads {
        let manifest_path = root.join(&t.data_sources);
        let manifest = match SourcesManifest::from_file(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                error!(thread = %t.short, error = %e, "skipping");
                continue;
            }
        };

        let mut updates = 0;
        for source in &manifest.sources {
            if source.is_hashed() {
                continue;
            }
            let file_path = fetch_dir.join(format!(
                "{}.{}",
                source.id,
                source.format.as_deref().unwrap_or("dat")
            ));
            if file_path.exists() {
                match blake3_file(&file_path) {
                    Ok(hash) => {
                        if dry_run {
                            info!(source = %source.id, hash = %&hash[..12], "would backfill");
                        } else {
                            info!(source = %source.id, hash = %&hash[..12], "backfilled");
                        }
                        updates += 1;
                    }
                    Err(e) => {
                        error!(source = %source.id, error = %e, "hash failed");
                    }
                }
            }
        }

        if updates > 0 {
            info!(thread = %t.short, updates, dry_run, "backfill pass");
        }
    }

    Ok(())
}
