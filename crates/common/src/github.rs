//! GitHub API client for repository mirroring
//!
//! This module provides functionality to interact with the GitHub API
//! for fetching organization repositories and managing webhooks.

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::config::{GitHubConfig, OrganizationMirrorConfig};

/// GitHub API client
#[derive(Clone)]
pub struct GitHubClient {
    client: reqwest::Client,
    config: GitHubConfig,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub clone_url: String,
    pub ssh_url: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub archived: bool,
}

/// GitHub API rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub limit: u32,
    pub remaining: u32,
    pub reset: i64,
}

/// GitHub webhook information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhook {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
    pub config: WebhookConfig,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_ssl: Option<String>,
}

/// Request to create a webhook
#[derive(Debug, Clone, Serialize)]
struct CreateWebhookRequest {
    name: String,
    active: bool,
    events: Vec<String>,
    config: WebhookConfig,
}

impl GitHubClient {
    /// Create a new GitHub API client
    pub fn new(config: GitHubConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("raibid-ci"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        // Add authorization if token is provided
        if let Some(ref token) = config.token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value)
                    .context("Failed to create authorization header")?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// Get rate limit information
    pub async fn get_rate_limit(&self) -> Result<RateLimit> {
        let url = format!("{}/rate_limit", self.config.api_url);

        #[derive(Deserialize)]
        struct RateLimitResponse {
            rate: RateLimit,
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch rate limit")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub API error: {} - {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string())
            ));
        }

        let data: RateLimitResponse = response
            .json()
            .await
            .context("Failed to parse rate limit response")?;

        Ok(data.rate)
    }

    /// Check if we're approaching rate limit threshold
    pub async fn check_rate_limit(&self) -> Result<bool> {
        let rate_limit = self.get_rate_limit().await?;

        debug!(
            "GitHub API rate limit: {}/{} remaining",
            rate_limit.remaining, rate_limit.limit
        );

        if rate_limit.remaining < self.config.rate_limit_threshold {
            warn!(
                "GitHub API rate limit low: {} requests remaining",
                rate_limit.remaining
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// List repositories for an organization
    pub async fn list_org_repositories(&self, org: &str) -> Result<Vec<GitHubRepository>> {
        let mut all_repos = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "{}/orgs/{}/repos?page={}&per_page={}",
                self.config.api_url, org, page, per_page
            );

            debug!("Fetching page {} of repositories for org: {}", page, org);

            let response = self
                .client
                .get(&url)
                .send()
                .await
                .context("Failed to fetch organization repositories")?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "GitHub API error: {} - {}",
                    response.status(),
                    response
                        .text()
                        .await
                        .unwrap_or_else(|_| "unknown error".to_string())
                ));
            }

            let repos: Vec<GitHubRepository> = response
                .json()
                .await
                .context("Failed to parse repositories response")?;

            if repos.is_empty() {
                break;
            }

            all_repos.extend(repos);
            page += 1;
        }

        info!(
            "Fetched {} repositories from organization: {}",
            all_repos.len(),
            org
        );

        Ok(all_repos)
    }

    /// Filter repositories based on organization mirror config
    pub fn filter_repositories(
        &self,
        repos: Vec<GitHubRepository>,
        config: &OrganizationMirrorConfig,
    ) -> Result<Vec<GitHubRepository>> {
        let include_regex =
            Regex::new(&config.include_pattern).context("Invalid include pattern regex")?;

        let exclude_regex = if let Some(ref pattern) = config.exclude_pattern {
            Some(Regex::new(pattern).context("Invalid exclude pattern regex")?)
        } else {
            None
        };

        let total_count = repos.len();
        let filtered: Vec<GitHubRepository> = repos
            .into_iter()
            .filter(|repo| {
                // Skip archived repositories
                if repo.archived {
                    debug!("Skipping archived repository: {}", repo.name);
                    return false;
                }

                // Apply private/public filtering
                if config.private_only && !repo.private {
                    debug!(
                        "Skipping public repository (private_only=true): {}",
                        repo.name
                    );
                    return false;
                }

                if config.public_only && repo.private {
                    debug!(
                        "Skipping private repository (public_only=true): {}",
                        repo.name
                    );
                    return false;
                }

                // Apply include pattern
                if !include_regex.is_match(&repo.name) {
                    debug!(
                        "Repository {} does not match include pattern: {}",
                        repo.name, config.include_pattern
                    );
                    return false;
                }

                // Apply exclude pattern
                if let Some(ref regex) = exclude_regex {
                    if regex.is_match(&repo.name) {
                        debug!(
                            "Repository {} matches exclude pattern: {:?}",
                            repo.name, config.exclude_pattern
                        );
                        return false;
                    }
                }

                true
            })
            .collect();

        info!(
            "Filtered {} repositories from {} total (org: {})",
            filtered.len(),
            total_count,
            config.name
        );

        Ok(filtered)
    }

    /// Get all repositories for an organization with filtering
    pub async fn get_org_repositories(
        &self,
        config: &OrganizationMirrorConfig,
    ) -> Result<Vec<GitHubRepository>> {
        // Check rate limit before making requests
        if !self.check_rate_limit().await? {
            return Err(anyhow!(
                "GitHub API rate limit threshold reached. Please wait before retrying."
            ));
        }

        let all_repos = self.list_org_repositories(&config.name).await?;
        let filtered_repos = self.filter_repositories(all_repos, config)?;

        Ok(filtered_repos)
    }

    /// List webhooks for a repository
    pub async fn list_webhooks(&self, owner: &str, repo: &str) -> Result<Vec<GitHubWebhook>> {
        let url = format!("{}/repos/{}/{}/hooks", self.config.api_url, owner, repo);

        debug!("Fetching webhooks for {}/{}", owner, repo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch webhooks")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub API error: {} - {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string())
            ));
        }

        let webhooks: Vec<GitHubWebhook> = response
            .json()
            .await
            .context("Failed to parse webhooks response")?;

        debug!("Found {} webhooks for {}/{}", webhooks.len(), owner, repo);
        Ok(webhooks)
    }

    /// Create a webhook for a repository
    pub async fn create_webhook(
        &self,
        owner: &str,
        repo: &str,
        webhook_url: &str,
        secret: Option<String>,
    ) -> Result<GitHubWebhook> {
        info!("Creating webhook for {}/{} -> {}", owner, repo, webhook_url);

        let request = CreateWebhookRequest {
            name: "web".to_string(),
            active: true,
            events: vec!["push".to_string(), "pull_request".to_string()],
            config: WebhookConfig {
                url: webhook_url.to_string(),
                content_type: "json".to_string(),
                secret,
                insecure_ssl: Some("0".to_string()),
            },
        };

        let url = format!("{}/repos/{}/{}/hooks", self.config.api_url, owner, repo);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create webhook")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());

            return Err(anyhow!(
                "Failed to create webhook for {}/{}: {} - {}",
                owner,
                repo,
                status,
                error_text
            ));
        }

        let webhook: GitHubWebhook = response
            .json()
            .await
            .context("Failed to parse webhook creation response")?;

        info!(
            "Successfully created webhook {} for {}/{}",
            webhook.id, owner, repo
        );
        Ok(webhook)
    }

    /// Check if a webhook already exists for a given URL
    pub async fn webhook_exists(&self, owner: &str, repo: &str, webhook_url: &str) -> Result<bool> {
        let webhooks = self.list_webhooks(owner, repo).await?;

        Ok(webhooks.iter().any(|w| w.config.url == webhook_url))
    }

    /// Ensure a webhook exists (create if needed)
    pub async fn ensure_webhook(
        &self,
        owner: &str,
        repo: &str,
        webhook_url: &str,
        secret: Option<String>,
    ) -> Result<()> {
        if self.webhook_exists(owner, repo, webhook_url).await? {
            debug!("Webhook already exists for {}/{}", owner, repo);
            return Ok(());
        }

        self.create_webhook(owner, repo, webhook_url, secret)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_repositories_private_only() {
        let config = GitHubConfig {
            api_url: "https://api.github.com".to_string(),
            token: None,
            rate_limit_threshold: 100,
        };

        let client = GitHubClient::new(config).unwrap();

        let repos = vec![
            GitHubRepository {
                name: "public-repo".to_string(),
                full_name: "org/public-repo".to_string(),
                private: false,
                clone_url: "https://github.com/org/public-repo.git".to_string(),
                ssh_url: "git@github.com:org/public-repo.git".to_string(),
                description: None,
                default_branch: "main".to_string(),
                archived: false,
            },
            GitHubRepository {
                name: "private-repo".to_string(),
                full_name: "org/private-repo".to_string(),
                private: true,
                clone_url: "https://github.com/org/private-repo.git".to_string(),
                ssh_url: "git@github.com:org/private-repo.git".to_string(),
                description: None,
                default_branch: "main".to_string(),
                archived: false,
            },
        ];

        let org_config = OrganizationMirrorConfig {
            name: "org".to_string(),
            private_only: true,
            public_only: false,
            include_pattern: ".*".to_string(),
            exclude_pattern: None,
            target_organization: None,
        };

        let filtered = client.filter_repositories(repos, &org_config).unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "private-repo");
    }

    #[test]
    fn test_filter_repositories_with_regex() {
        let config = GitHubConfig {
            api_url: "https://api.github.com".to_string(),
            token: None,
            rate_limit_threshold: 100,
        };

        let client = GitHubClient::new(config).unwrap();

        let repos = vec![
            GitHubRepository {
                name: "backend-api".to_string(),
                full_name: "org/backend-api".to_string(),
                private: false,
                clone_url: "https://github.com/org/backend-api.git".to_string(),
                ssh_url: "git@github.com:org/backend-api.git".to_string(),
                description: None,
                default_branch: "main".to_string(),
                archived: false,
            },
            GitHubRepository {
                name: "frontend-web".to_string(),
                full_name: "org/frontend-web".to_string(),
                private: false,
                clone_url: "https://github.com/org/frontend-web.git".to_string(),
                ssh_url: "git@github.com:org/frontend-web.git".to_string(),
                description: None,
                default_branch: "main".to_string(),
                archived: false,
            },
            GitHubRepository {
                name: "docs-site".to_string(),
                full_name: "org/docs-site".to_string(),
                private: false,
                clone_url: "https://github.com/org/docs-site.git".to_string(),
                ssh_url: "git@github.com:org/docs-site.git".to_string(),
                description: None,
                default_branch: "main".to_string(),
                archived: false,
            },
        ];

        let org_config = OrganizationMirrorConfig {
            name: "org".to_string(),
            private_only: false,
            public_only: false,
            include_pattern: "^(backend|frontend)-.*".to_string(),
            exclude_pattern: None,
            target_organization: None,
        };

        let filtered = client.filter_repositories(repos, &org_config).unwrap();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|r| r.name == "backend-api"));
        assert!(filtered.iter().any(|r| r.name == "frontend-web"));
        assert!(!filtered.iter().any(|r| r.name == "docs-site"));
    }
}
