//! API client for fetching real-time data from the server
//!
//! This module provides an async HTTP client for communicating with the
//! raibid-ci API server to fetch jobs, agents, queue metrics, and perform actions.

use anyhow::{Context, Result};
use raibid_common::{Job, JobStatus};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// API client configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Base URL of the API server
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("RAIBID_API_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            timeout: Duration::from_secs(5),
        }
    }
}

/// API client for raibid-ci server
#[derive(Clone)]
pub struct ApiClient {
    config: ApiConfig,
    client: reqwest::Client,
}

impl ApiClient {
    /// Create a new API client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ApiConfig::default())
    }

    /// Create a new API client with custom configuration
    pub fn with_config(config: ApiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// List jobs with optional filters
    pub async fn list_jobs(
        &self,
        status: Option<JobStatus>,
        repo: Option<String>,
        branch: Option<String>,
        limit: Option<usize>,
    ) -> Result<JobListResponse> {
        let mut url = format!("{}/jobs", self.config.base_url);
        let mut params = Vec::new();

        if let Some(s) = status {
            params.push(format!("status={}", s.as_str().to_lowercase()));
        }
        if let Some(r) = repo {
            params.push(format!("repo={}", urlencoding::encode(&r)));
        }
        if let Some(b) = branch {
            params.push(format!("branch={}", urlencoding::encode(&b)));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch jobs")?;

        if !response.status().is_success() {
            anyhow::bail!("API returned error: {}", response.status());
        }

        response
            .json::<JobListResponse>()
            .await
            .context("Failed to parse jobs response")
    }

    /// Get a specific job by ID
    pub async fn get_job(&self, id: &str) -> Result<Job> {
        let url = format!("{}/jobs/{}", self.config.base_url, id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch job")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Job not found: {}", id);
        }

        if !response.status().is_success() {
            anyhow::bail!("API returned error: {}", response.status());
        }

        response
            .json::<Job>()
            .await
            .context("Failed to parse job response")
    }

    /// Cancel a job
    pub async fn cancel_job(&self, id: &str) -> Result<()> {
        let url = format!("{}/jobs/{}/cancel", self.config.base_url, id);

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context("Failed to cancel job")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to cancel job: {}", response.status());
        }

        Ok(())
    }

    /// Trigger a new job
    pub async fn trigger_job(&self, repo: String, branch: String) -> Result<Job> {
        let url = format!("{}/jobs", self.config.base_url);

        let trigger = JobTrigger {
            repo,
            branch,
            commit: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&trigger)
            .send()
            .await
            .context("Failed to trigger job")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to trigger job: {}", response.status());
        }

        response
            .json::<Job>()
            .await
            .context("Failed to parse job response")
    }

    /// Get agent list (placeholder - actual endpoint would need to be implemented)
    pub async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let url = format!("{}/agents", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch agents")?;

        if !response.status().is_success() {
            // Return empty list if endpoint doesn't exist yet
            return Ok(Vec::new());
        }

        response
            .json::<Vec<AgentInfo>>()
            .await
            .context("Failed to parse agents response")
    }

    /// Get queue metrics (placeholder - actual endpoint would need to be implemented)
    pub async fn get_queue_metrics(&self) -> Result<QueueMetrics> {
        let url = format!("{}/metrics/queue", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch queue metrics")?;

        if !response.status().is_success() {
            // Return default metrics if endpoint doesn't exist yet
            return Ok(QueueMetrics::default());
        }

        response
            .json::<QueueMetrics>()
            .await
            .context("Failed to parse queue metrics response")
    }

    /// Check server health
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let url = format!("{}/health", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to check server health")?;

        if !response.status().is_success() {
            return Ok(HealthStatus {
                healthy: false,
                message: Some("Server returned error".to_string()),
            });
        }

        response
            .json::<HealthStatus>()
            .await
            .context("Failed to parse health response")
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default API client")
    }
}

/// Job list response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListResponse {
    pub jobs: Vec<Job>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Job trigger request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTrigger {
    pub repo: String,
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

/// Agent information from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub cpu: u8,
    pub memory: u8,
    pub uptime: u64,
}

/// Queue metrics from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub current_depth: u64,
    pub max_depth: u64,
    pub avg_depth: f64,
    #[serde(default)]
    pub history: Vec<u64>,
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self {
            current_depth: 0,
            max_depth: 0,
            avg_depth: 0.0,
            history: vec![0; 60],
        }
    }
}

/// Health status from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert!(config.base_url.starts_with("http"));
        assert_eq!(config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_api_client_creation() {
        let config = ApiConfig {
            base_url: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(10),
        };
        let client = ApiClient::with_config(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_queue_metrics_default() {
        let metrics = QueueMetrics::default();
        assert_eq!(metrics.current_depth, 0);
        assert_eq!(metrics.history.len(), 60);
    }
}
