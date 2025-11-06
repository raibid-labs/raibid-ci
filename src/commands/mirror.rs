//! Mirror command implementation
//!
//! Handles repository mirroring from GitHub to Gitea

use anyhow::{Context, Result};
use raibid_common::config::Config;
use raibid_common::mirroring::MirroringService;
use tracing::{error, info};

/// Execute repository mirroring
pub async fn execute_mirror() -> Result<()> {
    info!("Starting repository mirroring");

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    if !config.mirroring.enabled {
        error!("Mirroring is not enabled in configuration");
        anyhow::bail!("Mirroring disabled. Set mirroring.enabled: true in raibid.yaml");
    }

    info!("Mirroring enabled for:");
    info!("  - {} organizations", config.mirroring.organizations.len());
    info!(
        "  - {} individual repositories",
        config.mirroring.repositories.len()
    );

    // Create mirroring service
    let service = MirroringService::new(config.mirroring.clone(), config.gitea.clone())
        .context("Failed to create mirroring service")?;

    // Execute mirroring
    let result = service
        .mirror_all()
        .await
        .context("Mirroring failed")?;

    // Print results
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

        println!("\nTo trigger a test build job, run:");
        println!("  redis-cli -h {} -p {} XADD {} \\* \\",
            config.redis.host,
            config.redis.port,
            config.redis.job_stream
        );
        println!("    job '{{\"id\":\"test-1\",\"repo\":\"<org>/<repo>\",\"branch\":\"main\",\"commit\":\"HEAD\",\"status\":\"Pending\"}}'");
    }

    if result.failed > 0 {
        anyhow::bail!("{} repositories failed to mirror", result.failed);
    }

    Ok(())
}
