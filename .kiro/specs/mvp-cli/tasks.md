# Implementation Plan

- [x] 1. Setup project structure and core dependencies
  - ✅ Create new Rust binary crate `mvp-cli` in workspace
  - ✅ Add dependencies: clap, libp2p, tokio, serde, serde_json, rustyline, comfy-table, colored, indicatif
  - ✅ Define core module structure: main.rs, config.rs, client.rs, commands/, output.rs, repl.rs
  - ✅ Setup error types and Result aliases
  - _Requirements: All_

- [x] 2. Implement configuration management
  - [x] 2.1 Create configuration data structures
    - ✅ Define CliConfig struct with all settings
    - ✅ Implement Default trait for safe defaults
    - _Requirements: 7.1, 7.4_
  
  - [x] 2.2 Implement config file loading (TOML)
    - ✅ Parse config.toml using serde
    - ✅ Handle missing file gracefully
    - _Requirements: 7.1_
  
  - [x] 2.3 Implement environment variable parsing
    - ✅ Read MVP_CLI_* environment variables
    - ✅ Apply precedence over config file
    - _Requirements: 7.2_
  
  - [x] 2.4 Implement CLI flag parsing with clap
    - ✅ Define command structure with clap derive macros
    - ✅ Apply highest precedence to CLI flags
    - _Requirements: 7.3_
  
  - [x] 2.5 Implement configuration merging logic
    - ✅ Merge configs with correct precedence: file < env < flags
    - ✅ Validate merged configuration
    - _Requirements: 7.1, 7.2, 7.3_
  
  - [x] 2.6 Write property test for configuration precedence
    - ✅ **Property 31, 32, 33: Configuration loading and precedence**
    - **Validates: Requirements 7.1, 7.2, 7.3**
  
  - [x] 2.7 Write property test for invalid configuration handling
    - ✅ **Property 34: Invalid configuration handling**
    - **Validates: Requirements 7.4**

- [x] 3. Implement Node Client for P2P communication
  - [x] 3.1 Create NodeClient struct with libp2p swarm
    - ✅ Initialize libp2p swarm with gossipsub and identify protocols
    - ✅ Implement connection pool management
    - _Requirements: 1.1, 3.1_
  
  - [x] 3.2 Implement connection establishment
    - ✅ Connect to node via multiaddr
    - ✅ Handle connection errors and retries
    - _Requirements: 1.1_
  
  - [x] 3.3 Implement job submission protocol
    - ✅ Send JobOffer message via gossipsub
    - ✅ Wait for acknowledgment and job_id
    - _Requirements: 1.1, 1.2_
  
  - [x] 3.4 Implement job status query
    - ✅ Send status query message
    - ✅ Parse and return JobStatus response
    - _Requirements: 2.1_
  
  - [x] 3.5 Implement node discovery and listing
    - ✅ Query network for active peers
    - ✅ Collect node information and capabilities
    - _Requirements: 3.1, 3.2_
  
  - [x] 3.6 Write property test for connection management
    - ✅ **Property 27: Persistent connection maintenance**
    - **Validates: Requirements 6.2**

- [x] 4. Implement Job command handlers
  - [x] 4.1 Implement job submit command
    - ✅ Parse input from CLI args or file
    - ✅ Validate parameters before submission
    - ✅ Send job to node and return job_id
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  
  - [x] 4.2 Implement wait-for-result functionality
    - ✅ Poll job status until completion
    - ✅ Display result when available
    - _Requirements: 1.5_
  
  - [x] 4.3 Implement job status command
    - ✅ Query job status by job_id
    - ✅ Display current status and progress
    - _Requirements: 2.1, 2.3_
  
  - [x] 4.4 Implement job list command
    - ✅ Query all jobs from node
    - ✅ Format as table with filtering options
    - _Requirements: 2.2_
  
  - [x] 4.5 Implement job result command
    - ✅ Retrieve complete result for completed job
    - ✅ Display result with metadata
    - _Requirements: 2.4_
  
  - [x] 4.6 Implement error display for failed jobs
    - ✅ Extract error details from failed jobs
    - ✅ Format error information clearly
    - _Requirements: 2.5_
  
  - [x] 4.7 Write property test for job submission
    - ✅ **Property 1, 2: Job submission returns valid job_id with required fields**
    - **Validates: Requirements 1.1, 1.2**
  
  - [x] 4.8 Write property test for file input reading
    - ✅ **Property 3: File input reading**
    - **Validates: Requirements 1.3**
  
  - [x] 4.9 Write property test for parameter validation
    - ✅ **Property 4: Parameter validation before submission**
    - **Validates: Requirements 1.4**

- [x] 5. Implement Node command handlers
  - [x] 5.1 Implement node list command
    - ✅ Discover all nodes in network
    - ✅ Format node information as table
    - _Requirements: 3.1, 3.2_
  
  - [x] 5.2 Implement watch mode for node list
    - ✅ Continuously update node list
    - ✅ Detect nodes joining/leaving
    - _Requirements: 3.5_
  
  - [x] 5.3 Implement node info command
    - ✅ Query detailed information for specific node
    - ✅ Display models, metrics, and capabilities
    - _Requirements: 3.3_
  
  - [x] 5.4 Implement node health command
    - ✅ Query health status from node
    - ✅ Display health information
    - _Requirements: 3.4_
  
  - [x] 5.5 Write property test for node discovery
    - ✅ **Property 11, 12: Node discovery listing with complete information**
    - **Validates: Requirements 3.1, 3.2**

- [x] 6. Implement Network command handlers
  - [x] 6.1 Implement topology visualization
    - ✅ Query network topology from nodes
    - ✅ Build graph representation
    - ✅ Display as ASCII art or structured format
    - _Requirements: 5.1_
  
  - [x] 6.2 Implement connection metrics display
    - ✅ Collect latency and message count for connections
    - ✅ Display metrics in topology view
    - _Requirements: 5.2_
  
  - [x] 6.3 Implement route query command
    - ✅ Calculate routing path between two nodes
    - ✅ Display path with intermediate hops
    - _Requirements: 5.3_
  
  - [x] 6.4 Implement traffic monitoring
    - ✅ Subscribe to network message stream
    - ✅ Display messages in real-time
    - _Requirements: 5.4_
  
  - [x] 6.5 Implement connectivity problem detection
    - ✅ Analyze topology for isolated nodes
    - ✅ Highlight problematic connections
    - _Requirements: 5.5_
  
  - [x] 6.6 Write property test for topology visualization
    - ✅ **Property 21, 22: Topology visualization with metrics**
    - **Validates: Requirements 5.1, 5.2**

- [x] 7. Implement Multi-Node Manager
  - [x] 7.1 Create MultiNodeManager struct
    - ✅ Track running node instances
    - ✅ Manage process lifecycle
    - _Requirements: 4.1, 4.3_
  
  - [x] 7.2 Implement multi-node start command
    - ✅ Generate unique configurations for each instance
    - ✅ Assign unique ports automatically
    - ✅ Start node processes
    - _Requirements: 4.1_
  
  - [x] 7.3 Implement automatic bootstrap configuration
    - ✅ Configure first node as bootstrap
    - ✅ Configure subsequent nodes to connect to bootstrap
    - _Requirements: 4.2_
  
  - [x] 7.4 Implement instance information display
    - ✅ Show node_id, port, and PID for each instance
    - ✅ Format as table
    - _Requirements: 4.3_
  
  - [x] 7.5 Implement multi-node stop command
    - ✅ Send graceful shutdown signal to processes
    - ✅ Wait for clean termination
    - ✅ Clean up resources
    - _Requirements: 4.4_
  
  - [x] 7.6 Implement aggregated log viewing
    - ✅ Collect logs from all running instances
    - ✅ Merge and display with instance labels
    - _Requirements: 4.5_
  
  - [x] 7.7 Write property test for unique port assignment
    - ✅ **Property 16: Multiple instance startup with unique ports**
    - **Validates: Requirements 4.1**
  
  - [x] 7.8 Write property test for bootstrap configuration
    - ✅ **Property 17: Automatic bootstrap configuration**
    - **Validates: Requirements 4.2**

- [x] 8. Implement Output Formatter
  - [x] 8.1 Create OutputFormatter trait and implementations
    - ✅ Implement table formatting with comfy-table
    - ✅ Implement JSON serialization
    - ✅ Implement CSV generation
    - _Requirements: 10.1, 10.2_
  
  - [x] 8.2 Implement terminal detection
    - ✅ Detect if output is redirected
    - ✅ Disable ANSI colors for non-terminal output
    - _Requirements: 10.3_
  
  - [x] 8.3 Implement metadata inclusion
    - ✅ Add timestamp and CLI version to exports
    - ✅ Include in JSON and CSV outputs
    - _Requirements: 10.4_
  
  - [x] 8.4 Implement file output writing
    - ✅ Write formatted output to specified file
    - ✅ Handle file I/O errors
    - _Requirements: 10.5_
  
  - [x] 8.5 Write property test for JSON serialization
    - ✅ **Property 46: JSON serialization produces valid JSON**
    - **Validates: Requirements 10.1**
  
  - [x] 8.6 Write property test for CSV generation
    - ✅ **Property 47: CSV table generation**
    - **Validates: Requirements 10.2**
  
  - [x] 8.7 Write property test for terminal detection
    - ✅ **Property 48: Terminal detection disables formatting**
    - **Validates: Requirements 10.3**

- [x] 9. Implement Interactive Mode (REPL)
  - [x] 9.1 Create ReplEngine struct
    - ✅ Initialize rustyline editor
    - ✅ Setup command history
    - _Requirements: 6.1, 6.5_
  
  - [x] 9.2 Implement command loop
    - ✅ Display prompt and read input
    - ✅ Parse and execute commands
    - ✅ Display results and return to prompt
    - _Requirements: 6.1, 6.3_
  
  - [x] 9.3 Implement persistent connection management
    - ✅ Maintain NodeClient across commands
    - ✅ Reuse connections for efficiency
    - _Requirements: 6.2_
  
  - [x] 9.4 Implement auto-complete
    - ✅ Register command completions with rustyline
    - ✅ Provide suggestions for partial commands
    - _Requirements: 6.4_
  
  - [x] 9.5 Implement command history
    - ✅ Save history to file
    - ✅ Load history on startup
    - ✅ Allow history navigation
    - _Requirements: 6.5_
  
  - [x] 9.6 Write property test for command execution cycle
    - ✅ **Property 26, 28: Continuous command acceptance and formatted display**
    - **Validates: Requirements 6.1, 6.3**
  
  - [x] 9.7 Write property test for auto-complete
    - ✅ **Property 29: Auto-complete functionality**
    - **Validates: Requirements 6.4**

- [x] 10. Implement Script Automation
  - [x] 10.1 Create script parser
    - ✅ Parse script file line by line
    - ✅ Support comments and empty lines
    - _Requirements: 8.1_
  
  - [x] 10.2 Implement variable substitution
    - ✅ Parse variable definitions
    - ✅ Substitute variables in commands
    - _Requirements: 8.2_
  
  - [x] 10.3 Implement script execution engine
    - ✅ Execute commands sequentially
    - ✅ Handle errors based on configuration (continue/stop)
    - _Requirements: 8.1, 8.3_
  
  - [x] 10.4 Implement execution report generation
    - ✅ Track results of each command
    - ✅ Generate summary report
    - _Requirements: 8.4_
  
  - [x] 10.5 Implement dry-run mode
    - ✅ Validate script without execution
    - Report validation results
    - _Requirements: 8.5_
  
  - [x] 10.6 Write property test for sequential execution
    - ✅ **Property 36: Sequential script execution**
    - **Validates: Requirements 8.1**
  
  - [x] 10.7 Write property test for variable substitution
    - ✅ **Property 37: Variable substitution**
    - **Validates: Requirements 8.2**

- [x] 11. Implement Logging and Metrics Display
  - [x] 11.1 Implement log formatting
    - ✅ Apply colors based on severity level
    - ✅ Format timestamps and component names
    - _Requirements: 9.1_
  
  - [x] 11.2 Implement log filtering
    - ✅ Filter by level, component, or keyword
    - ✅ Support regex patterns
    - _Requirements: 9.2_
  
  - [x] 11.3 Implement ASCII graph generation for metrics
    - ✅ Use textplots or similar for ASCII graphs
    - ✅ Display metrics over time
    - _Requirements: 9.3_
  
  - [x] 11.4 Implement aggregate statistics calculation
    - ✅ Calculate mean, p95, p99 for metrics
    - ✅ Display statistics alongside graphs
    - _Requirements: 9.4_
  
  - [x] 11.5 Implement watch mode for logs and metrics
    - ✅ Continuously update display
    - ✅ Handle terminal resizing
    - _Requirements: 9.5_
  
  - [x] 11.6 Write property test for log filtering
    - ✅ **Property 42: Log filtering**
    - **Validates: Requirements 9.2**
  
  - [x] 11.7 Write property test for statistics calculation
    - ✅ **Property 44: Aggregate statistics calculation**
    - **Validates: Requirements 9.4**

- [x] 12. Implement Config command handlers
  - [x] 12.1 Implement config show command
    - ✅ Display all active configuration settings
    - ✅ Show source of each setting (file/env/flag)
    - _Requirements: 7.5_
  
  - [x] 12.2 Implement config set command (optional)
    - ✅ Allow setting config values interactively
    - ✅ Save to config file
    - _Requirements: 7.1_
  
  - [x] 12.3 Implement config validate command
    - ✅ Validate current configuration
    - ✅ Report any issues or warnings
    - _Requirements: 7.4_
  
  - [x] 12.4 Write property test for config display
    - ✅ **Property 35: Configuration display**
    - **Validates: Requirements 7.5**

- [x] 13. Implement main CLI entry point
  - [x] 13.1 Create main.rs with command routing
    - ✅ Parse CLI arguments
    - ✅ Route to appropriate command handler
    - ✅ Handle global flags (verbose, output format)
    - _Requirements: All_
  
  - [x] 13.2 Implement error handling and user-friendly messages
    - ✅ Catch all errors at top level
    - ✅ Display clear error messages
    - ✅ Suggest fixes when possible
    - _Requirements: All_
  
  - [x] 13.3 Implement version and help commands
    - ✅ Display version information
    - ✅ Generate help text from clap
    - _Requirements: All_

- [x] 14. Add shell completion generation
  - [x] 14.1 Implement completion generation command
    - ✅ Generate bash completion script
    - ✅ Generate zsh completion script
    - ✅ Generate fish completion script
    - _Requirements: 6.4_

- [x] 15. Create example configurations and scripts
  - [x] 15.1 Create example config.toml
    - ✅ Document all configuration options
    - ✅ Provide sensible defaults
    - _Requirements: 7.1_
  
  - [x] 15.2 Create example test scripts
    - ✅ Basic job submission script (test_basic.txt)
    - ✅ Multi-node testing script (test_multinode.txt)
    - ✅ Network exploration script (test_network.txt)
    - ✅ Script with variables (test_with_vars.txt)
    - _Requirements: 8.1_

- [x] 16. Write documentation
  - [x] 16.1 Create README.md for mvp-cli
    - ✅ Installation instructions
    - ✅ Quick start guide
    - ✅ Command reference
    - _Requirements: All_
  
  - [x] 16.2 Create usage examples
    - ✅ Document common workflows
    - ✅ Provide example commands
    - _Requirements: All_

- [x] 17. Checkpoint - Ensure all tests pass
  - ✅ All 20 property tests passed

- [x] 18. Integration testing with real MVP Node
  - [x] 18.1 Test job submission end-to-end
    - ✅ Created integration tests for CLI commands
    - ✅ Test job submit, status, list commands
    - ✅ Verify result output
    - _Requirements: 1.1, 1.2, 1.5_
  
  - [x] 18.2 Test multi-node scenario
    - ✅ Test multi start/stop/list commands
    - ✅ Verify network formation via topology command
    - ✅ Test output formats (JSON, CSV, Plain)
    - _Requirements: 4.1, 4.2, 4.3_
  
  - [x] 18.3 Test interactive mode workflow
    - ✅ Test node, network, config commands
    - ✅ Test shell completions (bash, zsh, fish)
    - ✅ Test script validation
    - _Requirements: 6.1, 6.2, 6.3_

- [x] 19. Final Checkpoint - Ensure all tests pass
  - ✅ 27 integration tests passed
  - ✅ 20 property tests passed
  - ✅ Total: 47 tests passing
