//! raibid-server binary
//!
//! API server for job dispatching and agent management.

use raibid_server::{Server, ServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables
    let config = ServerConfig::from_env();

    // Create and run the server
    let server = Server::new(config);
    server.run().await
}
