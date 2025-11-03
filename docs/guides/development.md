# Development Guide

This guide covers the development workflow, tooling, and best practices for contributing to raibid-ci.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Development Tools](#development-tools)
- [Pre-commit Hooks](#pre-commit-hooks)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Workflow](#workflow)

## Prerequisites

### Required Tools

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Just**: `cargo install just`
- **Git**: Standard git installation
- **Python 3**: For YAML/JSON validation in pre-commit hooks

### Optional Tools

- **Nushell** (0.96+): For automation scripts - see [Nushell Guide](./nushell.md)
- **Docker**: For container builds
- **kubectl**: For Kubernetes interaction
- **Tilt**: For development environment
- **Tanka**: For infrastructure management

## Getting Started

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/raibid-labs/raibid-ci.git
cd raibid-ci

# Install pre-commit hooks
./scripts/dev/install-hooks.sh

# Build the project
just build

# Run tests
just test
```

### Project Structure

```
raibid-ci/
├── crates/           # Rust workspace crates
│   ├── cli/         # CLI/TUI client
│   ├── server/      # API server
│   ├── agent/       # Build agent
│   ├── common/      # Shared code
│   └── tui/         # Terminal UI library
├── docs/            # Documentation
├── infra/           # Infrastructure manifests
├── scripts/         # Automation scripts
│   ├── nu/         # Nushell scripts
│   └── dev/        # Development tools
├── tanka/           # Tanka/Jsonnet configs
└── tests/           # Integration tests
```

## Development Tools

### Just Task Runner

The project uses [Just](https://github.com/casey/just) for task automation. View all available commands:

```bash
just --list
```

Common commands:

```bash
# Build
just build              # Build all crates
just build-cli          # Build CLI only
just build-server       # Build server only

# Run
just cli --help         # Run CLI
just tui                # Launch TUI
just server             # Run server

# Test
just test               # Run all tests
just test-unit          # Unit tests only
just test-integration   # Integration tests only

# Quality checks
just fmt                # Format code
just lint               # Run clippy
just check              # Run cargo check
just ci                 # Run all CI checks
```

### Cargo Workspace

The project is a Cargo workspace with multiple crates:

```bash
# Build specific crate
cargo build --package raibid-cli

# Test specific crate
cargo test --package raibid-server

# Check specific crate
cargo check --package raibid-agent
```

## Pre-commit Hooks

Pre-commit hooks enforce code quality automatically before commits.

### Installation

Install hooks once after cloning:

```bash
./scripts/dev/install-hooks.sh
```

This installs a pre-commit hook that runs:

1. **Code formatting check** (`cargo fmt --check`)
2. **Linting** (`cargo clippy`)
3. **Unit tests** (`cargo test --lib`)
4. **YAML/JSON validation** (infrastructure files)
5. **Nushell script validation** (if Nushell installed)

### What Gets Checked

#### 1. Code Formatting

Ensures all Rust code follows standard formatting rules:

```bash
cargo fmt --all -- --check
```

To fix formatting issues:

```bash
cargo fmt --all
# Or use: just fmt
```

#### 2. Clippy Linting

Runs Clippy with warnings as errors:

```bash
cargo clippy --workspace -- -D warnings
```

To see and fix issues:

```bash
cargo clippy --workspace
# Or use: just lint
```

#### 3. Unit Tests

Runs all library unit tests with a 120-second timeout:

```bash
cargo test --workspace --lib
```

To run tests manually:

```bash
cargo test --workspace
# Or use: just test
```

#### 4. Infrastructure Validation

Validates YAML and JSON files in the `infra/` directory using Python:

- Skips vendored files (`vendor/`, `charts/`)
- Skips Taskfiles (which use Go templates)
- Parses all other YAML/JSON for syntax errors

#### 5. Nushell Scripts

Validates Nushell scripts if Nushell is installed:

```bash
nu scripts/nu/validate-setup.nu
```

### Bypassing Hooks

In rare cases, you may need to bypass pre-commit checks:

```bash
# Bypass for a single commit
git commit --no-verify -m "message"

# Or use the shorthand
git commit -n -m "message"
```

**⚠️ Warning**: Only bypass hooks if you have a good reason (e.g., work in progress, merge conflicts). CI will still run all checks on pull requests.

### Hook Behavior

- **Automatic**: Runs on every `git commit`
- **Fast**: Only runs unit tests (not integration tests)
- **Safe**: Skips checks during merge/rebase
- **Informative**: Shows clear error messages with fix commands
- **Non-destructive**: Never modifies your code

### Troubleshooting

#### Hook Not Running

Ensure the hook is installed and executable:

```bash
ls -la .git/hooks/pre-commit
# Should show: -rwxr-xr-x (executable)

# If not, reinstall:
./scripts/dev/install-hooks.sh
```

#### Tests Timeout

Unit tests have a 120-second timeout. If tests are slow:

```bash
# Run tests manually to see which are slow
cargo test --workspace --lib -- --nocapture

# Consider optimizing or splitting slow tests
```

#### Python Not Available

YAML/JSON validation requires Python 3 with PyYAML:

```bash
# Install PyYAML
pip3 install pyyaml

# Or on Ubuntu/Debian
sudo apt-get install python3-yaml
```

#### Nushell Validation Fails

If Nushell validation fails:

```bash
# Run validation manually to see details
nu scripts/nu/validate-setup.nu

# Check Nushell version (requires 0.96+)
nu --version
```

### Uninstalling Hooks

To remove pre-commit hooks:

```bash
rm .git/hooks/pre-commit
```

## Testing

### Test Categories

```bash
# Unit tests (fast, runs in pre-commit)
just test-unit

# Integration tests (slower, requires infrastructure)
just test-integration

# Doc tests
just test-doc

# All tests
just test
```

### Running Specific Tests

```bash
# Test by name
just test-name my_test_function

# Test specific package
just test-package raibid-cli

# Test with output
just test-verbose
```

### Coverage

Generate test coverage reports:

```bash
just test-coverage
```

Opens HTML coverage report in `coverage/`.

## Code Quality

### Formatting

Always format code before committing:

```bash
just fmt
```

Check formatting without modifying:

```bash
just fmt-check
```

### Linting

Run Clippy to catch common issues:

```bash
just lint
```

Auto-fix some issues:

```bash
just lint-fix
```

### Full CI Check

Run all CI checks locally:

```bash
just ci
```

This runs: `check` + `lint` + `fmt-check` + `test`

## Workflow

### Typical Development Workflow

1. **Create a branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes**

   Edit code, add tests, update docs

3. **Run checks frequently**

   ```bash
   just fmt      # Format code
   just lint     # Check for issues
   just test     # Run tests
   ```

4. **Commit changes**

   ```bash
   git add .
   git commit -m "feat: add my feature"
   # Pre-commit hooks run automatically
   ```

5. **Push and create PR**

   ```bash
   git push origin feature/my-feature
   # Create PR on GitHub
   ```

### Watch Mode

Use cargo-watch for automatic rebuilds:

```bash
# Watch and rebuild
just watch

# Watch and test
just watch-test

# Watch and run CLI
just watch-cli

# Watch and run server
just watch-server
```

### Working with the TUI

Launch the TUI dashboard:

```bash
just tui
```

Development tips:

- TUI requires a terminal with mouse support
- Use `Ctrl+C` or `q` to quit
- Logs are written to `raibid-tui.log`

### Working with Infrastructure

See [Infrastructure Guide](../infra/README.md) for details on:

- Tanka/Jsonnet development
- Tilt development environment
- Kubernetes manifests
- Helm charts

## Best Practices

### Code Style

- Follow Rust standard conventions
- Use `cargo fmt` for formatting
- Address all Clippy warnings
- Write doc comments for public APIs
- Keep functions focused and small

### Testing

- Write unit tests for all logic
- Use integration tests for API endpoints
- Mock external dependencies
- Test error cases and edge cases
- Keep tests fast and deterministic

### Commits

- Write clear, descriptive commit messages
- Use conventional commit format: `feat:`, `fix:`, `docs:`, etc.
- Keep commits atomic and focused
- Reference issues in commit messages
- Run pre-commit checks (don't bypass)

### Documentation

- Update docs when changing behavior
- Document public APIs with doc comments
- Include examples in doc comments
- Keep README and guides up to date
- Use Mermaid diagrams for complex concepts

## Additional Resources

- [Nushell Guide](./nushell.md) - Automation with Nushell
- [Error Recovery Guide](./error-recovery.md) - Handling errors
- [Main README](../../README.md) - Project overview
- [CLAUDE.md](../../CLAUDE.md) - AI assistant guidelines
