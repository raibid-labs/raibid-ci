# TUI Development Guide

## WS-07: TUI Enhancement Work Summary

### Completed Issues

#### Issue #69: Extract TUI Crate from Existing Code
**Status:** COMPLETE (Closed)
**Commit:** Previous work + d2a69c5

The TUI crate was already properly extracted in previous development sessions. This issue verified the implementation meets all requirements:

- Standalone crate at `/home/beengud/raibid-labs/raibid-ci/crates/tui/`
- Modular architecture with clear separation of concerns
- Ratatui + Crossterm + Tokio integration
- Event loop with 1-second refresh
- Proper terminal management (init/cleanup)
- Comprehensive test coverage
- Clean keyboard event handling

**Deliverables:**
- [x] TUI crate structure
- [x] Event handling system
- [x] Terminal initialization/cleanup
- [x] Tests for event handling
- [x] Documentation (README.md added)

---

#### Issue #70: Implement Real-time Dashboard Widgets
**Status:** IN PROGRESS (Foundation Complete)
**Commit:** d2a69c5

Created API client module to enable real-time data fetching from the server. The dashboard widgets already exist and render mock data beautifully.

**Completed:**
- [x] API client module (`api_client.rs`)
- [x] HTTP methods for job operations
- [x] Filtering and pagination support
- [x] Environment-based configuration
- [x] Comprehensive documentation
- [x] Dashboard widgets (already existed)
- [x] 3-panel layout
- [x] Color-coded status indicators
- [x] Responsive design

**Remaining Work:**
- [ ] Integrate API client into App state
- [ ] Implement data source toggle (mock vs API)
- [ ] Add connection status indicator
- [ ] Handle API errors gracefully
- [ ] Real-time data refresh from API

**Key Files:**
- `src/api_client.rs` - HTTP client implementation
- `src/app.rs` - Needs API integration
- `src/ui.rs` - Dashboard rendering (complete)

---

#### Issue #71: Add Interactive Job Management
**Status:** IN PROGRESS (Foundation Complete)
**Commit:** d2a69c5

Implemented API methods for all interactive job operations. The TUI already has interactive features working with mock data.

**Completed:**
- [x] API methods: cancel_job, trigger_job, get_job
- [x] Interactive UI features (with mock data)
- [x] Job detail popup (Enter key)
- [x] Filter menu (f key)
- [x] Search functionality (/ key)
- [x] Confirmation dialogs
- [x] Keyboard navigation
- [x] Help screen

**Remaining Work:**
- [ ] Connect cancel action to API
- [ ] Add manual job trigger UI (n key)
- [ ] Implement log streaming (l key) via SSE
- [ ] Real-time job status updates
- [ ] Server-side filtering/searching

**Key Files:**
- `src/api_client.rs` - Job action methods (complete)
- `src/app.rs` - Needs action integration
- `src/ui.rs` - UI components (complete)

---

## Architecture Overview

### Module Structure
```
crates/tui/
├── src/
│   ├── lib.rs           # Public API, launch function
│   ├── app.rs           # Application state, event loop
│   ├── events.rs        # Event handling (keyboard, resize, tick)
│   ├── terminal.rs      # Terminal init/cleanup
│   ├── ui.rs            # Rendering logic for widgets
│   ├── mock_data.rs     # Mock data generators
│   └── api_client.rs    # HTTP client for server API [NEW]
├── Cargo.toml
├── README.md            # User documentation [NEW]
└── DEVELOPMENT.md       # This file [NEW]
```

### Data Flow

**Current (Mock Data):**
```
EventHandler -> App.update() -> generate_mock_data() -> App.jobs/agents/queue
                                                              |
                                                              v
                                                        ui::render()
```

**Target (Real API):**
```
EventHandler -> App.update() -> ApiClient.list_jobs() -> App.jobs
                                                             |
                                                             v
                                                       ui::render()
```

### API Client Design

The `ApiClient` provides async methods for server communication:

```rust
pub struct ApiClient {
    config: ApiConfig,
    client: reqwest::Client,
}

impl ApiClient {
    // Job operations
    pub async fn list_jobs(...) -> Result<JobListResponse>
    pub async fn get_job(id) -> Result<Job>
    pub async fn cancel_job(id) -> Result<()>
    pub async fn trigger_job(repo, branch) -> Result<Job>

    // Agent operations (placeholder)
    pub async fn list_agents() -> Result<Vec<AgentInfo>>

    // Metrics (placeholder)
    pub async fn get_queue_metrics() -> Result<QueueMetrics>

    // Health
    pub async fn health_check() -> Result<HealthStatus>
}
```

### Configuration

API client reads from environment:
```bash
export RAIBID_API_URL=http://localhost:8080
```

Default: `http://localhost:8080`

---

## Integration Guide

To complete the remaining work, follow these steps:

### 1. Integrate API Client into App State

**File:** `src/app.rs`

Add fields to `App` struct:
```rust
pub struct App {
    // ... existing fields ...

    /// API client for real data
    api_client: Option<ApiClient>,

    /// Data source mode
    data_source: DataSource,

    /// API connection status
    api_connected: bool,

    /// Last API error
    api_error: Option<String>,
}

enum DataSource {
    Mock,
    Api,
}
```

### 2. Implement Data Source Toggle

Add to `AppConfig`:
```rust
pub struct AppConfig {
    // ... existing fields ...

    /// Use real API data
    pub use_real_api: bool,

    /// API configuration
    pub api_config: ApiConfig,
}
```

In `App::new()`:
```rust
let api_client = if config.use_real_api {
    match ApiClient::with_config(config.api_config.clone()) {
        Ok(client) => Some(client),
        Err(e) => {
            tracing::warn!("Failed to create API client: {}", e);
            None
        }
    }
} else {
    None
};
```

### 3. Fetch Real Data in Update Loop

Modify `App::update()`:
```rust
pub fn update(&mut self) {
    match self.data_source {
        DataSource::Mock => {
            // Existing mock data generation
            let (jobs, agents, _) = generate_mock_data(&self.mock_config);
            self.jobs = jobs;
            self.agents = agents;

            let mut rng = rand::thread_rng();
            self.queue_data.update(&mut rng);
        }
        DataSource::Api => {
            // Fetch from API
            if let Some(ref client) = self.api_client {
                // Use tokio::spawn or block_on for async call
                if let Ok(response) = tokio::runtime::Handle::current()
                    .block_on(client.list_jobs(None, None, None, Some(100)))
                {
                    self.api_connected = true;
                    self.api_error = None;
                    // Convert API jobs to display format
                    self.jobs = response.jobs
                        .into_iter()
                        .map(|j| convert_api_job(j))
                        .collect();
                } else {
                    self.api_connected = false;
                    self.api_error = Some("Failed to fetch jobs".to_string());
                }
            }
        }
    }
}
```

### 4. Add Keyboard Shortcut for Toggle

In `App::handle_event()`:
```rust
KeyCode::Char('m') => {
    // Toggle between mock and API
    self.data_source = match self.data_source {
        DataSource::Mock => DataSource::Api,
        DataSource::Api => DataSource::Mock,
    };
}
```

### 5. Display Connection Status

In `ui.rs` header:
```rust
let mode_indicator = match ui_state.data_source {
    DataSource::Mock => Span::styled(" [MOCK] ", Style::default().fg(Color::Yellow)),
    DataSource::Api if ui_state.api_connected => {
        Span::styled(" [API] ", Style::default().fg(Color::Green))
    }
    DataSource::Api => {
        Span::styled(" [API DISCONNECTED] ", Style::default().fg(Color::Red))
    }
};
```

### 6. Implement Real Job Cancellation

In `App::handle_event()` when 'c' is pressed:
```rust
KeyCode::Char('c') => {
    if let Some(job) = self.get_selected_job() {
        if let Some(ref client) = self.api_client {
            let job_id = job.id.clone();
            // Spawn async task
            tokio::spawn(async move {
                if let Err(e) = client.cancel_job(&job_id).await {
                    tracing::error!("Failed to cancel job: {}", e);
                }
            });
        }
    }
}
```

---

## Testing

### Unit Tests
```bash
cargo test --package raibid-tui
```

Current coverage:
- App creation and configuration
- Event handling
- State management
- Terminal size validation
- Mock data generation

### Integration Testing
```bash
# Start API server
cargo run --bin raibid-server

# In another terminal, start TUI with API
export RAIBID_API_URL=http://localhost:8080
cargo run --bin raibid -- tui

# Test scenarios:
# 1. Press 'm' to toggle to API mode
# 2. Verify jobs load from API
# 3. Press 'c' to cancel a job
# 4. Press 'r' to refresh
# 5. Press 'f' to filter jobs
# 6. Press '/' to search
```

### Manual Testing Checklist
- [ ] TUI launches without errors
- [ ] All tabs render correctly
- [ ] Keyboard shortcuts work
- [ ] Mock data displays properly
- [ ] API mode connects successfully
- [ ] Job cancellation works
- [ ] Filtering and search work
- [ ] Terminal resizes gracefully
- [ ] Quit (q) exits cleanly
- [ ] No terminal corruption on errors

---

## Performance Considerations

### Current Performance
- Refresh rate: 1 second
- Mock data: 25 jobs, 5 agents
- Memory: Minimal (stateless rendering)
- CPU: Low (event-driven)

### Scaling Targets
- Support 100+ jobs without lag
- Pagination for large job lists
- Efficient rendering (only changed widgets)
- Async API calls don't block UI

### Optimization Strategies
1. **Pagination**: Fetch jobs in batches
2. **Caching**: Cache API responses for 1 second
3. **Incremental Updates**: Only update changed jobs
4. **Virtual Scrolling**: Render only visible rows
5. **Async Tasks**: Use `tokio::spawn` for API calls

---

## Dependencies

### Current Dependencies
```toml
[dependencies]
raibid-common = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
colored = { workspace = true }
tokio = { workspace = true }
reqwest = { workspace = true }      # NEW
urlencoding = { workspace = true }   # NEW
rand = { workspace = true }
chrono = { workspace = true }
```

### Why These Dependencies?
- **reqwest**: Async HTTP client for API calls
- **urlencoding**: URL-encode query parameters
- **tokio**: Async runtime for non-blocking operations
- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal control
- **chrono**: Date/time handling for timestamps

---

## Future Enhancements

### Short Term (Next Sprint)
1. Complete API integration in app.rs
2. Add log streaming via SSE
3. Implement manual job triggering
4. Add agent management

### Medium Term
1. Multi-page job list with pagination
2. Enhanced search with regex
3. Job history and statistics
4. Configuration editor in TUI

### Long Term
1. Tauri GUI wrapper
2. Remote TUI via SSH
3. Distributed tracing view
4. Real-time collaboration features

---

## Troubleshooting

### TUI Won't Launch
```bash
# Check terminal size
tput cols && tput lines  # Should be at least 80x24

# Check dependencies
cargo check --package raibid-tui

# Run with logging
RUST_LOG=debug cargo run --bin raibid -- tui
```

### API Connection Fails
```bash
# Verify API server is running
curl http://localhost:8080/health

# Check API URL
echo $RAIBID_API_URL

# Test API client directly
cargo test --package raibid-tui -- --test-threads=1
```

### Terminal Corrupted After Crash
```bash
# Reset terminal
reset

# Or manually
stty sane
clear
```

---

## Contributing

When working on TUI code:

1. **Keep UI logic separate**: All rendering in `ui.rs`
2. **State in App**: All state management in `app.rs`
3. **Test coverage**: Add tests for new features
4. **Documentation**: Update README and this guide
5. **Keyboard shortcuts**: Update help screen
6. **Error handling**: Always handle API errors gracefully

### Code Style
- Use meaningful variable names
- Comment complex logic
- Keep functions small and focused
- Follow existing patterns
- Run `cargo fmt` and `cargo clippy`

---

## References

- **Ratatui Docs**: https://ratatui.rs
- **Crossterm Docs**: https://docs.rs/crossterm
- **Tokio Docs**: https://tokio.rs
- **Reqwest Docs**: https://docs.rs/reqwest

## Commit

**Commit d2a69c5**: feat(tui): Add API client module for real-time data integration

This commit provides the foundation for issues #70 and #71, with a complete API client ready for integration.
