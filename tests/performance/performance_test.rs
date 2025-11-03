//! Performance and Load Testing
//!
//! This test suite validates system behavior under load:
//! - Burst load (many jobs at once)
//! - Sustained load (constant job rate)
//! - Ramp-up load (gradually increasing)
//! - Resource usage monitoring
//! - Autoscaling responsiveness

use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

const SERVER_URL: &str = "http://localhost:8080";
const REDIS_URL: &str = "redis://localhost:6379";

#[derive(Debug, Clone)]
struct LoadTestConfig {
    server_url: String,
    redis_url: String,
    concurrent_jobs: usize,
    total_jobs: usize,
    ramp_duration_secs: u64,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            server_url: std::env::var("TEST_SERVER_URL")
                .unwrap_or_else(|_| SERVER_URL.to_string()),
            redis_url: std::env::var("TEST_REDIS_URL").unwrap_or_else(|_| REDIS_URL.to_string()),
            concurrent_jobs: 10,
            total_jobs: 50,
            ramp_duration_secs: 30,
        }
    }
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    min_response_time: Duration,
    max_response_time: Duration,
    avg_response_time: Duration,
    p50_response_time: Duration,
    p95_response_time: Duration,
    p99_response_time: Duration,
    requests_per_second: f64,
    total_duration: Duration,
}

/// Burst load test - submit many jobs simultaneously
#[tokio::test]
#[ignore]
async fn test_burst_load() {
    let config = LoadTestConfig {
        concurrent_jobs: 20,
        total_jobs: 100,
        ..Default::default()
    };

    println!("\n=== Burst Load Test ===");
    println!("Concurrent jobs: {}", config.concurrent_jobs);
    println!("Total jobs: {}", config.total_jobs);

    let metrics = run_burst_load(&config).await;
    print_metrics(&metrics);

    // Performance targets
    assert!(
        metrics.avg_response_time < Duration::from_millis(500),
        "Average response time should be < 500ms"
    );
    assert!(
        metrics.p95_response_time < Duration::from_secs(2),
        "P95 response time should be < 2s"
    );
    assert!(
        metrics.successful_requests as f64 / metrics.total_requests as f64 > 0.95,
        "Success rate should be > 95%"
    );
}

/// Sustained load test - constant job submission rate
#[tokio::test]
#[ignore]
async fn test_sustained_load() {
    let config = LoadTestConfig {
        concurrent_jobs: 5,
        total_jobs: 100,
        ..Default::default()
    };

    println!("\n=== Sustained Load Test ===");
    println!("Job rate: {} concurrent", config.concurrent_jobs);
    println!("Duration: {} jobs", config.total_jobs);

    let metrics = run_sustained_load(&config).await;
    print_metrics(&metrics);

    // Performance targets
    assert!(
        metrics.requests_per_second > 5.0,
        "Should handle > 5 requests/second"
    );
    assert!(
        metrics.successful_requests as f64 / metrics.total_requests as f64 > 0.98,
        "Success rate should be > 98%"
    );
}

/// Ramp-up load test - gradually increase load
#[tokio::test]
#[ignore]
async fn test_ramp_up_load() {
    let config = LoadTestConfig {
        total_jobs: 60,
        ramp_duration_secs: 30,
        ..Default::default()
    };

    println!("\n=== Ramp-Up Load Test ===");
    println!("Total jobs: {}", config.total_jobs);
    println!("Ramp duration: {}s", config.ramp_duration_secs);

    let metrics = run_ramp_up_load(&config).await;
    print_metrics(&metrics);

    // Performance targets
    assert!(
        metrics.successful_requests as f64 / metrics.total_requests as f64 > 0.95,
        "Success rate should be > 95%"
    );
}

/// Resource usage monitoring test
#[tokio::test]
#[ignore]
async fn test_resource_usage() {
    println!("\n=== Resource Usage Test ===");

    // Get baseline metrics
    let baseline = collect_resource_metrics().await;
    println!("Baseline metrics: {:?}", baseline);

    // Run load test
    let config = LoadTestConfig {
        concurrent_jobs: 10,
        total_jobs: 50,
        ..Default::default()
    };

    let _metrics = run_burst_load(&config).await;

    // Collect metrics under load
    let under_load = collect_resource_metrics().await;
    println!("Under load metrics: {:?}", under_load);

    // Wait for cooldown
    sleep(Duration::from_secs(30)).await;

    // Collect post-load metrics
    let after_load = collect_resource_metrics().await;
    println!("After load metrics: {:?}", after_load);

    // Verify resource usage is reasonable
    if let (Some(baseline_mem), Some(load_mem)) =
        (baseline.get("memory_mb"), under_load.get("memory_mb"))
    {
        let memory_increase = load_mem - baseline_mem;
        println!("Memory increase: {}MB", memory_increase);
        assert!(
            memory_increase < 1024.0,
            "Memory increase should be < 1GB"
        );
    }
}

/// KEDA autoscaling responsiveness test
#[tokio::test]
#[ignore]
async fn test_autoscaling_responsiveness() {
    println!("\n=== Autoscaling Responsiveness Test ===");

    let config = LoadTestConfig {
        concurrent_jobs: 20,
        total_jobs: 100,
        ..Default::default()
    };

    // Measure time to first agent spawn
    let start = Instant::now();

    // Trigger jobs
    let client = Client::new();
    for i in 0..config.concurrent_jobs {
        let _ = trigger_job(&client, &config, i).await;
    }

    // Wait for first agent
    let agent_spawn_time = wait_for_agent_count(1, Duration::from_secs(60)).await;
    println!("Time to first agent: {:?}", agent_spawn_time);

    // Measure time to scale to max agents
    let max_agent_time = wait_for_agent_count(5, Duration::from_secs(120)).await;
    println!("Time to 5 agents: {:?}", max_agent_time);

    // Performance targets
    assert!(
        agent_spawn_time < Duration::from_secs(30),
        "First agent should spawn within 30s"
    );
    assert!(
        max_agent_time < Duration::from_secs(90),
        "Should scale to 5 agents within 90s"
    );
}

/// Run burst load test
async fn run_burst_load(config: &LoadTestConfig) -> PerformanceMetrics {
    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(config.concurrent_jobs));
    let mut handles = Vec::new();
    let start = Instant::now();

    for i in 0..config.total_jobs {
        let client = client.clone();
        let config = config.clone();
        let semaphore = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let req_start = Instant::now();
            let result = trigger_job(&client, &config, i).await;
            let duration = req_start.elapsed();
            (result.is_ok(), duration)
        });

        handles.push(handle);
    }

    let results: Vec<(bool, Duration)> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    calculate_metrics(results, start.elapsed())
}

/// Run sustained load test
async fn run_sustained_load(config: &LoadTestConfig) -> PerformanceMetrics {
    let client = Client::new();
    let mut results = Vec::new();
    let start = Instant::now();

    let delay_between_batches =
        Duration::from_millis(1000 / config.concurrent_jobs.max(1) as u64);

    for i in 0..config.total_jobs {
        let req_start = Instant::now();
        let result = trigger_job(&client, config, i).await;
        let duration = req_start.elapsed();
        results.push((result.is_ok(), duration));

        if (i + 1) % config.concurrent_jobs == 0 {
            sleep(delay_between_batches).await;
        }
    }

    calculate_metrics(results, start.elapsed())
}

/// Run ramp-up load test
async fn run_ramp_up_load(config: &LoadTestConfig) -> PerformanceMetrics {
    let client = Client::new();
    let mut results = Vec::new();
    let start = Instant::now();

    let delay_per_job =
        Duration::from_millis((config.ramp_duration_secs * 1000) / config.total_jobs as u64);

    for i in 0..config.total_jobs {
        let req_start = Instant::now();
        let result = trigger_job(&client, config, i).await;
        let duration = req_start.elapsed();
        results.push((result.is_ok(), duration));

        sleep(delay_per_job).await;
    }

    calculate_metrics(results, start.elapsed())
}

/// Trigger a single job
async fn trigger_job(client: &Client, config: &LoadTestConfig, job_num: usize) -> Result<(), ()> {
    let payload = json!({
        "ref": "refs/heads/main",
        "after": format!("perf-test-{}-{}", Utc::now().timestamp(), job_num),
        "repository": {
            "full_name": "raibid-ci/perf-test",
            "clone_url": "http://gitea:3000/raibid-ci/perf-test.git"
        },
        "pusher": {
            "username": "perf-test",
            "name": "Performance Test"
        }
    });

    client
        .post(format!("{}/webhooks/gitea", config.server_url))
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| ())?
        .error_for_status()
        .map(|_| ())
        .map_err(|_| ())
}

/// Calculate performance metrics
fn calculate_metrics(
    results: Vec<(bool, Duration)>,
    total_duration: Duration,
) -> PerformanceMetrics {
    let total_requests = results.len();
    let successful_requests = results.iter().filter(|(success, _)| *success).count();
    let failed_requests = total_requests - successful_requests;

    let mut response_times: Vec<Duration> = results.iter().map(|(_, d)| *d).collect();
    response_times.sort();

    let min_response_time = response_times.first().copied().unwrap_or_default();
    let max_response_time = response_times.last().copied().unwrap_or_default();

    let total_response_time: Duration = response_times.iter().sum();
    let avg_response_time = total_response_time / total_requests.max(1) as u32;

    let p50_idx = (total_requests as f64 * 0.50) as usize;
    let p95_idx = (total_requests as f64 * 0.95) as usize;
    let p99_idx = (total_requests as f64 * 0.99) as usize;

    let p50_response_time = response_times.get(p50_idx).copied().unwrap_or_default();
    let p95_response_time = response_times.get(p95_idx).copied().unwrap_or_default();
    let p99_response_time = response_times.get(p99_idx).copied().unwrap_or_default();

    let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();

    PerformanceMetrics {
        total_requests,
        successful_requests,
        failed_requests,
        min_response_time,
        max_response_time,
        avg_response_time,
        p50_response_time,
        p95_response_time,
        p99_response_time,
        requests_per_second,
        total_duration,
    }
}

/// Print performance metrics
fn print_metrics(metrics: &PerformanceMetrics) {
    println!("\n--- Performance Metrics ---");
    println!("Total requests: {}", metrics.total_requests);
    println!(
        "Successful: {} ({:.1}%)",
        metrics.successful_requests,
        (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0
    );
    println!(
        "Failed: {} ({:.1}%)",
        metrics.failed_requests,
        (metrics.failed_requests as f64 / metrics.total_requests as f64) * 100.0
    );
    println!("Min response time: {:?}", metrics.min_response_time);
    println!("Max response time: {:?}", metrics.max_response_time);
    println!("Avg response time: {:?}", metrics.avg_response_time);
    println!("P50 response time: {:?}", metrics.p50_response_time);
    println!("P95 response time: {:?}", metrics.p95_response_time);
    println!("P99 response time: {:?}", metrics.p99_response_time);
    println!("Requests/sec: {:.2}", metrics.requests_per_second);
    println!("Total duration: {:?}", metrics.total_duration);
    println!("---------------------------\n");
}

/// Collect resource metrics from Kubernetes
async fn collect_resource_metrics() -> HashMap<String, f64> {
    let mut metrics = HashMap::new();

    // Get pod metrics using kubectl top
    if let Ok(output) = tokio::process::Command::new("kubectl")
        .args(&["top", "pods", "-n", "raibid-ci", "--no-headers"])
        .output()
        .await
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut total_cpu = 0.0;
            let mut total_memory = 0.0;

            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    // Parse CPU (e.g., "100m" -> 0.1 cores)
                    if let Some(cpu) = parts[1].strip_suffix('m') {
                        if let Ok(cpu_val) = cpu.parse::<f64>() {
                            total_cpu += cpu_val / 1000.0;
                        }
                    }

                    // Parse memory (e.g., "256Mi" -> 256 MB)
                    if let Some(mem) = parts[2].strip_suffix("Mi") {
                        if let Ok(mem_val) = mem.parse::<f64>() {
                            total_memory += mem_val;
                        }
                    }
                }
            }

            metrics.insert("cpu_cores".to_string(), total_cpu);
            metrics.insert("memory_mb".to_string(), total_memory);
        }
    }

    // Get Redis memory usage
    if let Ok(redis_client) = redis::Client::open(REDIS_URL) {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            if let Ok(info) = redis::cmd("INFO")
                .arg("memory")
                .query_async::<_, String>(&mut conn)
                .await
            {
                for line in info.lines() {
                    if let Some(mem_str) = line.strip_prefix("used_memory:") {
                        if let Ok(mem_bytes) = mem_str.parse::<f64>() {
                            metrics.insert("redis_memory_mb".to_string(), mem_bytes / 1024.0 / 1024.0);
                        }
                    }
                }
            }
        }
    }

    metrics
}

/// Wait for specific number of agents to be running
async fn wait_for_agent_count(target_count: usize, timeout: Duration) -> Duration {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            break;
        }

        if let Ok(output) = tokio::process::Command::new("kubectl")
            .args(&[
                "get",
                "pods",
                "-n",
                "raibid-ci",
                "-l",
                "app=raibid-agent",
                "--field-selector=status.phase=Running",
                "-o",
                "json",
            ])
            .output()
            .await
        {
            if output.status.success() {
                if let Ok(pods) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(items) = pods.get("items").and_then(|v| v.as_array()) {
                        if items.len() >= target_count {
                            return start.elapsed();
                        }
                    }
                }
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    start.elapsed()
}
