//! MVP CLI - Command Line Interface for Decentralized AI Network
//!
//! This CLI provides commands for interacting with MVP Nodes:
//! - Submit and track inference jobs
//! - Monitor node status and health
//! - Explore network topology
//! - Manage multiple node instances for testing
//!
//! ## Usage
//!
//! ```bash
//! # Submit a job
//! mvp-cli job submit --node node1 "What is AI?"
//!
//! # List nodes
//! mvp-cli node list
//!
//! # Start multiple nodes for testing
//! mvp-cli multi start 3
//!
//! # Interactive mode
//! mvp-cli interactive
//! ```

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tracing::{info, error};

mod commands;
mod client;
mod config;
mod output;
mod repl;
mod error;

use config::CliConfig;
use error::CliResult;

/// MVP CLI - Decentralized AI Network Command Line Interface
#[derive(Parser)]
#[command(name = "mvp-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "RAIS Network")]
#[command(about = "Command Line Interface for MVP Node - Decentralized AI Network")]
#[command(long_about = r#"
MVP CLI provides a complete interface for interacting with MVP Nodes
in the decentralized AI network.

Features:
  - Submit inference jobs and track their status
  - Monitor nodes and their health
  - Explore network topology
  - Manage multiple node instances for testing
  - Interactive REPL mode

Configuration:
  Configuration can be provided via:
  1. Config file (~/.config/mvp-cli/config.toml)
  2. Environment variables (MVP_CLI_* prefix)
  3. Command-line flags (highest priority)

Examples:
  mvp-cli job submit --node node1 "What is the capital of France?"
  mvp-cli node list --watch
  mvp-cli multi start 3 --base-port 5000
  mvp-cli interactive
"#)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, env = "MVP_CLI_CONFIG")]
    config: Option<PathBuf>,
    
    /// Output format (table, json, csv, plain)
    #[arg(short, long, default_value = "table", env = "MVP_CLI_OUTPUT")]
    output: output::OutputFormat,
    
    /// Enable verbose output
    #[arg(short, long, env = "MVP_CLI_VERBOSE")]
    verbose: bool,
    
    /// Disable colored output
    #[arg(long, env = "MVP_CLI_NO_COLOR")]
    no_color: bool,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "MVP_CLI_LOG_LEVEL")]
    log_level: String,
    
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage inference jobs
    Job {
        #[command(subcommand)]
        action: commands::job::JobCommand,
    },
    
    /// Monitor and query nodes
    Node {
        #[command(subcommand)]
        action: commands::node::NodeCommand,
    },
    
    /// Explore network topology
    Network {
        #[command(subcommand)]
        action: commands::network::NetworkCommand,
    },
    
    /// Manage multiple node instances
    Multi {
        #[command(subcommand)]
        action: commands::multi::MultiCommand,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: commands::config::ConfigCommand,
    },
    
    /// Run automation scripts
    Script {
        #[command(subcommand)]
        action: commands::script::ScriptCommand,
    },
    
    /// Start interactive REPL mode
    Interactive,
    
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn init_logging(log_level: &str, verbose: bool) {
    use tracing_subscriber::EnvFilter;
    
    let level = if verbose { "debug" } else { log_level };
    
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();
}

fn print_banner() {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║        MVP CLI - Decentralized AI         ║".cyan());
    println!("{}", format!("║              Version {}               ║", env!("CARGO_PKG_VERSION")).cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();
}

#[tokio::main]
async fn main() -> CliResult<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(&cli.log_level, cli.verbose);
    
    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }
    
    // Load configuration
    let config = CliConfig::load(cli.config.as_deref())?;
    
    // Create output formatter
    let formatter = output::OutputFormatter::new(cli.output.clone(), !cli.no_color);
    
    // Execute command
    let result = match cli.command {
        Commands::Job { action } => {
            commands::job::execute(action, &config, &formatter).await
        }
        Commands::Node { action } => {
            commands::node::execute(action, &config, &formatter).await
        }
        Commands::Network { action } => {
            commands::network::execute(action, &config, &formatter).await
        }
        Commands::Multi { action } => {
            commands::multi::execute(action, &config, &formatter).await
        }
        Commands::Config { action } => {
            commands::config::execute(action, &config, &formatter).await
        }
        Commands::Script { action } => {
            commands::script::execute(action, &config, &formatter).await
        }
        Commands::Interactive => {
            print_banner();
            repl::start_repl(&config, &formatter).await
        }
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            use clap_complete::generate;
            use std::io;
            
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "mvp-cli", &mut io::stdout());
            Ok(())
        }
    };
    
    // Handle result
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            if cli.verbose {
                if let Some(source) = e.source() {
                    eprintln!("{}: {}", "Caused by".yellow(), source);
                }
            }
            std::process::exit(1);
        }
    }
}











