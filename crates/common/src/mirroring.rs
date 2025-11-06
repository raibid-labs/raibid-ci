//! Repository mirroring orchestration
//!
//! This module orchestrates GitHub to Gitea repository mirroring by:
//! - Fetching repositories from GitHub organizations
//! - Creating mirrors in Gitea
//! - Setting up webhooks for continuous sync

use anyhow::{Context, Result};
use std::env;
use tracing::{info, warn};

use crate::config::{MirroringConfig, OrganizationMirrorConfig};
use crate::gitea_api::GiteaClient;
use crate::github::GitHubClient;

/// Mirroring service orchestrator
pub struct MirroringService {
    github_client: GitHubClient,
    gitea_client: GiteaClient,
    config: MirroringConfig,
}

/// Mirroring result summary
#[derive(Debug)]
pub struct MirroringResult {
    pub total_repos: usize,
    pub mirrored: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl MirroringService {
    /// Create a new mirroring service
    pub fn new(config: MirroringConfig, gitea_config: crate::config::GiteaConfig) -> Result<Self> {
        // Get GitHub token from environment
        let github_token = env::var("GITHUB_TOKEN").ok();
        if github_token.is_none() {
            warn!("GITHUB_TOKEN not set. Public repository access will be limited.");
        }

        // Create GitHub config with token
        let mut github_config = config.github.clone();
        if github_config.token.is_none() {
            github_config.token = github_token.clone();
        }

        let github_client = GitHubClient::new(github_config)?;

        // Gitea admin password is available via environment if needed
        let _gitea_password = env::var("RAIBID_GITEA_ADMIN_PASSWORD")
            .or_else(|_| env::var("GITEA_ADMIN_PASSWORD"))
            .ok();

        // For now, we create a Gitea token placeholder
        // In a real implementation, we'd authenticate with Gitea and get a token
        let gitea_token = env::var("GITEA_TOKEN").ok();

        let gitea_client = GiteaClient::new(gitea_config, gitea_token)?;

        Ok(Self {
            github_client,
            gitea_client,
            config,
        })
    }

    /// Mirror all configured GitHub organizations to Gitea
    pub async fn mirror_all_organizations(&self) -> Result<MirroringResult> {
        info!("Starting organization mirroring");

        let mut total_repos = 0;
        let mut mirrored = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for org_config in &self.config.organizations {
            match self.mirror_organization(org_config).await {
                Ok(result) => {
                    total_repos += result.total_repos;
                    mirrored += result.mirrored;
                    failed += result.failed;
                    errors.extend(result.errors);
                }
                Err(e) => {
                    let error_msg =
                        format!("Failed to mirror organization {}: {}", org_config.name, e);
                    warn!("{}", error_msg);
                    errors.push(error_msg);
                    failed += 1;
                }
            }
        }

        let result = MirroringResult {
            total_repos,
            mirrored,
            failed,
            errors,
        };

        info!(
            "Organization mirroring completed: {}/{} repos mirrored, {} failed",
            result.mirrored, result.total_repos, result.failed
        );

        Ok(result)
    }

    /// Mirror a single GitHub organization to Gitea
    pub async fn mirror_organization(
        &self,
        org_config: &OrganizationMirrorConfig,
    ) -> Result<MirroringResult> {
        info!("Mirroring organization: {}", org_config.name);

        // Fetch repositories from GitHub
        let github_repos = self
            .github_client
            .get_org_repositories(org_config)
            .await
            .context(format!(
                "Failed to fetch repositories for organization: {}",
                org_config.name
            ))?;

        info!(
            "Found {} repositories to mirror from {}",
            github_repos.len(),
            org_config.name
        );

        if github_repos.is_empty() {
            return Ok(MirroringResult {
                total_repos: 0,
                mirrored: 0,
                failed: 0,
                errors: Vec::new(),
            });
        }

        // Determine target organization in Gitea
        let target_org = org_config
            .target_organization
            .as_ref()
            .unwrap_or(&org_config.name);

        // Get GitHub token for authentication
        let github_token = env::var("GITHUB_TOKEN").ok();

        // Mirror repositories to Gitea
        let gitea_repos = self
            .gitea_client
            .mirror_github_repositories(&github_repos, target_org, github_token.clone())
            .await?;

        let mirrored = gitea_repos.len();
        let failed = github_repos.len() - mirrored;

        // Register webhooks if enabled and we have a GitHub token
        if self.config.webhooks.enabled && github_token.is_some() {
            if let Err(e) = self
                .register_webhooks_for_repos(&github_repos, &gitea_repos)
                .await
            {
                warn!("Failed to register webhooks: {}", e);
                // Don't fail the entire mirroring operation due to webhook errors
            }
        }

        Ok(MirroringResult {
            total_repos: github_repos.len(),
            mirrored,
            failed,
            errors: Vec::new(),
        })
    }

    /// Mirror individual configured repositories
    pub async fn mirror_individual_repositories(&self) -> Result<MirroringResult> {
        info!(
            "Mirroring {} individual repositories",
            self.config.repositories.len()
        );

        let total_repos = self.config.repositories.len();
        let mut mirrored = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        let mut mirrored_repos = Vec::new();

        for repo_config in &self.config.repositories {
            // Parse source (e.g., "owner/repo")
            let parts: Vec<&str> = repo_config.source.split('/').collect();
            if parts.len() != 2 {
                let error_msg = format!("Invalid repository source format: {}", repo_config.source);
                warn!("{}", error_msg);
                errors.push(error_msg);
                failed += 1;
                continue;
            }

            let owner = parts[0];
            let repo_name = parts[1];

            // Construct clone URL
            let clone_url = format!("https://github.com/{}.git", repo_config.source);

            // Determine target name
            let default_name = repo_name.to_string();
            let target_name = repo_config.target.as_ref().unwrap_or(&default_name);

            // Get GitHub token for authentication if needed
            let github_token = if repo_config.private {
                env::var("GITHUB_TOKEN").ok()
            } else {
                None
            };

            // Mirror to Gitea admin user (or could be an org)
            let gitea_admin = &self.gitea_client.config.admin_user;

            match self
                .gitea_client
                .create_mirror(
                    &clone_url,
                    target_name,
                    gitea_admin,
                    None,
                    repo_config.private,
                    github_token.clone(),
                )
                .await
            {
                Ok(gitea_repo) => {
                    info!("Successfully mirrored: {}", repo_config.source);
                    mirrored += 1;
                    mirrored_repos.push((owner, repo_name, gitea_repo));
                }
                Err(e) => {
                    let error_msg = format!("Failed to mirror {}: {}", repo_config.source, e);
                    warn!("{}", error_msg);
                    errors.push(error_msg);
                    failed += 1;
                }
            }
        }

        // Register webhooks if enabled and we have a GitHub token
        if self.config.webhooks.enabled && env::var("GITHUB_TOKEN").is_ok() {
            if let Err(e) = self
                .register_webhooks_for_individual_repos(&mirrored_repos)
                .await
            {
                warn!("Failed to register webhooks: {}", e);
                // Don't fail the entire mirroring operation due to webhook errors
            }
        }

        Ok(MirroringResult {
            total_repos,
            mirrored,
            failed,
            errors,
        })
    }

    /// Perform complete mirroring: organizations + individual repos
    pub async fn mirror_all(&self) -> Result<MirroringResult> {
        info!("Starting complete mirroring process");

        let org_result = self.mirror_all_organizations().await?;
        let individual_result = self.mirror_individual_repositories().await?;

        let combined = MirroringResult {
            total_repos: org_result.total_repos + individual_result.total_repos,
            mirrored: org_result.mirrored + individual_result.mirrored,
            failed: org_result.failed + individual_result.failed,
            errors: [org_result.errors, individual_result.errors].concat(),
        };

        info!(
            "Complete mirroring finished: {}/{} repos mirrored, {} failed",
            combined.mirrored, combined.total_repos, combined.failed
        );

        if !combined.errors.is_empty() {
            info!("Errors encountered:");
            for error in &combined.errors {
                info!("  - {}", error);
            }
        }

        Ok(combined)
    }

    /// Register webhooks for mirrored repositories
    async fn register_webhooks_for_repos(
        &self,
        github_repos: &[crate::github::GitHubRepository],
        gitea_repos: &[crate::gitea_api::GiteaRepository],
    ) -> Result<()> {
        if !self.config.webhooks.enabled {
            return Ok(());
        }

        info!(
            "Registering webhooks for {} repositories",
            gitea_repos.len()
        );

        // Determine webhook endpoint URL
        let webhook_url = if let Some(ref url) = self.config.webhooks.endpoint_url {
            url.clone()
        } else {
            // Auto-configure webhook URL using Gitea URL
            format!("{}/api/v1/repos/webhook", self.gitea_client.config.url)
        };

        // Get webhook secret from config or environment
        let webhook_secret = self
            .config
            .webhooks
            .secret
            .clone()
            .or_else(|| env::var("WEBHOOK_SECRET").ok());

        let mut registered = 0;
        let mut failed = 0;

        for github_repo in github_repos {
            // Parse owner from full_name (format: "owner/repo")
            let parts: Vec<&str> = github_repo.full_name.split('/').collect();
            if parts.len() != 2 {
                warn!("Invalid repository full_name: {}", github_repo.full_name);
                failed += 1;
                continue;
            }

            let owner = parts[0];
            let repo = parts[1];

            // Only register webhooks for successfully mirrored repos
            if !gitea_repos.iter().any(|gr| gr.name == github_repo.name) {
                continue;
            }

            match self
                .github_client
                .ensure_webhook(owner, repo, &webhook_url, webhook_secret.clone())
                .await
            {
                Ok(_) => {
                    info!("Webhook registered for {}/{}", owner, repo);
                    registered += 1;
                }
                Err(e) => {
                    warn!("Failed to register webhook for {}/{}: {}", owner, repo, e);
                    failed += 1;
                }
            }
        }

        info!(
            "Webhook registration complete: {} registered, {} failed",
            registered, failed
        );

        Ok(())
    }

    /// Register webhooks for individual mirrored repositories
    async fn register_webhooks_for_individual_repos(
        &self,
        mirrored_repos: &[(&str, &str, crate::gitea_api::GiteaRepository)],
    ) -> Result<()> {
        if !self.config.webhooks.enabled || mirrored_repos.is_empty() {
            return Ok(());
        }

        info!(
            "Registering webhooks for {} individual repositories",
            mirrored_repos.len()
        );

        // Determine webhook endpoint URL
        let webhook_url = if let Some(ref url) = self.config.webhooks.endpoint_url {
            url.clone()
        } else {
            format!("{}/api/v1/repos/webhook", self.gitea_client.config.url)
        };

        // Get webhook secret from config or environment
        let webhook_secret = self
            .config
            .webhooks
            .secret
            .clone()
            .or_else(|| env::var("WEBHOOK_SECRET").ok());

        let mut registered = 0;
        let mut failed = 0;

        for (owner, repo, _gitea_repo) in mirrored_repos {
            match self
                .github_client
                .ensure_webhook(owner, repo, &webhook_url, webhook_secret.clone())
                .await
            {
                Ok(_) => {
                    info!("Webhook registered for {}/{}", owner, repo);
                    registered += 1;
                }
                Err(e) => {
                    warn!("Failed to register webhook for {}/{}: {}", owner, repo, e);
                    failed += 1;
                }
            }
        }

        info!(
            "Individual webhook registration complete: {} registered, {} failed",
            registered, failed
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirroring_result() {
        let result = MirroringResult {
            total_repos: 10,
            mirrored: 8,
            failed: 2,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        assert_eq!(result.total_repos, 10);
        assert_eq!(result.mirrored, 8);
        assert_eq!(result.failed, 2);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_parse_repository_source() {
        let source = "owner/repo";
        let parts: Vec<&str> = source.split('/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "owner");
        assert_eq!(parts[1], "repo");
    }
}
