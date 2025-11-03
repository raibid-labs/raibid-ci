//! Integration tests for infrastructure management commands
//!
//! These tests verify that the CLI commands are properly wired up and
//! can handle various argument combinations.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that the main help displays all commands
#[test]
fn test_help_shows_all_commands() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("health"))
        .stdout(predicate::str::contains("destroy"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("jobs"))
        .stdout(predicate::str::contains("tui"));
}

/// Test init command help
#[test]
fn test_init_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Initialize infrastructure components",
        ))
        .stdout(predicate::str::contains("k3s"))
        .stdout(predicate::str::contains("gitea"))
        .stdout(predicate::str::contains("redis"))
        .stdout(predicate::str::contains("keda"))
        .stdout(predicate::str::contains("flux"))
        .stdout(predicate::str::contains("all"));
}

/// Test init k3s subcommand help
#[test]
fn test_init_k3s_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "k3s", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Initialize k3s Kubernetes cluster",
        ))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--skip-checks"))
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--rootless"));
}

/// Test init gitea subcommand help
#[test]
fn test_init_gitea_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "gitea", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize Gitea Git server"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--skip-checks"))
        .stdout(predicate::str::contains("--service-type"))
        .stdout(predicate::str::contains("--admin-user"));
}

/// Test init redis subcommand help
#[test]
fn test_init_redis_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "redis", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize Redis with Streams"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--skip-checks"))
        .stdout(predicate::str::contains("--persistence"));
}

/// Test init keda subcommand help
#[test]
fn test_init_keda_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "keda", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize KEDA autoscaler"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--skip-checks"));
}

/// Test init flux subcommand help
#[test]
fn test_init_flux_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "flux", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize Flux GitOps"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--skip-checks"))
        .stdout(predicate::str::contains("--repo-path"));
}

/// Test init all subcommand help
#[test]
fn test_init_all_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["init", "all", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize all components"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--skip-checks"));
}

/// Test health command help
#[test]
fn test_health_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["health", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Check infrastructure health"))
        .stdout(predicate::str::contains("COMPONENT"))
        .stdout(predicate::str::contains("--json"));
}

/// Test destroy command help
#[test]
fn test_destroy_help() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["destroy", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Destroy infrastructure components",
        ))
        .stdout(predicate::str::contains("COMPONENT"))
        .stdout(predicate::str::contains("--yes"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--force"));
}

/// Test init without subcommand fails with error
#[test]
fn test_init_without_subcommand_fails() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.arg("init");

    // Clap shows error when required subcommand is missing
    cmd.assert().failure().code(2); // Clap error code for usage error
}

/// Test destroy without component fails
#[test]
fn test_destroy_without_component_fails() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.arg("destroy");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

/// Test that init command is visible in help but setup is not
#[test]
fn test_init_visible_setup_hidden() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.arg("--help");

    // Init should be visible, setup should not be (it's hidden)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Initialize infrastructure components",
        ))
        .stdout(predicate::str::contains("Setup infrastructure component").not());
}

/// Test verbose flag works globally
#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.args(["-v", "init", "--help"]);

    cmd.assert().success();
}

/// Test that version flag works
#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("raibid"));
}

/// Test that no command shows error message
#[test]
fn test_no_command_shows_message() {
    let mut cmd = Command::cargo_bin("raibid").unwrap();

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("No command specified"));
}
