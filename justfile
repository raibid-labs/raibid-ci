# raibid-ci justfile
# Development automation for building, running, and testing

# Default recipe - show available commands
default:
    @just --list

# === Build Commands ===

# Build all workspace crates in dev mode
build:
    cargo build --workspace

# Build all workspace crates in release mode
build-release:
    cargo build --workspace --release

# Build only the CLI in dev mode
build-cli:
    cargo build --package raibid-cli

# Build only the CLI in release mode
build-cli-release:
    cargo build --package raibid-cli --release

# Build only the server in dev mode
build-server:
    cargo build --package raibid-server

# Build only the server in release mode
build-server-release:
    cargo build --package raibid-server --release

# Build only the agent in dev mode
build-agent:
    cargo build --package raibid-agent

# Build only the agent in release mode
build-agent-release:
    cargo build --package raibid-agent --release

# Clean build artifacts
clean:
    cargo clean

# === Run Commands ===

# Run the CLI with help
cli *ARGS:
    cargo run --package raibid-cli -- {{ARGS}}

# Run the CLI in release mode
cli-release *ARGS:
    cargo run --package raibid-cli --release -- {{ARGS}}

# Launch the TUI dashboard
tui:
    cargo run --package raibid-cli -- tui

# Launch the TUI dashboard in release mode
tui-release:
    cargo run --package raibid-cli --release -- tui

# Run the server (default config)
server PORT="8080":
    RUST_LOG=raibid_server=debug,tower_http=debug cargo run --package raibid-server -- --port {{PORT}}

# Run the server in release mode
server-release PORT="8080":
    RUST_LOG=raibid_server=info cargo run --package raibid-server --release -- --port {{PORT}}

# Run the agent
agent:
    RUST_LOG=raibid_agent=debug cargo run --package raibid-agent

# Run the agent in release mode
agent-release:
    RUST_LOG=raibid_agent=info cargo run --package raibid-agent --release

# === Test Commands ===

# Run all tests
test:
    cargo test --workspace

# Run all tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run tests for a specific package
test-package PACKAGE:
    cargo test --package {{PACKAGE}}

# Run only unit tests (lib tests)
test-unit:
    cargo test --workspace --lib

# Run only integration tests
test-integration:
    cargo test --workspace --test '*'

# Run only doc tests
test-doc:
    cargo test --workspace --doc

# Run tests for the CLI
test-cli:
    cargo test --package raibid-cli

# Run tests for the server
test-server:
    cargo test --package raibid-server

# Run tests for the agent
test-agent:
    cargo test --package raibid-agent

# Run tests for common lib
test-common:
    cargo test --package raibid-common

# Run tests for TUI
test-tui:
    cargo test --package raibid-tui

# Run specific test by name
test-name NAME:
    cargo test --workspace {{NAME}}

# Run tests and show coverage (requires tarpaulin)
test-coverage:
    cargo tarpaulin --workspace --out Html --output-dir coverage

# Run E2E tests (requires external services)
test-e2e:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running E2E tests (requires external services)..."
    TEST_EXTERNAL=1 cargo test --test ci_pipeline_test -- --ignored --nocapture

# Run E2E tests and cleanup
test-e2e-clean: test-e2e
    ./tests/e2e/cleanup.sh

# Run performance tests
test-performance:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running performance tests..."
    TEST_EXTERNAL=1 cargo test --test performance_test -- --ignored --nocapture

# Run cache tests
test-cache:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running cache tests..."
    TEST_EXTERNAL=1 cargo test --test cache_test -- --ignored --nocapture

# Run failure recovery tests
test-failure:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running failure recovery tests..."
    TEST_EXTERNAL=1 cargo test --test failure_test -- --ignored --nocapture

# Run all external tests (E2E, performance, cache, failure)
test-all-external: test-e2e test-performance test-cache test-failure
    @echo "All external tests completed!"

# === Check Commands ===

# Run cargo check on workspace
check:
    cargo check --workspace

# Run clippy (linter)
lint:
    cargo clippy --workspace -- -D warnings

# Run clippy with auto-fixes
lint-fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staged

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format all code
fmt:
    cargo fmt --all

# Run all quality checks (check, lint, fmt, test)
ci: check lint fmt-check test
    @echo "All CI checks passed!"

# === Development Commands ===

# Watch for changes and rebuild
watch:
    cargo watch -x "build --workspace"

# Watch for changes and run tests
watch-test:
    cargo watch -x "test --workspace"

# Watch for changes and run the CLI
watch-cli *ARGS:
    cargo watch -x "run --package raibid-cli -- {{ARGS}}"

# Watch for changes and run the server
watch-server PORT="8080":
    cargo watch -x "run --package raibid-server -- --port {{PORT}}"

# === Install Commands ===

# Install the CLI binary globally
install-cli:
    cargo install --path crates/cli

# Install all development tools
install-tools:
    cargo install cargo-watch cargo-tarpaulin

# === Utility Commands ===

# Show dependency tree
deps:
    cargo tree --workspace

# Show outdated dependencies
outdated:
    cargo outdated --workspace

# Update dependencies
update:
    cargo update

# Generate documentation
docs:
    cargo doc --workspace --no-deps --open

# Show workspace info
info:
    @echo "=== Workspace Info ==="
    @echo "Crates:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | "  - \(.name) v\(.version)"'
    @echo ""
    @echo "Binaries:"
    @ls -1 target/debug/raibid* 2>/dev/null | grep -v '\.d$$' | sed 's/^/  - /' || echo "  (none built yet)"

# Run benchmarks (if any)
bench:
    cargo bench --workspace

# === Docker Commands ===

# Build Docker image for server (local)
docker-build-server:
    docker build -t raibid-server:latest -f crates/server/Dockerfile .

# Build Docker image for server and push to Gitea registry
docker-build-server-registry:
    docker build -t localhost:30500/raibid-admin/raibid-server:latest -f crates/server/Dockerfile .
    docker push localhost:30500/raibid-admin/raibid-server:latest

# Build Docker image for agent (local)
docker-build-agent:
    docker build -t raibid-agent:latest -f crates/agent/Dockerfile .

# Build Docker image for agent and push to Gitea registry
docker-build-agent-registry:
    docker build -t localhost:30500/raibid-admin/raibid-agent:latest -f crates/agent/Dockerfile .
    docker push localhost:30500/raibid-admin/raibid-agent:latest

# Build both Docker images (local tags)
docker-build-all: docker-build-server docker-build-agent

# Build both Docker images and push to registry
docker-build-all-registry: docker-build-server-registry docker-build-agent-registry

# === Registry Commands ===

# Login to Gitea container registry
registry-login:
    docker login localhost:30500 -u raibid-admin -p adminadmin

# Push server image to registry (assumes already built with registry tag)
registry-push-server:
    docker push localhost:30500/raibid-admin/raibid-server:latest

# Push agent image to registry (assumes already built with registry tag)
registry-push-agent:
    docker push localhost:30500/raibid-admin/raibid-agent:latest

# Pull server image from registry
registry-pull-server:
    docker pull localhost:30500/raibid-admin/raibid-server:latest

# Pull agent image from registry
registry-pull-agent:
    docker pull localhost:30500/raibid-admin/raibid-agent:latest

# List images in registry via API
registry-list:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Fetching packages from Gitea registry..."
    curl -s -H "Authorization: token 53a9be18fd910b12c7095bab63c4e90e2a6815fb" \
        "http://localhost:3000/api/v1/packages/raibid-admin?type=container" | \
        jq -r '.[] | "\(.name):\(.version)"'

# Show registry info
registry-info:
    @echo "=== Gitea Container Registry ==="
    @echo "Registry URL: localhost:30500"
    @echo "Namespace: raibid-admin"
    @echo "User: raibid-admin"
    @echo "Password: adminadmin"
    @echo ""
    @echo "Images:"
    @echo "  - localhost:30500/raibid-admin/raibid-server:latest"
    @echo "  - localhost:30500/raibid-admin/raibid-agent:latest"
    @echo ""
    @echo "Web UI: http://localhost:3000 (Packages > Container)"

# Tag and push custom image to registry
registry-push IMAGE TAG="latest":
    docker tag {{IMAGE}}:{{TAG}} localhost:30500/raibid-admin/{{IMAGE}}:{{TAG}}
    docker push localhost:30500/raibid-admin/{{IMAGE}}:{{TAG}}

# === Tanka Commands ===

# Show Tanka-generated manifests
tk-show:
    cd tanka && tk show environments/local

# Show Tanka-generated manifests with output redirect
tk-show-yaml:
    cd tanka && tk show environments/local --dangerous-allow-redirect

# Diff Tanka manifests against cluster (requires k3s)
tk-diff:
    cd tanka && tk diff environments/local

# Apply Tanka manifests to cluster (requires k3s)
tk-apply:
    cd tanka && tk apply environments/local

# Export Tanka manifests to YAML files
tk-export OUTPUT="manifests":
    cd tanka && tk export {{OUTPUT}} environments/local

# Vendor Helm charts for Tanka
tk-vendor:
    #!/usr/bin/env bash
    set -euo pipefail
    cd tanka
    echo "Adding Helm repositories..."
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo add gitea-charts https://dl.gitea.io/charts/
    helm repo add kedacore https://kedacore.github.io/charts
    helm repo add fluxcd-community https://fluxcd-community.github.io/helm-charts
    helm repo update
    echo "Pulling Helm charts..."
    helm pull bitnami/redis --version 18.19.4 --untar -d vendor/
    helm pull gitea-charts/gitea --version 10.1.4 --untar -d vendor/
    helm pull kedacore/keda --version 2.14.0 --untar -d vendor/
    helm pull fluxcd-community/flux2 --version 2.12.4 --untar -d vendor/
    echo "✓ Helm charts vendored successfully"
    ls -la vendor/

# Count resources generated by Tanka
tk-count:
    cd tanka && tk show environments/local --dangerous-allow-redirect | grep -c "^kind:" || true

# Validate Tanka configuration (jsonnet syntax)
tk-validate:
    cd tanka && tk fmt --test

# Install/update Tanka dependencies
tk-install:
    cd tanka && jb install

# === Tilt Commands ===

# Start Tilt development environment (interactive)
dev:
    tilt up

# Stop Tilt development environment
dev-down:
    tilt down

# Run Tilt in CI mode (headless)
dev-ci:
    tilt ci

# View Tilt resources
dev-status:
    tilt get all

# === Combined Setup Commands ===

# Complete setup: vendor charts and start dev environment
setup-dev: tk-vendor dev
    @echo "Development environment ready!"

# Quick start (assumes charts already vendored)
quick-start: dev

# === Git Commands ===

# Show current branch and status
status:
    @git branch --show-current
    @git status --short

# Run pre-commit checks
pre-commit: fmt lint test
    @echo "Pre-commit checks passed! Ready to commit."
