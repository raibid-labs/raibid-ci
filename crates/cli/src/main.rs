mod api;
mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::Cli;

fn main() -> Result<()> {
    // Initialize logging
    setup_logging()?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let _config = raibid_common::Config::load()?;

    // Handle commands
    match cli.command {
        None => {
            // No subcommand provided, show help
            println!("No command specified. Use --help for usage information.");
            std::process::exit(1);
        }
        Some(cli::Commands::Config(cmd)) => {
            // Handle config subcommands
            commands::config::handle(&cmd)
        }
        Some(cli::Commands::Tui) => {
            // Launch TUI dashboard
            raibid_tui::launch()
        }
        Some(cli::Commands::Init(cmd)) => {
            // Handle init command
            commands::init::execute(&cmd.command)
        }
        Some(cli::Commands::Health(cmd)) => {
            // Handle health command
            commands::health::execute(cmd.component, cmd.json)
        }
        Some(cli::Commands::Destroy(cmd)) => {
            // Handle destroy command
            commands::destroy::execute(cmd.component, cmd.yes, cmd.dry_run, cmd.force)
        }
        Some(cli::Commands::Setup { component }) => {
            // Handle setup command (deprecated, use init instead)
            let comp = component.parse()?;
            commands::setup::execute(comp)
        }
        Some(cli::Commands::Teardown { component }) => {
            // Handle teardown command (deprecated, use destroy instead)
            let comp = component.parse()?;
            commands::teardown::execute(comp)
        }
        Some(cli::Commands::Status { component }) => {
            // Handle status command (deprecated, use health instead)
            let comp = match component {
                Some(c) => Some(c.parse()?),
                None => None,
            };
            commands::status::execute(comp)
        }
        Some(cli::Commands::Jobs(cmd)) => {
            // Handle jobs subcommands
            commands::jobs::handle(&cmd)
        }
        Some(cli::Commands::Mirror(cmd)) => {
            // Handle mirror subcommands (async)
            tokio::runtime::Runtime::new()?.block_on(async { commands::mirror::handle(&cmd).await })
        }
    }
}

fn setup_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    Ok(())
}
