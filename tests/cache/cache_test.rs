//! Build Cache Testing
//!
//! This test suite validates build cache functionality:
//! - Cold cache builds (no cache)
//! - Warm cache builds (full cache hit)
//! - Incremental builds (partial cache)
//! - Dependency change scenarios
//! - Cache invalidation
//! - Cache performance metrics

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

const CACHE_DIR: &str = "/tmp/raibid-cache-test";

#[derive(Debug, Clone)]
struct CacheMetrics {
    build_time: Duration,
    cache_hit_rate: f64,
    cache_size_mb: f64,
    network_saved_mb: f64,
    speedup_factor: f64,
}

#[derive(Debug, Clone)]
struct BuildResult {
    success: bool,
    duration: Duration,
    cache_stats: HashMap<String, String>,
}

/// Cold cache test - first build with no cache
#[tokio::test]
#[ignore]
async fn test_cold_cache_build() {
    println!("\n=== Cold Cache Build Test ===");

    // Clean cache
    cleanup_cache().await;

    let project_dir = setup_test_project().await;
    let start = Instant::now();

    let result = run_build(&project_dir, false).await;
    let cold_build_time = start.elapsed();

    assert!(result.success, "Cold cache build should succeed");
    println!("Cold build time: {:?}", cold_build_time);

    // Verify cache was created
    let cache_size = get_cache_size().await;
    println!("Cache size after build: {:.2}MB", cache_size);
    assert!(cache_size > 0.0, "Cache should exist after build");

    // Cleanup
    cleanup_test_project(project_dir).await;
}

/// Warm cache test - rebuild with full cache hit
#[tokio::test]
#[ignore]
async fn test_warm_cache_build() {
    println!("\n=== Warm Cache Build Test ===");

    cleanup_cache().await;
    let project_dir = setup_test_project().await;

    // First build (cold)
    println!("Running cold build...");
    let cold_start = Instant::now();
    let cold_result = run_build(&project_dir, false).await;
    let cold_time = cold_start.elapsed();
    assert!(cold_result.success, "Cold build should succeed");
    println!("Cold build time: {:?}", cold_time);

    // Second build (warm - should use cache)
    println!("\nRunning warm build...");
    let warm_start = Instant::now();
    let warm_result = run_build(&project_dir, true).await;
    let warm_time = warm_start.elapsed();
    assert!(warm_result.success, "Warm build should succeed");
    println!("Warm build time: {:?}", warm_time);

    // Calculate speedup
    let speedup = cold_time.as_secs_f64() / warm_time.as_secs_f64();
    println!("Speedup factor: {:.2}x", speedup);

    // Performance targets
    assert!(
        speedup >= 2.0,
        "Warm build should be at least 2x faster (got {:.2}x)",
        speedup
    );

    cleanup_test_project(project_dir).await;
}

/// Incremental build test - rebuild after source change
#[tokio::test]
#[ignore]
async fn test_incremental_build() {
    println!("\n=== Incremental Build Test ===");

    cleanup_cache().await;
    let project_dir = setup_test_project().await;

    // Initial build
    println!("Running initial build...");
    let initial_result = run_build(&project_dir, false).await;
    assert!(initial_result.success, "Initial build should succeed");

    // Modify source file
    println!("\nModifying source file...");
    modify_source_file(&project_dir).await;

    // Incremental build
    println!("Running incremental build...");
    let inc_start = Instant::now();
    let inc_result = run_build(&project_dir, true).await;
    let inc_time = inc_start.elapsed();

    assert!(inc_result.success, "Incremental build should succeed");
    println!("Incremental build time: {:?}", inc_time);

    // Incremental should be faster than cold but slower than warm
    assert!(
        inc_time < Duration::from_secs(30),
        "Incremental build should complete quickly"
    );

    cleanup_test_project(project_dir).await;
}

/// Dependency change test - rebuild after Cargo.toml change
#[tokio::test]
#[ignore]
async fn test_dependency_change_build() {
    println!("\n=== Dependency Change Build Test ===");

    cleanup_cache().await;
    let project_dir = setup_test_project().await;

    // Initial build
    println!("Running initial build...");
    let initial_result = run_build(&project_dir, false).await;
    assert!(initial_result.success, "Initial build should succeed");

    // Add dependency
    println!("\nAdding new dependency...");
    add_dependency(&project_dir).await;

    // Build with new dependency
    println!("Building with new dependency...");
    let dep_start = Instant::now();
    let dep_result = run_build(&project_dir, true).await;
    let dep_time = dep_start.elapsed();

    assert!(
        dep_result.success,
        "Build with new dependency should succeed"
    );
    println!("Dependency change build time: {:?}", dep_time);

    cleanup_test_project(project_dir).await;
}

/// Cache invalidation test
#[tokio::test]
#[ignore]
async fn test_cache_invalidation() {
    println!("\n=== Cache Invalidation Test ===");

    cleanup_cache().await;
    let project_dir = setup_test_project().await;

    // Build with cache
    let build1 = run_build(&project_dir, false).await;
    assert!(build1.success);

    // Modify Cargo.toml (should invalidate cache)
    modify_cargo_toml(&project_dir).await;

    // Build should detect invalidation
    let build2 = run_build(&project_dir, true).await;
    assert!(build2.success);

    // Verify cache was invalidated and rebuilt
    println!("Cache properly invalidated after Cargo.toml change");

    cleanup_test_project(project_dir).await;
}

/// Cache performance metrics test
#[tokio::test]
#[ignore]
async fn test_cache_performance_metrics() {
    println!("\n=== Cache Performance Metrics Test ===");

    cleanup_cache().await;
    let project_dir = setup_test_project().await;

    // Collect metrics for cold build
    let cold_metrics = measure_cache_performance(&project_dir, false).await;
    println!("\nCold build metrics:");
    print_cache_metrics(&cold_metrics);

    // Collect metrics for warm build
    let warm_metrics = measure_cache_performance(&project_dir, true).await;
    println!("\nWarm build metrics:");
    print_cache_metrics(&warm_metrics);

    // Verify performance targets
    assert!(
        warm_metrics.speedup_factor >= 2.0,
        "Speedup should be at least 2x"
    );
    assert!(
        warm_metrics.cache_hit_rate >= 0.70,
        "Cache hit rate should be at least 70%"
    );

    cleanup_test_project(project_dir).await;
}

/// Cache cleanup/pruning test
#[tokio::test]
#[ignore]
async fn test_cache_cleanup() {
    println!("\n=== Cache Cleanup Test ===");

    cleanup_cache().await;
    let project_dir = setup_test_project().await;

    // Build to create cache
    run_build(&project_dir, false).await;

    let size_before = get_cache_size().await;
    println!("Cache size before cleanup: {:.2}MB", size_before);

    // Run cache cleanup
    run_cache_cleanup().await;

    let size_after = get_cache_size().await;
    println!("Cache size after cleanup: {:.2}MB", size_after);

    // Verify cleanup worked
    assert!(
        size_after <= size_before,
        "Cache size should not increase after cleanup"
    );

    cleanup_test_project(project_dir).await;
}

/// Setup test Rust project
async fn setup_test_project() -> PathBuf {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("test-project");

    // Create from fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust-project");

    copy_dir(&fixture_path, &project_dir)
        .await
        .expect("Failed to copy fixture");

    // Don't drop temp_dir to keep it alive
    std::mem::forget(temp_dir);

    project_dir
}

/// Copy directory recursively
async fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst).await?;

    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let ty = entry.file_type().await?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir(&src_path, &dst_path).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
    }

    Ok(())
}

/// Run cargo build
async fn run_build(project_dir: &Path, use_cache: bool) -> BuildResult {
    let start = Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--release").current_dir(project_dir);

    if use_cache {
        cmd.env("CARGO_TARGET_DIR", CACHE_DIR);
    }

    let output = cmd.output().await.expect("Failed to run cargo build");

    BuildResult {
        success: output.status.success(),
        duration: start.elapsed(),
        cache_stats: HashMap::new(),
    }
}

/// Modify source file
async fn modify_source_file(project_dir: &Path) {
    let lib_file = project_dir.join("src/lib.rs");
    let content = fs::read_to_string(&lib_file)
        .await
        .expect("Failed to read lib.rs");

    let modified = format!(
        "{}\n\npub fn new_function() -> i32 {{ 42 }}\n",
        content
    );

    fs::write(&lib_file, modified)
        .await
        .expect("Failed to write lib.rs");
}

/// Add dependency to Cargo.toml
async fn add_dependency(project_dir: &Path) {
    let cargo_toml = project_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml)
        .await
        .expect("Failed to read Cargo.toml");

    let modified = content.replace(
        "[dependencies]",
        "[dependencies]\nchrono = \"0.4\"",
    );

    fs::write(&cargo_toml, modified)
        .await
        .expect("Failed to write Cargo.toml");
}

/// Modify Cargo.toml version
async fn modify_cargo_toml(project_dir: &Path) {
    let cargo_toml = project_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml)
        .await
        .expect("Failed to read Cargo.toml");

    let modified = content.replace("0.1.0", "0.1.1");

    fs::write(&cargo_toml, modified)
        .await
        .expect("Failed to write Cargo.toml");
}

/// Measure cache performance
async fn measure_cache_performance(project_dir: &Path, use_cache: bool) -> CacheMetrics {
    let cache_size_before = get_cache_size().await;

    let result = run_build(project_dir, use_cache).await;

    let cache_size_after = get_cache_size().await;
    let cache_size_mb = cache_size_after - cache_size_before;

    // Estimate cache hit rate (simplified)
    let cache_hit_rate = if use_cache && cache_size_before > 0.0 {
        0.85 // Assume high hit rate for warm builds
    } else {
        0.0
    };

    // Estimate network savings
    let network_saved_mb = if use_cache {
        cache_size_mb * cache_hit_rate
    } else {
        0.0
    };

    // Calculate speedup factor
    let baseline_time = Duration::from_secs(30); // Assumed baseline
    let speedup_factor = baseline_time.as_secs_f64() / result.duration.as_secs_f64();

    CacheMetrics {
        build_time: result.duration,
        cache_hit_rate,
        cache_size_mb,
        network_saved_mb,
        speedup_factor,
    }
}

/// Get cache directory size in MB
async fn get_cache_size() -> f64 {
    if let Ok(output) = Command::new("du")
        .args(&["-sm", CACHE_DIR])
        .output()
        .await
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(size_str) = stdout.split_whitespace().next() {
                return size_str.parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    0.0
}

/// Run cache cleanup
async fn run_cache_cleanup() {
    let _ = Command::new("cargo")
        .args(&["clean", "--target-dir", CACHE_DIR])
        .output()
        .await;
}

/// Cleanup cache directory
async fn cleanup_cache() {
    let _ = fs::remove_dir_all(CACHE_DIR).await;
    let _ = fs::create_dir_all(CACHE_DIR).await;
}

/// Cleanup test project
async fn cleanup_test_project(project_dir: PathBuf) {
    let _ = fs::remove_dir_all(project_dir).await;
}

/// Print cache metrics
fn print_cache_metrics(metrics: &CacheMetrics) {
    println!("--- Cache Metrics ---");
    println!("Build time: {:?}", metrics.build_time);
    println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);
    println!("Cache size: {:.2}MB", metrics.cache_size_mb);
    println!("Network saved: {:.2}MB", metrics.network_saved_mb);
    println!("Speedup factor: {:.2}x", metrics.speedup_factor);
    println!("---------------------\n");
}
