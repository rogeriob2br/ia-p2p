//! Configuration management for MVP CLI
//!
//! Handles loading configuration from:
//! 1. Config file (~/.config/mvp-cli/config.toml)
//! 2. Environment variables (MVP_CLI_* prefix)
//! 3. Command-line arguments (highest priority)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::{CliError, CliResult};

/// CLI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default node to connect to
    #[serde(default)]
    pub default_node: Option<String>,
    
    /// Default model for inference
    #[serde(default)]
    pub default_model: Option<String>,
    
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// Number of retry attempts
    #[serde(default = "default_retry")]
    pub retry_attempts: u32,
    
    /// Connection settings
    #[serde(default)]
    pub connection: ConnectionConfig,
    
    /// Multi-node settings
    #[serde(default)]
    pub multi_node: MultiNodeConfig,
    
    /// Interactive mode settings
    #[serde(default)]
    pub interactive: InteractiveConfig,
    
    /// Output settings
    #[serde(default)]
    pub output: OutputConfig,
}

fn default_timeout() -> u64 { 30 }
fn default_retry() -> u32 { 3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Default port for node connections
    #[serde(default = "default_port")]
    pub default_port: u16,
    
    /// Connection timeout in seconds
    #[serde(default = "default_conn_timeout")]
    pub connection_timeout_secs: u64,
    
    /// Maximum concurrent connections
    #[serde(default = "default_max_conn")]
    pub max_connections: usize,
}

fn default_port() -> u16 { 4001 }
fn default_conn_timeout() -> u64 { 10 }
fn default_max_conn() -> usize { 10 }

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            default_port: default_port(),
            connection_timeout_secs: default_conn_timeout(),
            max_connections: default_max_conn(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiNodeConfig {
    /// Base port for multi-node instances
    #[serde(default = "default_base_port")]
    pub base_port: u16,
    
    /// Port range for instances
    #[serde(default = "default_port_range")]
    pub port_range: u16,
    
    /// Directory for node logs
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    
    /// Path to mvp-node binary
    #[serde(default)]
    pub node_binary: Option<String>,
}

fn default_base_port() -> u16 { 5000 }
fn default_port_range() -> u16 { 100 }
fn default_log_dir() -> String { "./logs".to_string() }

impl Default for MultiNodeConfig {
    fn default() -> Self {
        Self {
            base_port: default_base_port(),
            port_range: default_port_range(),
            log_dir: default_log_dir(),
            node_binary: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveConfig {
    /// Path to history file
    #[serde(default = "default_history_file")]
    pub history_file: String,
    
    /// Maximum history entries
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    
    /// Enable auto-complete
    #[serde(default = "default_true")]
    pub auto_complete_enabled: bool,
    
    /// Prompt string
    #[serde(default = "default_prompt")]
    pub prompt: String,
}

fn default_history_file() -> String { "~/.mvp_cli_history".to_string() }
fn default_history_size() -> usize { 1000 }
fn default_true() -> bool { true }
fn default_prompt() -> String { "mvp> ".to_string() }

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            history_file: default_history_file(),
            history_size: default_history_size(),
            auto_complete_enabled: default_true(),
            prompt: default_prompt(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Table style (modern, ascii, minimal)
    #[serde(default = "default_table_style")]
    pub table_style: String,
    
    /// Date format
    #[serde(default = "default_date_format")]
    pub date_format: String,
    
    /// Truncate long output
    #[serde(default = "default_true")]
    pub truncate_long_output: bool,
    
    /// Maximum output lines
    #[serde(default = "default_max_lines")]
    pub max_output_lines: usize,
}

fn default_table_style() -> String { "modern".to_string() }
fn default_date_format() -> String { "%Y-%m-%d %H:%M:%S".to_string() }
fn default_max_lines() -> usize { 100 }

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            table_style: default_table_style(),
            date_format: default_date_format(),
            truncate_long_output: default_true(),
            max_output_lines: default_max_lines(),
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_node: None,
            default_model: Some("tinyllama".to_string()),
            timeout_secs: default_timeout(),
            retry_attempts: default_retry(),
            connection: ConnectionConfig::default(),
            multi_node: MultiNodeConfig::default(),
            interactive: InteractiveConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl CliConfig {
    /// Load configuration from file and environment
    pub fn load(config_path: Option<&Path>) -> CliResult<Self> {
        let mut config = Self::default();
        
        // Try to load from file
        let path = config_path
            .map(PathBuf::from)
            .or_else(Self::default_config_path);
        
        if let Some(ref p) = path {
            if p.exists() {
                let content = std::fs::read_to_string(p)?;
                config = toml::from_str(&content)
                    .map_err(|e| CliError::ConfigError(format!("Failed to parse config: {}", e)))?;
            }
        }
        
        // Apply environment variable overrides
        config.apply_env_overrides();
        
        Ok(config)
    }
    
    /// Get default config path
    fn default_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("mvp-cli").join("config.toml"))
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        if let Ok(node) = std::env::var("MVP_CLI_DEFAULT_NODE") {
            self.default_node = Some(node);
        }
        
        if let Ok(model) = std::env::var("MVP_CLI_DEFAULT_MODEL") {
            self.default_model = Some(model);
        }
        
        if let Ok(timeout) = std::env::var("MVP_CLI_TIMEOUT") {
            if let Ok(t) = timeout.parse() {
                self.timeout_secs = t;
            }
        }
        
        if let Ok(port) = std::env::var("MVP_CLI_DEFAULT_PORT") {
            if let Ok(p) = port.parse() {
                self.connection.default_port = p;
            }
        }
    }
    
    /// Save configuration to file
    pub fn save(&self, path: &Path) -> CliResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Generate default config file
    pub fn generate_default(path: &Path) -> CliResult<()> {
        let config = Self::default();
        config.save(path)
    }
    
    /// Get configuration as displayable table
    pub fn as_display_table(&self) -> Vec<(String, String, String)> {
        vec![
            ("default_node".to_string(), 
             self.default_node.clone().unwrap_or_default(), 
             "config".to_string()),
            ("default_model".to_string(), 
             self.default_model.clone().unwrap_or_default(), 
             "config".to_string()),
            ("timeout_secs".to_string(), 
             self.timeout_secs.to_string(), 
             "config".to_string()),
            ("retry_attempts".to_string(), 
             self.retry_attempts.to_string(), 
             "config".to_string()),
            ("connection.default_port".to_string(), 
             self.connection.default_port.to_string(), 
             "config".to_string()),
            ("multi_node.base_port".to_string(), 
             self.multi_node.base_port.to_string(), 
             "config".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.connection.default_port, 4001);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = CliConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("timeout_secs"));
        
        let parsed: CliConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.timeout_secs, config.timeout_secs);
    }
}











