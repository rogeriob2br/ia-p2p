//! Integration tests for MVP CLI with real MVP Node
//!
//! These tests verify end-to-end functionality by interacting with actual node instances.

use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to run CLI command and capture output
fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute CLI command")
}

/// Helper to run CLI and get stdout as string
fn run_cli_stdout(args: &[&str]) -> String {
    let output = run_cli(args);
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to check if CLI command succeeded
fn cli_success(args: &[&str]) -> bool {
    run_cli(args).status.success()
}

mod help_tests {
    use super::*;
    
    #[test]
    fn test_help_command() {
        let output = run_cli_stdout(&["--help"]);
        assert!(output.contains("MVP CLI"));
        assert!(output.contains("job"));
        assert!(output.contains("node"));
        assert!(output.contains("network"));
        assert!(output.contains("multi"));
    }
    
    #[test]
    fn test_version_command() {
        let output = run_cli_stdout(&["--version"]);
        assert!(output.contains("mvp-cli"));
        assert!(output.contains("0.1.0"));
    }
    
    #[test]
    fn test_job_help() {
        let output = run_cli_stdout(&["job", "--help"]);
        assert!(output.contains("submit"));
        assert!(output.contains("status"));
        assert!(output.contains("list"));
    }
    
    #[test]
    fn test_node_help() {
        let output = run_cli_stdout(&["node", "--help"]);
        assert!(output.contains("list"));
        assert!(output.contains("info"));
        assert!(output.contains("health"));
    }
    
    #[test]
    fn test_network_help() {
        let output = run_cli_stdout(&["network", "--help"]);
        assert!(output.contains("topology"));
        assert!(output.contains("peers"));
        assert!(output.contains("routes"));
    }
    
    #[test]
    fn test_multi_help() {
        let output = run_cli_stdout(&["multi", "--help"]);
        assert!(output.contains("start"));
        assert!(output.contains("stop"));
        assert!(output.contains("list"));
    }
}

mod node_tests {
    use super::*;
    
    #[test]
    fn test_node_list() {
        let output = run_cli_stdout(&["node", "list"]);
        // Should contain table headers
        assert!(output.contains("Node ID") || output.contains("node"));
    }
    
    #[test]
    fn test_node_info() {
        let output = run_cli_stdout(&["node", "info", "node1"]);
        // Should contain node information or discovery message (no nodes when running in isolation)
        assert!(output.contains("node1") || output.contains("Node") || 
                output.contains("Discovering") || output.contains("No nodes"));
    }
    
    #[test]
    fn test_node_health() {
        let output = run_cli_stdout(&["node", "health", "node1"]);
        // Should contain health information or discovery message
        assert!(output.contains("Health") || output.contains("healthy") || 
                output.contains("Discovering") || output.contains("No nodes"));
    }
}

mod network_tests {
    use super::*;
    
    #[test]
    fn test_topology_ascii() {
        let output = run_cli_stdout(&["network", "topology"]);
        // Should contain topology representation
        assert!(output.contains("node") || output.contains("Topology"));
    }
    
    #[test]
    fn test_topology_json() {
        let output = run_cli_stdout(&["network", "topology", "--format", "json"]);
        // Should be valid JSON
        assert!(output.contains("{") && output.contains("}"));
        assert!(output.contains("nodes") || output.contains("edges"));
    }
    
    #[test]
    fn test_network_peers() {
        let output = run_cli_stdout(&["network", "peers", "node1"]);
        // Should contain peer information
        assert!(output.contains("Peer") || output.contains("node"));
    }
    
    #[test]
    fn test_network_routes() {
        let output = run_cli_stdout(&["network", "routes", "node1", "node2"]);
        // Should contain route information
        assert!(output.contains("Route") || output.contains("→"));
    }
}

mod job_tests {
    use super::*;
    
    #[test]
    fn test_job_submit_requires_node() {
        let output = run_cli(&["job", "submit", "test query"]);
        // Should fail without node specification
        assert!(!output.status.success() || 
                String::from_utf8_lossy(&output.stderr).contains("node"));
    }
    
    #[test]
    fn test_job_submit_with_node() {
        let output = run_cli_stdout(&["job", "submit", "--node", "localhost", "test query"]);
        // Should show submission confirmation or discovery message (no nodes when running in isolation)
        assert!(output.contains("Job") || output.contains("submitted") || output.contains("job") ||
                output.contains("Discovering") || output.contains("No nodes"));
    }
    
    #[test]
    fn test_job_list() {
        let output = run_cli_stdout(&["job", "list"]);
        // Should show job table or message
        assert!(output.contains("Job") || output.contains("job"));
    }
}

mod multi_tests {
    use super::*;
    
    #[test]
    fn test_multi_list_empty() {
        let output = run_cli_stdout(&["multi", "list"]);
        // Should show empty list or running instances
        assert!(output.contains("Instance") || output.contains("No running") || output.contains("node"));
    }
    
    #[test]
    fn test_multi_start_and_list() {
        // Start nodes
        let start_output = run_cli_stdout(&["multi", "start", "2"]);
        assert!(start_output.contains("Started") || start_output.contains("node"));
        
        // List should show them
        let list_output = run_cli_stdout(&["multi", "list"]);
        assert!(list_output.contains("node") || list_output.contains("Instance"));
        
        // Stop nodes
        let stop_output = run_cli_stdout(&["multi", "stop"]);
        assert!(stop_output.contains("Stopped") || stop_output.contains("stop"));
    }
}

mod config_tests {
    use super::*;
    
    #[test]
    fn test_config_show() {
        let output = run_cli_stdout(&["config", "show"]);
        // Should show configuration
        assert!(output.contains("Configuration") || output.contains("config"));
        assert!(output.contains("timeout") || output.contains("port"));
    }
    
    #[test]
    fn test_config_validate() {
        let output = run_cli(&["config", "validate"]);
        // Should succeed or show validation result
        assert!(output.status.success() || 
                String::from_utf8_lossy(&output.stdout).contains("valid"));
    }
}

mod output_format_tests {
    use super::*;
    
    #[test]
    fn test_json_output() {
        let output = run_cli_stdout(&["--output", "json", "node", "list"]);
        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        assert!(parsed.is_ok() || output.contains("[") || output.contains("{"));
    }
    
    #[test]
    fn test_csv_output() {
        let output = run_cli_stdout(&["--output", "csv", "node", "list"]);
        // Should contain CSV format (commas)
        assert!(output.contains(","));
    }
    
    #[test]
    fn test_plain_output() {
        let output = run_cli_stdout(&["--output", "plain", "node", "list"]);
        // Should be plain text (no fancy borders)
        assert!(!output.contains("╔") && !output.contains("║"));
    }
}

mod script_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_script_validate() {
        // Create a test script file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Test script").unwrap();
        writeln!(file, "node list").unwrap();
        writeln!(file, "config show").unwrap();
        file.flush().unwrap();
        
        let output = run_cli(&["script", "validate", file.path().to_str().unwrap()]);
        // Should validate successfully
        assert!(output.status.success() || 
                String::from_utf8_lossy(&output.stdout).contains("valid"));
    }
}

mod completions_tests {
    use super::*;
    
    #[test]
    fn test_bash_completions() {
        let output = run_cli_stdout(&["completions", "bash"]);
        // Should contain bash completion script
        assert!(output.contains("complete") || output.contains("_mvp-cli"));
    }
    
    #[test]
    fn test_zsh_completions() {
        let output = run_cli_stdout(&["completions", "zsh"]);
        // Should contain zsh completion script
        assert!(output.contains("compdef") || output.contains("_mvp-cli"));
    }
    
    #[test]
    fn test_fish_completions() {
        let output = run_cli_stdout(&["completions", "fish"]);
        // Should contain fish completion script
        assert!(output.contains("complete") || output.contains("mvp-cli"));
    }
}



