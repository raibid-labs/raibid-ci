//! Gitea API client for repository mirroring
//!
//! This module provides functionality to interact with the Gitea API
//! for creating and managing repository mirrors.

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::config::GiteaConfig;
use crate::github::GitHubRepository;

/// Gitea API client
#[derive(Clone)]
pub struct GiteaClient {
    client: reqwest::Client,
    pub config: GiteaConfig,
}

/// Gitea repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub mirror: bool,
    pub clone_url: String,
    pub html_url: String,
}

/// Gitea organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaOrganization {
    pub id: u64,
    pub username: String,
    pub full_name: String,
    pub description: Option<String>,
}

/// Request to create a repository mirror
#[derive(Debug, Clone, Serialize)]
struct CreateMirrorRequest {
    clone_addr: String,
    repo_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
    mirror: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_token: Option<String>,
}

/// Request to create an organization
#[derive(Debug, Clone, Serialize)]
struct CreateOrgRequest {
    username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl GiteaClient {
    /// Create a new Gitea API client
    pub fn new(config: GiteaConfig, token: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();

        // Add authorization if token is provided
        if let Some(ref t) = token {
            let auth_value = format!("token {}", t);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value)
                    .context("Failed to create authorization header")?,
            );
        }

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// Get API base URL
    fn api_url(&self) -> String {
        format!("{}/api/v1", self.config.url)
    }

    /// Check if a repository exists
    pub async fn repository_exists(&self, owner: &str, repo: &str) -> Result<bool> {
        let url = format!("{}/repos/{}/{}", self.api_url(), owner, repo);

        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }

    /// Get repository information
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<GiteaRepository> {
        let url = format!("{}/repos/{}/{}", self.api_url(), owner, repo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch repository")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Gitea API error: {} - {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string())
            ));
        }

        let repo: GiteaRepository = response
            .json()
            .await
            .context("Failed to parse repository response")?;

        Ok(repo)
    }

    /// Create a repository mirror
    pub async fn create_mirror(
        &self,
        clone_url: &str,
        repo_name: &str,
        owner: &str,
        description: Option<String>,
        private: bool,
        auth_token: Option<String>,
    ) -> Result<GiteaRepository> {
        info!(
            "Creating mirror for {} as {}/{}",
            clone_url, owner, repo_name
        );

        // Get owner UID (organization or user)
        let uid = self.get_owner_uid(owner).await?;

        let request = CreateMirrorRequest {
            clone_addr: clone_url.to_string(),
            repo_name: repo_name.to_string(),
            description,
            private: Some(private),
            mirror: true,
            uid: Some(uid),
            auth_token,
        };

        let url = format!("{}/repos/migrate", self.api_url());

        debug!("POST {}: {:?}", url, request);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create mirror")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());

            return Err(anyhow!(
                "Failed to create mirror {}/{}: {} - {}",
                owner,
                repo_name,
                status,
                error_text
            ));
        }

        let repo: GiteaRepository = response
            .json()
            .await
            .context("Failed to parse mirror creation response")?;

        info!("Successfully created mirror: {}", repo.full_name);
        Ok(repo)
    }

    /// Sync an existing mirror
    pub async fn sync_mirror(&self, owner: &str, repo: &str) -> Result<()> {
        info!("Syncing mirror {}/{}", owner, repo);

        let url = format!("{}/repos/{}/{}/mirror-sync", self.api_url(), owner, repo);

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context("Failed to sync mirror")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Gitea API error: {} - {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string())
            ));
        }

        info!("Successfully triggered sync for {}/{}", owner, repo);
        Ok(())
    }

    /// Check if an organization exists
    pub async fn organization_exists(&self, org_name: &str) -> Result<bool> {
        let url = format!("{}/orgs/{}", self.api_url(), org_name);

        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }

    /// Create an organization
    pub async fn create_organization(
        &self,
        name: &str,
        description: Option<String>,
    ) -> Result<GiteaOrganization> {
        info!("Creating organization: {}", name);

        let request = CreateOrgRequest {
            username: name.to_string(),
            full_name: Some(name.to_string()),
            description,
        };

        let url = format!("{}/orgs", self.api_url());

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create organization")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Gitea API error: {} - {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string())
            ));
        }

        let org: GiteaOrganization = response
            .json()
            .await
            .context("Failed to parse organization response")?;

        info!("Successfully created organization: {}", org.username);
        Ok(org)
    }

    /// Ensure organization exists (create if needed)
    pub async fn ensure_organization(&self, name: &str) -> Result<()> {
        if self.organization_exists(name).await? {
            debug!("Organization {} already exists", name);
            return Ok(());
        }

        self.create_organization(name, None).await?;
        Ok(())
    }

    /// Get owner UID (user or organization)
    async fn get_owner_uid(&self, owner: &str) -> Result<u64> {
        // Try as organization first
        let org_url = format!("{}/orgs/{}", self.api_url(), owner);
        let org_response = self.client.get(&org_url).send().await?;

        if org_response.status().is_success() {
            let org: GiteaOrganization = org_response
                .json()
                .await
                .context("Failed to parse organization")?;
            return Ok(org.id);
        }

        // Try as user
        let user_url = format!("{}/users/{}", self.api_url(), owner);
        let user_response = self.client.get(&user_url).send().await?;

        if user_response.status().is_success() {
            #[derive(Deserialize)]
            struct User {
                id: u64,
            }
            let user: User = user_response.json().await.context("Failed to parse user")?;
            return Ok(user.id);
        }

        Err(anyhow!("Owner {} not found as organization or user", owner))
    }

    /// Mirror a GitHub repository to Gitea
    pub async fn mirror_github_repository(
        &self,
        github_repo: &GitHubRepository,
        target_org: &str,
        github_token: Option<String>,
    ) -> Result<GiteaRepository> {
        // Ensure target organization exists
        self.ensure_organization(target_org).await?;

        // Check if mirror already exists
        if self
            .repository_exists(target_org, &github_repo.name)
            .await?
        {
            info!(
                "Mirror already exists: {}/{}. Syncing...",
                target_org, github_repo.name
            );
            self.sync_mirror(target_org, &github_repo.name).await?;
            return self.get_repository(target_org, &github_repo.name).await;
        }

        // Create new mirror
        self.create_mirror(
            &github_repo.clone_url,
            &github_repo.name,
            target_org,
            github_repo.description.clone(),
            github_repo.private,
            github_token,
        )
        .await
    }

    /// Mirror multiple GitHub repositories to Gitea
    pub async fn mirror_github_repositories(
        &self,
        github_repos: &[GitHubRepository],
        target_org: &str,
        github_token: Option<String>,
    ) -> Result<Vec<GiteaRepository>> {
        let mut mirrored = Vec::new();

        for repo in github_repos {
            match self
                .mirror_github_repository(repo, target_org, github_token.clone())
                .await
            {
                Ok(gitea_repo) => {
                    mirrored.push(gitea_repo);
                }
                Err(e) => {
                    warn!("Failed to mirror repository {}: {}", repo.name, e);
                    // Continue with other repositories
                }
            }
        }

        info!(
            "Successfully mirrored {}/{} repositories",
            mirrored.len(),
            github_repos.len()
        );

        Ok(mirrored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mirror_request_serialization() {
        let request = CreateMirrorRequest {
            clone_addr: "https://github.com/test/repo.git".to_string(),
            repo_name: "repo".to_string(),
            description: Some("Test repo".to_string()),
            private: Some(true),
            mirror: true,
            uid: Some(123),
            auth_token: Some("token123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("clone_addr"));
        assert!(json.contains("repo_name"));
        assert!(json.contains("mirror"));
    }

    #[test]
    fn test_api_url() {
        let config = GiteaConfig {
            url: "http://gitea.local:3000".to_string(),
            admin_user: "admin".to_string(),
            admin_password: None,
            registry_enabled: true,
            registry_port: 5000,
        };

        let client = GiteaClient::new(config, None).unwrap();
        assert_eq!(client.api_url(), "http://gitea.local:3000/api/v1");
    }
}
