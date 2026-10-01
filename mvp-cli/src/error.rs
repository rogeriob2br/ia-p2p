//! Error types for MVP CLI

use thiserror::Error;

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// CLI Error types
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Command error: {0}")]
    CommandError(String),
    
    #[error("Job error: {0}")]
    JobError(String),
    
    #[error("Node error: {0}")]
    NodeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Process error: {0}")]
    ProcessError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Query error [{code}]: {message}")]
    QueryError { message: String, code: String },
    
    #[error("No nodes discovered. Make sure nodes are running on the network.")]
    NoNodesDiscovered,
    
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            CliError::ConnectionError(_) | 
            CliError::Timeout(_) |
            CliError::NetworkError(_)
        )
    }
    
    /// Get suggested action for the error
    pub fn suggested_action(&self) -> Option<String> {
        match self {
            CliError::ConnectionError(_) => {
                Some("Check if the node is running and accessible".to_string())
            }
            CliError::ConfigError(_) => {
                Some("Run 'mvp-cli config show' to see current configuration".to_string())
            }
            CliError::NotFound(what) => {
                Some(format!("Verify that {} exists", what))
            }
            CliError::Timeout(_) => {
                Some("Try increasing the timeout with --timeout flag".to_string())
            }
            CliError::NoNodesDiscovered => {
                Some("Make sure mvp-node is running. Use 'mvp-node start' to start a node.".to_string())
            }
            CliError::NetworkError(_) => {
                Some("Check your network connection and firewall settings.".to_string())
            }
            CliError::QueryError { code, .. } if code == "NOT_FOUND" => {
                Some("The requested resource was not found on the node.".to_string())
            }
            _ => None,
        }
    }
    
    /// Get the source of the error if available
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::IoError(e) => Some(e),
            CliError::Other(e) => e.source(),
            _ => None,
        }
    }
}



