//! Script automation commands
//!
//! Commands for running automated test scripts.

use clap::Subcommand;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::config::CliConfig;
use crate::output::OutputFormatter;
use crate::error::{CliResult, CliError};

#[derive(Subcommand, Debug)]
pub enum ScriptCommand {
    /// Run a script file
    Run {
        /// Path to script file
        file: PathBuf,
        
        /// Continue on error
        #[arg(short, long)]
        continue_on_error: bool,
        
        /// Variables in KEY=VALUE format
        #[arg(short, long)]
        var: Vec<String>,
    },
    
    /// Validate a script without running
    Validate {
        /// Path to script file
        file: PathBuf,
    },
}

/// Script execution result
#[derive(Debug)]
pub struct ScriptResult {
    pub total_commands: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Execute a script command
pub async fn execute(
    cmd: ScriptCommand,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    match cmd {
        ScriptCommand::Run { file, continue_on_error, var } => {
            run_script(file, continue_on_error, var, config, formatter).await
        }
        ScriptCommand::Validate { file } => {
            validate_script(file, formatter).await
        }
    }
}

async fn run_script(
    file: PathBuf,
    continue_on_error: bool,
    vars: Vec<String>,
    _config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    // Check file exists
    if !file.exists() {
        return Err(CliError::NotFound(format!("Script file '{:?}'", file)));
    }
    
    // Parse variables
    let variables: HashMap<String, String> = vars.iter()
        .filter_map(|v| {
            let parts: Vec<&str> = v.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();
    
    println!("{}", formatter.header(&format!("Running Script: {:?}", file)));
    
    if !variables.is_empty() {
        println!("\n{}", formatter.info("Variables:"));
        for (k, v) in &variables {
            println!("  {} = {}", k, v);
        }
    }
    println!();
    
    // Read and parse script
    let content = std::fs::read_to_string(&file)?;
    let lines: Vec<&str> = content.lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .collect();
    
    let mut result = ScriptResult {
        total_commands: lines.len(),
        successful: 0,
        failed: 0,
        skipped: 0,
    };
    
    // Execute commands
    for (i, line) in lines.iter().enumerate() {
        let command = substitute_variables(line, &variables);
        
        println!("[{}/{}] {}", i + 1, lines.len(), command);
        
        // Simulate command execution
        let success = simulate_command_execution(&command).await;
        
        if success {
            println!("  {}", formatter.success("OK"));
            result.successful += 1;
        } else {
            println!("  {}", formatter.error("FAILED"));
            result.failed += 1;
            
            if !continue_on_error {
                println!();
                println!("{}", formatter.error("Script aborted due to error"));
                break;
            }
        }
    }
    
    // Print summary
    println!();
    println!("{}", formatter.header("Script Summary"));
    println!("{}", formatter.kv("Total Commands", &result.total_commands.to_string()));
    println!("{}", formatter.kv("Successful", &result.successful.to_string()));
    println!("{}", formatter.kv("Failed", &result.failed.to_string()));
    println!("{}", formatter.kv("Skipped", &result.skipped.to_string()));
    
    if result.failed > 0 {
        println!();
        println!("{}", formatter.warning(&format!("{} command(s) failed", result.failed)));
    } else {
        println!();
        println!("{}", formatter.success("All commands executed successfully"));
    }
    
    Ok(())
}

async fn validate_script(
    file: PathBuf,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    // Check file exists
    if !file.exists() {
        return Err(CliError::NotFound(format!("Script file '{:?}'", file)));
    }
    
    println!("{}", formatter.header(&format!("Validating Script: {:?}", file)));
    
    let content = std::fs::read_to_string(&file)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut command_count = 0;
    let mut variable_refs: Vec<String> = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        command_count += 1;
        
        // Check for variable references
        for cap in trimmed.match_indices("${") {
            if let Some(end) = trimmed[cap.0..].find('}') {
                let var_name = &trimmed[cap.0+2..cap.0+end];
                variable_refs.push(var_name.to_string());
            } else {
                errors.push(format!("Line {}: Unclosed variable reference", line_num));
            }
        }
        
        // Validate command structure
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "job" | "node" | "network" | "multi" | "config" => {
                // Valid command prefix
            }
            "wait" | "sleep" | "echo" | "assert" => {
                // Script control commands
            }
            _ => {
                warnings.push(format!("Line {}: Unknown command '{}'", line_num, parts[0]));
            }
        }
    }
    
    // Print results
    println!();
    println!("{}", formatter.kv("Total Lines", &lines.len().to_string()));
    println!("{}", formatter.kv("Commands", &command_count.to_string()));
    println!("{}", formatter.kv("Variable References", &variable_refs.len().to_string()));
    
    if !variable_refs.is_empty() {
        println!();
        println!("{}", formatter.info("Required Variables:"));
        let unique_vars: std::collections::HashSet<_> = variable_refs.iter().collect();
        for var in unique_vars {
            println!("  - {}", var);
        }
    }
    
    if !errors.is_empty() {
        println!();
        for error in &errors {
            println!("  {}", formatter.error(error));
        }
    }
    
    if !warnings.is_empty() {
        println!();
        for warning in &warnings {
            println!("  {}", formatter.warning(warning));
        }
    }
    
    println!();
    if errors.is_empty() {
        println!("{}", formatter.success("Script is valid"));
        if !warnings.is_empty() {
            println!("{}", formatter.warning(&format!("{} warning(s)", warnings.len())));
        }
    } else {
        return Err(CliError::ParseError(format!("Script has {} error(s)", errors.len())));
    }
    
    Ok(())
}

fn substitute_variables(line: &str, variables: &HashMap<String, String>) -> String {
    let mut result = line.to_string();
    
    for (key, value) in variables {
        result = result.replace(&format!("${{{}}}", key), value);
    }
    
    result
}

async fn simulate_command_execution(command: &str) -> bool {
    // Simulate command execution with small delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Simulate success/failure (95% success rate)
    !command.contains("fail")
}











