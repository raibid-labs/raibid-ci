//! Mirror command implementation
//!
//! Handles repository mirroring from GitHub to Gitea

use anyhow::{Context, Result};
use raibid_common::config::Config;
use raibid_common::mirroring::MirroringService;
use tracing::{error, info};

use crate::cli::{MirrorCommand, MirrorSubcommand};

/// Handle mirror command
pub async fn handle(cmd: &MirrorCommand) -> Result<()> {
    match &cmd.command {
        MirrorSubcommand::Run { dry_run, json } => execute_mirror(*dry_run, *json).await,
    }
}

/// Execute repository mirroring
async fn execute_mirror(dry_run: bool, json: bool) -> Result<()> {
    info!("Starting repository mirroring");

    if dry_run {
        info!("DRY RUN MODE - no changes will be made");
    }

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    if !config.mirroring.enabled {
        error!("Mirroring is not enabled in configuration");
        anyhow::bail!("Mirroring disabled. Set mirroring.enabled: true in raibid.yaml");
    }

    if !json {
        println!("\n========================================");
        println!("    REPOSITORY MIRRORING");
        println!("========================================");
        println!("Organizations: {}", config.mirroring.organizations.len());
        println!("Individual repos: {}", config.mirroring.repositories.len());
        println!("========================================\n");
    }

    if dry_run {
        if json {
            println!(
                "{{\"dry_run\":true,\"organizations\":{},\"repositories\":{}}}",
                config.mirroring.organizations.len(),
                config.mirroring.repositories.len()
            );
        } else {
            println!("DRY RUN - Would mirror:");
            for org in &config.mirroring.organizations {
                println!("  - Organization: {}", org.name);
                if org.private_only {
                    println!("    (private repositories only)");
                }
            }
            for repo in &config.mirroring.repositories {
                println!("  - Repository: {}", repo.source);
            }
        }
        return Ok(());
    }

    // Create mirroring service
    let service = MirroringService::new(config.mirroring.clone(), config.gitea.clone())
        .context("Failed to create mirroring service")?;

    // Execute mirroring
    let result = service.mirror_all().await.context("Mirroring failed")?;

    // Print results
    if json {
        println!(
            "{{\"total\":{},\"mirrored\":{},\"failed\":{},\"errors\":{}}}",
            result.total_repos,
            result.mirrored,
            result.failed,
            serde_json::to_string(&result.errors)?
        );
    } else {
        println!("\n========================================");
        println!("       MIRRORING COMPLETE");
        println!("========================================");
        println!("Total repositories: {}", result.total_repos);
        println!("Successfully mirrored: {}", result.mirrored);
        println!("Failed: {}", result.failed);

        if !result.errors.is_empty() {
            println!("\nErrors:");
            for error in &result.errors {
                println!("  ✗ {}", error);
            }
        }

        if result.mirrored > 0 {
            println!("\n✓ Mirroring completed successfully!");
            println!("\nRepositories are now available in Gitea at:");
            println!("  {}", config.gitea.url);

            println!("\nTo trigger a test build job:");
            println!("  1. Find a mirrored repository name");
            println!("  2. Run: raibid jobs trigger --repo <org>/<repo> --branch main");
        }
    }

    if result.failed > 0 {
        anyhow::bail!("{} repositories failed to mirror", result.failed);
    }

    Ok(())
}
