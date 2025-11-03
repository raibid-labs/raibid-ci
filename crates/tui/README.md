# Raibid CI TUI

Terminal User Interface for raibid-ci using Ratatui.

## Features

### Completed (Issue #69)
- **Standalone TUI Crate**: Fully extracted into `/home/beengud/raibid-labs/raibid-ci/crates/tui/`
- **Modular Architecture**:
  - `app.rs`: Application state and event loop
  - `events.rs`: Keyboard and terminal event handling
  - `terminal.rs`: Terminal initialization and cleanup
  - `ui.rs`: Rendering logic for dashboard widgets
  - `mock_data.rs`: Mock data generators for testing
  - `api_client.rs`: HTTP client for real API integration
- **Event Loop**: 1-second refresh rate with proper tick handling
- **Terminal Management**: Proper init/cleanup, no corruption on errors
- **Keyboard Support**: Quit on 'q' or Ctrl+C

### Dashboard Widgets (Issue #70)
- **3-Panel Layout**: Jobs (60%), Agents (20%), Queue (20%)
- **Jobs Table Widget**:
  - Color-coded status indicators
  - Progress bars
  - Duration tracking
  - Filter and search support
- **Agents List Widget**:
  - Status display (Idle/Busy/Starting/Stopping)
  - CPU and memory usage bars
  - Uptime tracking
- **Queue Depth Sparkline**:
  - 60-second history visualization
  - Current/max/average depth display
- **API Client Module**:
  - Async HTTP client for server communication
  - Job listing with filters (status, repo, branch)
  - Individual job fetching
  - Job cancellation support
  - Job triggering support
  - Health check endpoint

### Interactive Features (Issue #71)
- **Job Detail View**: Press Enter to view detailed job information
- **Filter Menu**: Press 'f' to filter jobs by status
- **Search**: Press '/' to search jobs by repo/branch/ID
- **Help Screen**: Press '?' for keyboard shortcuts
- **Confirmation Dialogs**: For dangerous actions like job cancellation
- **Tab Navigation**: Switch between Jobs/Agents/Config/Logs tabs

## Running the TUI

Launch via CLI:
```bash
cargo run --bin raibid -- tui
```

## Keyboard Shortcuts

### Navigation
- `Tab` / `←` / `→`: Switch between tabs
- `1` / `2` / `3` / `4`: Jump directly to tab (Jobs/Agents/Config/Logs)
- `↑` / `↓`: Navigate items in lists

### Actions
- `Enter`: View job details (on Jobs tab)
- `c`: Cancel selected job
- `r`: Refresh data
- `f`: Filter jobs by status
- `/`: Search jobs (by repo/branch/ID)
- `m`: Toggle between mock and API data (when API available)
- `Esc`: Close popup / Clear filters
- `?`: Show help screen
- `q` / `Ctrl+C`: Quit application

### In Popups
- `Esc`: Close popup
- `y` / `n`: Confirm/cancel action in confirmation dialogs

## Architecture

### Module Structure
```
crates/tui/
├── src/
│   ├── lib.rs           # Public API and launch function
│   ├── app.rs           # Application state and event loop
│   ├── events.rs        # Event handling (keyboard, resize, tick)
│   ├── terminal.rs      # Terminal initialization and cleanup
│   ├── ui.rs            # Rendering logic for all widgets
│   ├── mock_data.rs     # Mock data generators
│   └── api_client.rs    # HTTP API client for real data
├── Cargo.toml
└── README.md
```

### Data Flow
1. **Event Loop**: `EventHandler` polls for keyboard events and generates tick events
2. **State Updates**: On tick, `App` refreshes data (from mock or API)
3. **Rendering**: `ui::render()` draws all widgets based on current state
4. **Actions**: User input triggers state changes and API calls

### API Integration (Ready for Use)

The `ApiClient` module provides async HTTP methods:
```rust
use raibid_tui::{ApiClient, ApiConfig};

// Create client
let client = ApiClient::new()?;

// List jobs
let jobs = client.list_jobs(
    Some(JobStatus::Running), // Filter by status
    None,                      // No repo filter
    None,                      // No branch filter
    Some(50),                  // Limit to 50 jobs
).await?;

// Get specific job
let job = client.get_job("job-1234").await?;

// Cancel job
client.cancel_job("job-1234").await?;

// Trigger new job
let job = client.trigger_job(
    "raibid-ci".to_string(),
    "main".to_string(),
).await?;
```

### Configuration

Set API URL via environment variable:
```bash
export RAIBID_API_URL=http://localhost:8080
cargo run --bin raibid -- tui
```

Default: `http://localhost:8080`

## Testing

Run tests:
```bash
cargo test --package raibid-tui
```

Test coverage includes:
- App creation and configuration
- Event handling (keyboard, tick, resize)
- State management (jobs, agents, queue data)
- UI state transitions
- Terminal size validation

## Performance

- **Minimum Terminal Size**: 80x24 characters
- **Refresh Rate**: 1 second (configurable)
- **Target Performance**: Smooth rendering with 100+ jobs
- **Memory**: Efficient state management with pagination support

## Future Enhancements

1. **Real-time Log Streaming**: SSE connection for job logs
2. **Agent Management**: Start/stop agents from TUI
3. **Build Triggers**: Create new jobs interactively
4. **Config Editing**: Modify configuration from TUI
5. **Metrics Graphs**: Enhanced visualizations
6. **Multi-page Support**: Pagination for large job lists
7. **Async State Management**: Non-blocking API calls

## Dependencies

- **ratatui**: Terminal UI framework (v0.25)
- **crossterm**: Cross-platform terminal manipulation (v0.27)
- **tokio**: Async runtime (v1)
- **reqwest**: HTTP client (v0.11)
- **serde**: Serialization framework (v1)
- **chrono**: Date and time handling (v0.4)
- **rand**: Random number generation for mock data (v0.8)

## Contributing

When modifying the TUI:
1. Keep UI logic in `ui.rs`
2. Keep state management in `app.rs`
3. Add new widgets as separate functions in `ui.rs`
4. Update keyboard shortcuts in help screen
5. Maintain test coverage for state changes

## License

MIT OR Apache-2.0
