//! Common test utilities for raibid-server integration tests

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize tracing for tests with reduced verbosity to avoid log noise.
///
/// This function configures tracing to suppress expected ERROR logs from
/// tower_http when tests intentionally trigger error responses (404, 500).
/// It should be called at the beginning of each integration test.
///
/// The function uses `Once` to ensure tracing is only initialized once,
/// even when called from multiple tests running in parallel.
pub fn init_test_tracing() {
    INIT.call_once(|| {
        use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

        // Configure filter to suppress noisy logs in tests
        let env_filter = EnvFilter::new("warn")
            // Suppress tower_http trace logs that generate ERROR messages for expected failures
            .add_directive("tower_http::trace=off".parse().unwrap())
            // Allow error-level logs from the application itself
            .add_directive("raibid_server=error".parse().unwrap());

        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_test_writer())
            .try_init();
    });
}
