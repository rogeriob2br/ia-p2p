//! Configuration management commands
//!
//! Commands for viewing and managing CLI configuration.

use clap::Subcommand;
use std::path::PathBuf;
use crate::config::CliConfig;
use crate::output::OutputFormatter;
use crate::error::{CliResult, CliError};

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Show current configuration
    Show {
        /// Show only a specific key
        #[arg(short, long)]
        key: Option<String>,
    },
    
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Validate current configuration
    Validate,
    
    /// Initialize a new config file
    Init {
        /// Path for config file
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Overwrite existing file
        #[arg(short, long)]
        force: bool,
    },
}

/// Execute a config command
pub async fn execute(
    cmd: ConfigCommand,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    match cmd {
        ConfigCommand::Show { key } => {
            show_config(key, config, formatter).await
        }
        ConfigCommand::Set { key, value } => {
            set_config(key, value, config, formatter).await
        }
        ConfigCommand::Validate => {
            validate_config(config, formatter).await
        }
        ConfigCommand::Init { path, force } => {
            init_config(path, force, formatter).await
        }
    }
}

async fn show_config(
    key: Option<String>,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.header("Configuration"));
    
    let entries = config.as_display_table();
    
    let rows: Vec<Vec<String>> = entries.iter()
        .filter(|(k, _, _)| {
            key.as_ref().map_or(true, |filter| k.contains(filter))
        })
        .map(|(k, v, source)| vec![k.clone(), v.clone(), source.clone()])
        .collect();
    
    if rows.is_empty() {
        println!("{}", formatter.warning("No matching configuration found"));
        return Ok(());
    }
    
    println!("{}", formatter.table(
        vec!["Key", "Value", "Source"],
        rows
    ));
    
    println!();
    println!("{}", formatter.info("Sources: config = file, env = environment, flag = CLI"));
    
    Ok(())
}

async fn set_config(
    key: String,
    value: String,
    _config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    // Get config file path
    let config_path = dirs::config_dir()
        .map(|p| p.join("mvp-cli").join("config.toml"))
        .ok_or_else(|| CliError::ConfigError("Could not determine config directory".to_string()))?;
    
    // Load existing config or create new
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content)
            .map_err(|e| CliError::ConfigError(format!("Failed to parse config: {}", e)))?
    } else {
        CliConfig::default()
    };
    
    // Set the value
    match key.as_str() {
        "default_node" => config.default_node = Some(value.clone()),
        "default_model" => config.default_model = Some(value.clone()),
        "timeout_secs" => {
            config.timeout_secs = value.parse()
                .map_err(|_| CliError::ConfigError("Invalid timeout value".to_string()))?;
        }
        "retry_attempts" => {
            config.retry_attempts = value.parse()
                .map_err(|_| CliError::ConfigError("Invalid retry value".to_string()))?;
        }
        "connection.default_port" => {
            config.connection.default_port = value.parse()
                .map_err(|_| CliError::ConfigError("Invalid port value".to_string()))?;
        }
        "multi_node.base_port" => {
            config.multi_node.base_port = value.parse()
                .map_err(|_| CliError::ConfigError("Invalid port value".to_string()))?;
        }
        _ => {
            return Err(CliError::ConfigError(format!("Unknown config key: {}", key)));
        }
    }
    
    // Save config
    config.save(&config_path)?;
    
    println!("{}", formatter.success(&format!("Set {} = {}", key, value)));
    println!("{}", formatter.info(&format!("Saved to {:?}", config_path)));
    
    Ok(())
}

async fn validate_config(
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.header("Configuration Validation"));
    
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    // Validate timeout
    if config.timeout_secs == 0 {
        errors.push("timeout_secs cannot be 0");
    } else if config.timeout_secs > 3600 {
        warnings.push("timeout_secs is very high (>1 hour)");
    }
    
    // Validate port
    if config.connection.default_port < 1024 {
        warnings.push("connection.default_port is a privileged port (<1024)");
    }
    
    // Validate multi-node settings
    if config.multi_node.port_range < 10 {
        warnings.push("multi_node.port_range is very small (<10)");
    }
    
    // Display results
    if errors.is_empty() && warnings.is_empty() {
        println!("{}", formatter.success("Configuration is valid"));
    } else {
        for error in &errors {
            println!("  {} {}", formatter.error("ERROR:"), error);
        }
        for warning in &warnings {
            println!("  {} {}", formatter.warning("WARN:"), warning);
        }
        
        println!();
        if !errors.is_empty() {
            return Err(CliError::ConfigError("Configuration has errors".to_string()));
        }
        println!("{}", formatter.warning("Configuration valid with warnings"));
    }
    
    Ok(())
}

async fn init_config(
    path: Option<PathBuf>,
    force: bool,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    let config_path = path.unwrap_or_else(|| {
        dirs::config_dir()
            .map(|p| p.join("mvp-cli").join("config.toml"))
            .expect("Could not determine config directory")
    });
    
    if config_path.exists() && !force {
        return Err(CliError::ConfigError(
            format!("Config file already exists at {:?}. Use --force to overwrite.", config_path)
        ));
    }
    
    // Create default config
    CliConfig::generate_default(&config_path)?;
    
    println!("{}", formatter.success(&format!("Created config file at {:?}", config_path)));
    println!();
    println!("{}", formatter.info("Edit this file to customize settings"));
    println!("{}", formatter.info("Run 'mvp-cli config show' to view current settings"));
    
    Ok(())
}











