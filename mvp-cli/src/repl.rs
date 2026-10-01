//! Interactive REPL (Read-Eval-Print-Loop) for MVP CLI
//!
//! Provides an interactive shell with command history and auto-completion.

use rustyline::error::ReadlineError;
use rustyline::{Editor, Config, EditMode};
use rustyline::history::DefaultHistory;
use colored::Colorize;
use crate::config::CliConfig;
use crate::output::OutputFormatter;
use crate::error::{CliResult, CliError};

/// Start the interactive REPL
pub async fn start_repl(
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", "Type 'help' for commands, 'exit' to quit".dimmed());
    println!();
    
    // Setup rustyline
    let rl_config = Config::builder()
        .history_ignore_space(true)
        .edit_mode(EditMode::Emacs)
        .build();
    
    let mut rl: Editor<(), DefaultHistory> = Editor::with_config(rl_config)
        .map_err(|e| CliError::Other(anyhow::anyhow!("Failed to create editor: {}", e)))?;
    
    // Load history if exists
    let history_path = shellexpand::tilde(&config.interactive.history_file).to_string();
    let _ = rl.load_history(&history_path);
    
    // Main REPL loop
    loop {
        let prompt = format!("{} ", config.interactive.prompt.cyan());
        
        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                // Add to history
                let _ = rl.add_history_entry(input);
                
                // Handle special commands
                match input {
                    "exit" | "quit" | "q" => {
                        println!("{}", "Goodbye!".green());
                        break;
                    }
                    "help" | "?" => {
                        print_help(formatter);
                    }
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    "history" => {
                        print_history(&rl);
                    }
                    _ => {
                        // Execute command
                        if let Err(e) = execute_repl_command(input, config, formatter).await {
                            eprintln!("{}: {}", "Error".red(), e);
                        }
                    }
                }
                
                println!();
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".dimmed());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".green());
                break;
            }
            Err(err) => {
                eprintln!("{}: {:?}", "Error".red(), err);
                break;
            }
        }
    }
    
    // Save history
    let _ = rl.save_history(&history_path);
    
    Ok(())
}

fn print_help(formatter: &OutputFormatter) {
    println!("{}", formatter.header("Available Commands"));
    println!();
    
    println!("{}", "Job Commands:".bold());
    println!("  job submit <input>     Submit an inference job");
    println!("  job status <job_id>    Check job status");
    println!("  job list               List all jobs");
    println!("  job result <job_id>    Get job result");
    println!();
    
    println!("{}", "Node Commands:".bold());
    println!("  node list              List discovered nodes");
    println!("  node info <node_id>    Get node details");
    println!("  node health <node_id>  Check node health");
    println!("  node metrics <node_id> View node metrics");
    println!();
    
    println!("{}", "Network Commands:".bold());
    println!("  network topology       Show network topology");
    println!("  network peers <node>   List node peers");
    println!("  network traffic        Monitor traffic");
    println!();
    
    println!("{}", "Multi-Node Commands:".bold());
    println!("  multi start <count>    Start multiple nodes");
    println!("  multi stop [nodes]     Stop nodes");
    println!("  multi list             List running nodes");
    println!("  multi logs <node_id>   View node logs");
    println!();
    
    println!("{}", "Other Commands:".bold());
    println!("  config show            Show configuration");
    println!("  connect <node>         Connect to a node");
    println!("  clear                  Clear screen");
    println!("  history                Show command history");
    println!("  help                   Show this help");
    println!("  exit                   Exit REPL");
}

fn print_history(rl: &Editor<(), DefaultHistory>) {
    println!("Command History:");
    for (i, entry) in rl.history().iter().enumerate() {
        println!("  {}: {}", i + 1, entry);
    }
}

async fn execute_repl_command(
    input: &str,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    
    if parts.is_empty() {
        return Ok(());
    }
    
    match parts[0] {
        "job" => {
            execute_job_command(&parts[1..], config, formatter).await
        }
        "node" => {
            execute_node_command(&parts[1..], config, formatter).await
        }
        "network" => {
            execute_network_command(&parts[1..], config, formatter).await
        }
        "multi" => {
            execute_multi_command(&parts[1..], config, formatter).await
        }
        "config" => {
            execute_config_command(&parts[1..], config, formatter).await
        }
        "connect" => {
            if parts.len() < 2 {
                println!("{}", formatter.warning("Usage: connect <node>"));
            } else {
                println!("{}", formatter.success(&format!("Connected to {}", parts[1])));
            }
            Ok(())
        }
        _ => {
            println!("{}", formatter.warning(&format!("Unknown command: {}. Type 'help' for available commands.", parts[0])));
            Ok(())
        }
    }
}

async fn execute_job_command(
    args: &[&str],
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    if args.is_empty() {
        println!("{}", formatter.warning("Usage: job <submit|status|list|result|cancel>"));
        return Ok(());
    }
    
    match args[0] {
        "submit" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: job submit <input>"));
            } else {
                let input = args[1..].join(" ");
                let cmd = crate::commands::job::JobCommand::Submit {
                    input,
                    node: config.default_node.clone(),
                    input_file: None,
                    model: config.default_model.clone(),
                    wait: true,
                    timeout: 60,
                };
                crate::commands::job::execute(cmd, config, formatter).await?;
            }
        }
        "status" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: job status <job_id>"));
            } else {
                let cmd = crate::commands::job::JobCommand::Status {
                    job_id: args[1].to_string(),
                    node: config.default_node.clone(),
                };
                crate::commands::job::execute(cmd, config, formatter).await?;
            }
        }
        "list" => {
            let cmd = crate::commands::job::JobCommand::List {
                node: config.default_node.clone(),
                status: None,
                limit: 50,
            };
            crate::commands::job::execute(cmd, config, formatter).await?;
        }
        "result" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: job result <job_id>"));
            } else {
                let cmd = crate::commands::job::JobCommand::Result {
                    job_id: args[1].to_string(),
                    node: config.default_node.clone(),
                };
                crate::commands::job::execute(cmd, config, formatter).await?;
            }
        }
        "cancel" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: job cancel <job_id>"));
            } else {
                let cmd = crate::commands::job::JobCommand::Cancel {
                    job_id: args[1].to_string(),
                    node: config.default_node.clone(),
                };
                crate::commands::job::execute(cmd, config, formatter).await?;
            }
        }
        _ => {
            println!("{}", formatter.warning(&format!("Unknown job command: {}. Available: submit, status, list, result, cancel", args[0])));
        }
    }
    
    Ok(())
}

async fn execute_node_command(
    args: &[&str],
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    if args.is_empty() {
        println!("{}", formatter.warning("Usage: node <list|info|health|metrics>"));
        return Ok(());
    }
    
    match args[0] {
        "list" => {
            let cmd = crate::commands::node::NodeCommand::List {
                watch: false,
                interval: 5,
                discovery_timeout: 3,
            };
            crate::commands::node::execute(cmd, config, formatter).await?;
        }
        "info" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: node info <node_id>"));
            } else {
                let cmd = crate::commands::node::NodeCommand::Info {
                    node_id: args[1].to_string(),
                };
                crate::commands::node::execute(cmd, config, formatter).await?;
            }
        }
        "health" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: node health <node_id>"));
            } else {
                let cmd = crate::commands::node::NodeCommand::Health {
                    node_id: args[1].to_string(),
                };
                crate::commands::node::execute(cmd, config, formatter).await?;
            }
        }
        "metrics" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: node metrics <node_id>"));
            } else {
                let cmd = crate::commands::node::NodeCommand::Metrics {
                    node_id: args[1].to_string(),
                    metric: None,
                    watch: false,
                };
                crate::commands::node::execute(cmd, config, formatter).await?;
            }
        }
        _ => {
            println!("{}", formatter.warning(&format!("Unknown node command: {}", args[0])));
        }
    }
    
    Ok(())
}

async fn execute_network_command(
    args: &[&str],
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    if args.is_empty() {
        println!("{}", formatter.warning("Usage: network <topology|peers|traffic>"));
        return Ok(());
    }
    
    match args[0] {
        "topology" => {
            let cmd = crate::commands::network::NetworkCommand::Topology {
                format: "ascii".to_string(),
            };
            crate::commands::network::execute(cmd, config, formatter).await?;
        }
        "peers" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: network peers <node_id>"));
            } else {
                let cmd = crate::commands::network::NetworkCommand::Peers {
                    node_id: args[1].to_string(),
                };
                crate::commands::network::execute(cmd, config, formatter).await?;
            }
        }
        "traffic" => {
            let cmd = crate::commands::network::NetworkCommand::Traffic {
                watch: false,
                filter: None,
            };
            crate::commands::network::execute(cmd, config, formatter).await?;
        }
        _ => {
            println!("{}", formatter.warning(&format!("Unknown network command: {}", args[0])));
        }
    }
    
    Ok(())
}

async fn execute_multi_command(
    args: &[&str],
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    if args.is_empty() {
        println!("{}", formatter.warning("Usage: multi <start|stop|list|logs>"));
        return Ok(());
    }
    
    match args[0] {
        "start" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: multi start <count>"));
            } else {
                let count: usize = args[1].parse().unwrap_or(1);
                let cmd = crate::commands::multi::MultiCommand::Start {
                    count,
                    base_port: None,
                    model: None,
                    binary: None,
                };
                crate::commands::multi::execute(cmd, config, formatter).await?;
            }
        }
        "stop" => {
            let nodes: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();
            let cmd = crate::commands::multi::MultiCommand::Stop { nodes };
            crate::commands::multi::execute(cmd, config, formatter).await?;
        }
        "list" => {
            let cmd = crate::commands::multi::MultiCommand::List;
            crate::commands::multi::execute(cmd, config, formatter).await?;
        }
        "logs" => {
            if args.len() < 2 {
                println!("{}", formatter.warning("Usage: multi logs <node_id>"));
            } else {
                let cmd = crate::commands::multi::MultiCommand::Logs {
                    node_id: args[1].to_string(),
                    lines: 50,
                    follow: false,
                };
                crate::commands::multi::execute(cmd, config, formatter).await?;
            }
        }
        _ => {
            println!("{}", formatter.warning(&format!("Unknown multi command: {}", args[0])));
        }
    }
    
    Ok(())
}

async fn execute_config_command(
    args: &[&str],
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    if args.is_empty() || args[0] == "show" {
        let cmd = crate::commands::config::ConfigCommand::Show { key: None };
        crate::commands::config::execute(cmd, config, formatter).await?;
    } else {
        println!("{}", formatter.warning(&format!("Unknown config command: {}", args[0])));
    }
    
    Ok(())
}



