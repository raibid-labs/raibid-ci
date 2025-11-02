# Job Management CLI Commands

This document provides comprehensive documentation for the job management commands in the raibid-cli tool.

## Overview

The `raibid jobs` command group provides tools for managing CI/CD jobs, including listing, viewing details, triggering builds, canceling jobs, and viewing logs.

## Prerequisites

- raibid-cli must be installed and configured
- The raibid-server API must be running and accessible
- Set `RAIBID_API_URL` environment variable if not using default (http://localhost:8080)

## Commands

### jobs list

List jobs with optional filters.

**Usage:**
```bash
raibid jobs list [OPTIONS]
```

**Options:**
- `-s, --status <STATUS>` - Filter by status (pending, running, success, failed, cancelled)
- `-r, --repo <REPO>` - Filter by repository name
- `-b, --branch <BRANCH>` - Filter by branch name
- `-l, --limit <LIMIT>` - Maximum number of jobs to return (default: 25)
- `-o, --offset <OFFSET>` - Offset for pagination (default: 0)
- `--json` - Output as JSON

**Examples:**

List all jobs:
```bash
raibid jobs list
```

List only running jobs:
```bash
raibid jobs list --status running
```

List jobs for a specific repository:
```bash
raibid jobs list --repo raibid-cli
```

List failed jobs for a specific branch:
```bash
raibid jobs list --status failed --branch main
```

Get JSON output for scripting:
```bash
raibid jobs list --json | jq '.jobs[] | select(.status == "running")'
```

Paginate through results:
```bash
raibid jobs list --limit 10 --offset 0  # First page
raibid jobs list --limit 10 --offset 10 # Second page
```

**Output Format:**

Table format (default):
```
╔════════════╦══════════════╦═══════════╦══════════╦═══════════╦══════════╗
║     ID     ║ Repository   ║  Branch   ║  Status  ║  Started  ║ Duration ║
╠════════════╬══════════════╬═══════════╬══════════╬═══════════╬══════════╣
║ job-1234   ║ raibid-cli   ║ main      ║ ✓ Success║ 5m ago    ║ 2m 30s   ║
║ job-1235   ║ raibid-server║ feature/x ║ ▶ Running║ 1m ago    ║ 1m 15s...║
╚════════════╩══════════════╩═══════════╩══════════╩═══════════╩══════════╝

Info: Showing 2 of 25 jobs (offset: 0)
```

JSON format (with --json):
```json
{
  "jobs": [
    {
      "id": "job-1234",
      "repo": "raibid-cli",
      "branch": "main",
      "commit": "abc123def456",
      "status": "success",
      "started_at": "2025-11-01T20:00:00Z",
      "finished_at": "2025-11-01T20:02:30Z",
      "duration": 150,
      "agent_id": "agent-001",
      "exit_code": 0
    }
  ],
  "total": 25,
  "offset": 0,
  "limit": 25
}
```

### jobs show

Show detailed information about a specific job.

**Usage:**
```bash
raibid jobs show <JOB_ID> [OPTIONS]
```

**Arguments:**
- `<JOB_ID>` - The ID of the job to show

**Options:**
- `--json` - Output as JSON

**Examples:**

Show job details:
```bash
raibid jobs show job-1234
```

Get JSON output:
```bash
raibid jobs show job-1234 --json
```

**Output Format:**

Human-readable format (default):
```
Job Details
ID:             job-1234
Repository:     raibid-cli
Branch:         main
Commit:         abc123def456
Status:         ✓ Success
Started:        5m ago
Finished:       3m ago
Duration:       2m 30s
Agent:          agent-001
Exit Code:      0
```

### jobs logs

Show logs for a specific job.

**Usage:**
```bash
raibid jobs logs <JOB_ID> [OPTIONS]
```

**Arguments:**
- `<JOB_ID>` - The ID of the job to show logs for

**Options:**
- `-f, --follow` - Follow log output (stream new logs in real-time)
- `-t, --tail <TAIL>` - Number of lines to show from the end

**Examples:**

Show all logs for a job:
```bash
raibid jobs logs job-1234
```

Show last 50 lines:
```bash
raibid jobs logs job-1234 --tail 50
```

Follow logs in real-time:
```bash
raibid jobs logs job-1234 --follow
```

**Output Format:**

```
Info: Logs for job job-1234:
────────────────────────────────────────────────────────────────────────────────
[10:00:00] Job job-1234 started
[10:00:02] Cloning repository raibid-cli...
[10:00:05] Checked out branch: main
[10:00:10] Running cargo build --release...
[10:02:30] Build completed successfully
[10:02:31] Total duration: 150s
```

When following logs (with `--follow`), the output will continuously update until the job finishes:
```
Info: Following logs for job job-1234...
────────────────────────────────────────────────────────────────────────────────
[10:00:00] Job job-1234 started
[10:00:02] Cloning repository raibid-cli...
[10:00:05] Checked out branch: main
[10:00:10] Running cargo build --release...
[10:01:45] Compiling dependencies (1/3)
[10:02:20] Compiling dependencies (2/3)
[10:02:55] Compiling project crates
────────────────────────────────────────────────────────────────────────────────
Info: Job finished with status: ✓ Success
```

### jobs trigger

Trigger a new job.

**Usage:**
```bash
raibid jobs trigger --repo <REPO> --branch <BRANCH> [OPTIONS]
```

**Options:**
- `-r, --repo <REPO>` - Repository to build (required)
- `-b, --branch <BRANCH>` - Branch to build (required)
- `-c, --commit <COMMIT>` - Commit SHA to build (optional, defaults to latest)
- `--json` - Output as JSON

**Examples:**

Trigger a build for main branch:
```bash
raibid jobs trigger --repo raibid-cli --branch main
```

Trigger a build for a specific commit:
```bash
raibid jobs trigger --repo raibid-cli --branch main --commit abc123def456
```

Get JSON output:
```bash
raibid jobs trigger --repo raibid-cli --branch main --json
```

**Output Format:**

Human-readable format (default):
```
Info: Triggering build for raibid-cli/main...
Success: Job created successfully!

Job Details
ID:             job-1236
Repository:     raibid-cli
Branch:         main
Commit:         latest
Status:         ⏳ Pending
Started:        0s ago
Duration:       -
```

### jobs cancel

Cancel a running or pending job.

**Usage:**
```bash
raibid jobs cancel <JOB_ID> [OPTIONS]
```

**Arguments:**
- `<JOB_ID>` - The ID of the job to cancel

**Options:**
- `--json` - Output as JSON

**Examples:**

Cancel a job:
```bash
raibid jobs cancel job-1234
```

Get JSON output:
```bash
raibid jobs cancel job-1234 --json
```

**Output Format:**

Human-readable format (default):
```
Info: Cancelling job job-1234...
Success: Job cancelled successfully!

Job Details
ID:             job-1234
Repository:     raibid-cli
Branch:         feature/test
Commit:         xyz789abc012
Status:         ⊘ Cancelled
Started:        2m ago
Finished:       0s ago
Duration:       2m 5s
Agent:          agent-002
Exit Code:      143
```

## Environment Variables

- `RAIBID_API_URL` - Base URL for the raibid-server API (default: http://localhost:8080)

## Status Values

Jobs can have the following statuses:

- **Pending** (⏳) - Job is waiting to be executed
- **Running** (▶) - Job is currently executing
- **Success** (✓) - Job completed successfully
- **Failed** (✗) - Job failed during execution
- **Cancelled** (⊘) - Job was cancelled by user

## Exit Codes

- `0` - Success, command completed successfully
- `1` - General error (API error, network error, etc.)
- `2` - Command line parsing error

## Scripting Examples

### Monitor all running jobs

```bash
#!/bin/bash
while true; do
    clear
    raibid jobs list --status running
    sleep 5
done
```

### Wait for a job to complete

```bash
#!/bin/bash
JOB_ID=$1

while true; do
    STATUS=$(raibid jobs show "$JOB_ID" --json | jq -r '.status')
    if [[ "$STATUS" == "success" || "$STATUS" == "failed" || "$STATUS" == "cancelled" ]]; then
        echo "Job finished with status: $STATUS"
        exit 0
    fi
    sleep 2
done
```

### Trigger builds for multiple repos

```bash
#!/bin/bash
REPOS=("raibid-cli" "raibid-server" "raibid-tui")

for repo in "${REPOS[@]}"; do
    echo "Triggering build for $repo"
    raibid jobs trigger --repo "$repo" --branch main
done
```

### Export job data to CSV

```bash
raibid jobs list --json | jq -r '.jobs[] | [.id, .repo, .branch, .status, .duration] | @csv' > jobs.csv
```

## Troubleshooting

### Connection refused error

If you see "Failed to create API client" or "Connection refused":

1. Check that raibid-server is running
2. Verify the API URL is correct: `echo $RAIBID_API_URL`
3. Test connectivity: `curl http://localhost:8080/health`

### Invalid status filter

If filtering by status fails, ensure you use one of the valid status values (lowercase):
- pending
- running
- success
- failed
- cancelled

### Rate limiting

If you're making many requests, the API may rate limit you. Add delays between requests in scripts.

## See Also

