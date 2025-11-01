# Raibid-CI Scripts

Automation scripts for development, deployment, infrastructure management, and utilities.

## Directory Structure

```
scripts/
├── README.md                      # This file
├── TEMPLATE.sh                    # Bash script template
├── TEMPLATE.nu                    # Nushell script template
├── dev/                           # Development and orchestration scripts
├── deploy/                        # Deployment and release scripts
├── infra/                         # Infrastructure and CI/CD scripts
└── utils/                         # General utility scripts
```

## Script Categories

### Development Scripts (`dev/`)

Scripts for local development, multi-agent orchestration, and development workflows.

#### `launch-orchestrator.nu`

Launches the Claude Code orchestrator agent that coordinates multi-agent parallel development.

**Usage:**
```bash
nu scripts/dev/launch-orchestrator.nu
```

**What it does:**
1. Checks prerequisites (gh CLI, authentication, documentation)
2. Shows current project status (open issues, paused work)
3. Displays orchestrator instructions
4. Provides guidance for spawning the orchestrator agent

**Prerequisites:**
- [Nushell](https://www.nushell.sh/) installed
- [GitHub CLI (gh)](https://cli.github.com/) installed and authenticated
- Claude Code CLI installed and authenticated
- Repository properly configured

**Note:** This script displays instructions and checks prerequisites. To actually spawn the orchestrator agent in Claude Code, use the Task tool as shown in the script output.

#### `orchestrator-monitor.sh`

Monitors GitHub issues and orchestrates agent spawning for raibid-ci development.

**Usage:**
```bash
./scripts/dev/orchestrator-monitor.sh
```

**What it does:**
1. Checks status of open issues
2. Detects when clarifying questions are answered
3. Posts resumption signals on issues
4. Spawns development agents for ready work
5. Maintains state in `/tmp/raibid_orchestrator_state.json`

**Use Case:** Run periodically (e.g., via cron) to automate agent spawning.

### Deployment Scripts (`deploy/`)

Scripts for building releases, packaging, and deployment automation.

#### `build-release.sh`

Build release artifacts for raibid-cli with cross-platform support.

**Usage:**
```bash
./scripts/deploy/build-release.sh [VERSION]
```

**What it does:**
1. Builds x86_64 binary
2. Builds ARM64/aarch64 binary (if toolchain installed)
3. Creates installation scripts
4. Generates documentation package
5. Creates release tarballs
6. Generates SHA256 checksums

**Example:**
```bash
./scripts/deploy/build-release.sh 0.1.0
```

**Output:**
- `release/raibid-cli-VERSION-x86_64-linux.tar.gz`
- `release/raibid-cli-VERSION-aarch64-linux.tar.gz`
- `release/SHA256SUMS`

**Prerequisites:**
- Rust toolchain installed
- `aarch64-unknown-linux-gnu` target (optional, for ARM64 builds)

### Infrastructure Scripts (`infra/`)

Scripts for CI/CD automation, issue management, and GitHub Actions integration.

#### `spawn-agent-comment.sh`

Posts spawn trigger comment on GitHub issue for orchestrator detection.

**Usage:**
```bash
ISSUE_NUMBER=42 ./scripts/infra/spawn-agent-comment.sh
```

**Environment Variables:**
- `ISSUE_NUMBER` - GitHub issue number to comment on

**What it does:**
1. Fetches issue details
2. Determines appropriate agent type
3. Posts structured spawn trigger comment
4. Includes orchestrator state metadata

**Use Case:** Called by GitHub Actions workflows to signal issue readiness.

#### `check-issue-readiness.sh`

Analyzes issue to determine if clarifying questions are answered.

**Usage:**
```bash
ISSUE_NUMBER=42 ./scripts/infra/check-issue-readiness.sh
```

**Environment Variables:**
- `ISSUE_NUMBER` - GitHub issue number to check

**GitHub Action Outputs:**
- `ready` - true/false indicating if issue is ready
- `unanswered_count` - Number of unanswered questions
- `total_questions` - Total number of clarifying questions

**Use Case:** Called by GitHub Actions to gate issue assignment.

#### `check-draft-status.sh`

Checks if issue has draft label for enrichment workflow.

**Usage:**
```bash
ISSUE_NUMBER=42 ./scripts/infra/check-draft-status.sh
```

**Environment Variables:**
- `ISSUE_NUMBER` - GitHub issue number to check

**GitHub Action Outputs:**
- `is_draft` - true/false indicating draft status
- `draft_label` - Name of the draft label if present

**Use Case:** Routes draft issues through enrichment process before implementation.

#### `assign-next-issue.sh`

Finds highest priority ready issue for agent assignment.

**Usage:**
```bash
./scripts/infra/assign-next-issue.sh
```

**GitHub Action Outputs:**
- `issue_number` - Number of next issue to assign (or empty if none)

**Priority Order:**
1. Critical priority
2. High priority
3. Medium priority
4. Low priority
5. Oldest ready issue (no priority label)

**Use Case:** Called by GitHub Actions to assign work to available agents.

### Utility Scripts (`utils/`)

General-purpose utility scripts for common tasks. Currently empty, ready for future utilities.

## Script Templates

Two templates are provided for creating new scripts:

### Bash Script Template (`TEMPLATE.sh`)

Full-featured Bash script template with:
- Standard header and documentation
- Argument parsing with help, verbose, debug flags
- Colored logging functions (info, success, warning, error, debug)
- Dependency checking
- Cleanup traps
- Error handling with `set -euo pipefail`

**To create a new Bash script:**
```bash
cp scripts/TEMPLATE.sh scripts/category/new-script.sh
# Edit and customize the script
chmod +x scripts/category/new-script.sh
```

### Nushell Script Template (`TEMPLATE.nu`)

Full-featured Nushell script template with:
- Standard header and documentation
- Argument parsing with flags
- Colored logging functions
- Dependency checking
- Modern Nushell idioms

**To create a new Nushell script:**
```bash
cp scripts/TEMPLATE.nu scripts/category/new-script.nu
# Edit and customize the script
chmod +x scripts/category/new-script.nu
```

## Naming Conventions

All scripts follow these naming conventions:

1. **Use kebab-case:** `script-name.sh` or `script-name.nu`
2. **Include file extension:** `.sh` for Bash, `.nu` for Nushell, `.py` for Python
3. **Be descriptive:** Name should clearly indicate purpose
4. **Avoid abbreviations:** Use full words for clarity

**Examples:**
- `build-release.sh` (not `bld-rel.sh`)
- `check-issue-readiness.sh` (not `check_issue.sh`)
- `launch-orchestrator.nu` (not `launch.nu`)

## Error Handling Standards

All scripts should follow these error handling practices:

1. **Use strict mode:**
   - Bash: `set -euo pipefail`
   - Exit on errors, undefined variables, and pipe failures

2. **Meaningful exit codes:**
   - `0` - Success
   - `1` - General error
   - `2` - Invalid arguments
   - `3` - Missing dependencies
   - `4+` - Script-specific errors

3. **Colored output:**
   - Use color codes for clarity (info=blue, success=green, warning=yellow, error=red)
   - Always reset colors with `NC` (no color)

4. **Trap cleanup:**
   - Use `trap cleanup EXIT` for cleanup on exit
   - Handle both success and error cases

## Dependency Documentation

Scripts must document their dependencies in the header:

```bash
# Dependencies:
#   - bash >= 4.0
#   - jq >= 1.6 (JSON processing)
#   - gh (GitHub CLI)
#   - curl (HTTP requests)
```

And check for dependencies in code:

```bash
check_dependencies() {
    local missing_deps=()

    if ! command_exists "jq"; then
        missing_deps+=("jq")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 3
    fi
}
```

## Script Execution

### Making Scripts Executable

```bash
chmod +x scripts/category/script-name.sh
```

### Running Scripts

```bash
# From project root
./scripts/category/script-name.sh [OPTIONS] [ARGS]

# Or with full path
/path/to/raibid-ci/scripts/category/script-name.sh [OPTIONS] [ARGS]
```

### Nushell Scripts

```bash
# Run directly (if executable)
./scripts/category/script-name.nu [OPTIONS] [ARGS]

# Or explicitly with nu
nu scripts/category/script-name.nu [OPTIONS] [ARGS]
```

## Linting and Quality Checks

### ShellCheck (Bash)

All Bash scripts should pass ShellCheck:

```bash
shellcheck scripts/**/*.sh
```

**Install ShellCheck:**
```bash
# Ubuntu/Debian
sudo apt install shellcheck

# macOS
brew install shellcheck

# Or via snap
sudo snap install shellcheck
```

### Nushell Format

Nushell scripts should follow Nushell formatting conventions:

```bash
nu --check scripts/**/*.nu
```

## Development Workflow

### 1. Launch Orchestrator
```bash
nu scripts/dev/launch-orchestrator.nu
```

### 2. Orchestrator Monitors Issues
The orchestrator will:
- Monitor GitHub issues every 5 minutes
- Check for clarifying questions that need answers
- Detect when questions are answered
- Spawn development agents for ready issues
- Track progress and dependencies
- Post status updates

### 3. Answer Clarifying Questions
As project maintainer, review and answer questions on GitHub issues:
```bash
gh issue list --label "status:paused"
gh issue view <number>
```

Answer questions in issue comments using this format:
```markdown
## Answers to Clarifying Questions

**Q1: Project naming**
A: Use `raibid` (shorter). Users can alias to `raibid-cli` if they prefer.

**Q2: Configuration format**
A: Use YAML. More common in DevOps tooling and supports comments.
```

### 4. Orchestrator Resumes Agents
Once questions are answered, the orchestrator will:
- Detect the answers
- Post resumption signal on issue
- Spawn or resume development agents
- Agents proceed with TDD workflow

### 5. Monitor Progress
```bash
# View all open issues
gh issue list

# View open PRs
gh pr list

# View CI runs
gh run list

# View issue with comments
gh issue view <number>
```

## Testing Scripts

When developing new scripts:

1. **Test with various inputs:** Valid, invalid, edge cases
2. **Test error conditions:** Missing deps, failed commands
3. **Test cleanup:** Ensure cleanup runs on success and failure
4. **Test with ShellCheck:** Run linter before committing
5. **Test in CI:** Ensure scripts work in GitHub Actions environment

## Integration with GitHub Actions

Scripts in `infra/` are designed for GitHub Actions integration.

**Example workflow step:**
```yaml
- name: Check issue readiness
  id: check
  env:
    ISSUE_NUMBER: ${{ github.event.issue.number }}
  run: ./scripts/infra/check-issue-readiness.sh

- name: Use output
  if: steps.check.outputs.ready == 'true'
  run: echo "Issue is ready for work"
```

## Contributing New Scripts

When adding new scripts:

1. **Choose appropriate directory:**
   - `dev/` - Development and orchestration
   - `deploy/` - Deployment and releases
   - `infra/` - CI/CD and GitHub integration
   - `utils/` - General utilities

2. **Copy appropriate template:**
   ```bash
   cp scripts/TEMPLATE.sh scripts/category/new-script.sh
   ```

3. **Update header documentation:**
   - Description, usage, options, examples
   - Dependencies, exit codes, notes

4. **Implement functionality:**
   - Use logging functions
   - Check dependencies
   - Handle errors appropriately

5. **Make executable:**
   ```bash
   chmod +x scripts/category/new-script.sh
   ```

6. **Test thoroughly:**
   - Run with various inputs
   - Test error conditions
   - Run ShellCheck

7. **Update this README:**
   - Add entry in appropriate category
   - Document usage and examples

8. **Commit and create PR:**
   ```bash
   git add scripts/category/new-script.sh scripts/README.md
   git commit -m "feat: add new-script.sh for [purpose]"
   ```

## Common Issues and Solutions

### Issue: Script not executable
**Solution:** `chmod +x scripts/category/script-name.sh`

### Issue: Command not found
**Solution:** Check dependencies are installed, check PATH

### Issue: GitHub Actions output not working
**Solution:** Ensure using `>> $GITHUB_OUTPUT` syntax (not deprecated `set-output`)

### Issue: Nushell script fails
**Solution:** Check Nushell version compatibility, verify syntax with `nu --check`

## Additional Resources

- **Orchestrator Guide:** `docs/ORCHESTRATOR_AGENT.md`
- **Questions Document:** `docs/CLARIFYING_QUESTIONS.md`
- **Setup Summary:** `docs/SETUP_COMPLETE.md`
- **Workstreams:** `docs/workstreams/`
- **Quick Start:** `docs/workstreams/START_HERE.md`
- **ShellCheck Wiki:** https://github.com/koalaman/shellcheck/wiki
- **Nushell Book:** https://www.nushell.sh/book/

## Support

For issues or questions:
- Check `docs/SETUP_COMPLETE.md` for troubleshooting
- Review `docs/ORCHESTRATION.md` for multi-agent workflow
- See `docs/workstreams/START_HERE.md` for quick reference
- Open an issue: https://github.com/raibid-labs/raibid-ci/issues
