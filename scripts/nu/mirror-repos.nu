#!/usr/bin/env nu
# Mirror GitHub repositories to Gitea and trigger test job
#
# This script:
# 1. Runs the mirroring process using raibid-cli
# 2. Triggers a test job on a mirrored repository

use modules/gitea.nu *

def main [] {
    print $"(ansi blue)Starting repository mirroring process...(ansi reset)"

    # Check if GITHUB_TOKEN is set
    if "GITHUB_TOKEN" not-in $env {
        print $"(ansi red)⚠ WARNING:(ansi reset) GITHUB_TOKEN not set. Only public repos will be accessible."
        print "Set it with: export GITHUB_TOKEN=\"ghp_...\""
    }

    # Check if Gitea is accessible
    print "\nChecking Gitea connectivity..."
    let gitea_url = "http://localhost:3000"
    try {
        gitea-check-connection $gitea_url
    } catch {
        print $"(ansi red)✗(ansi reset) Cannot connect to Gitea at ($gitea_url)"
        print "Make sure Gitea is running (tilt up should start it)"
        exit 1
    }

    # TODO: Once raibid-cli mirror command is implemented, use it here
    # For now, provide instructions
    print $"\n(ansi green)✓(ansi reset) Gitea is accessible"
    print "\nTo trigger mirroring:"
    print "  1. Ensure GITHUB_TOKEN is set: export GITHUB_TOKEN=\"ghp_...\""
    print "  2. Ensure GITEA_TOKEN is set: export GITEA_TOKEN=\"...\""
    print "  3. Run: cargo run --bin raibid-cli -- mirror run"
    print "\nConfiguration loaded from: ./raibid.yaml"

    # After mirroring completes, trigger a test job
    print "\n(ansi blue)After mirroring completes, trigger a test job:(ansi reset)"
    print "  redis-cli XADD raibid:jobs * job '{\"id\":\"test-1\",\"repo\":\"raibid-labs/raibid-ci\",\"branch\":\"main\",\"commit\":\"HEAD\",\"status\":\"Pending\"}'"
}
