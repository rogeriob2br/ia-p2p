# MVP CLI

Command Line Interface for the Decentralized AI Network.

## Features

- **Real P2P Communication**: Direct peer-to-peer queries using libp2p request-response protocol
- **Automatic Node Discovery**: Nodes are discovered automatically via mDNS
- **Job Management**: Submit inference jobs, track status, view results
- **Node Monitoring**: List nodes, check health, view metrics, ping
- **Network Exploration**: Visualize topology, monitor traffic
- **Multi-Node Testing**: Start and manage multiple node instances
- **Interactive Mode**: REPL with command history and auto-completion
- **Script Automation**: Run test scripts with variable substitution
- **Multiple Output Formats**: Table, JSON, CSV, Plain text

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         mvp-cli                              │
│                                                              │
│  ┌──────────────┐         ┌─────────────────┐              │
│  │   Commands   │────────▶│   NodeClient    │              │
│  │              │         │   (P2P Peer)    │              │
│  │ - node list  │         │                 │              │
│  │ - node info  │         │  ┌───────────┐  │              │
│  │ - job submit │         │  │  Swarm    │  │              │
│  │ - job status │         │  │           │  │              │
│  └──────────────┘         │  │ - mDNS    │  │              │
│                           │  │ - identify│  │              │
│                           │  │ - req/res │  │              │
│                           │  └───────────┘  │              │
│                           └─────────────────┘              │
└─────────────────────────────────────────────────────────────┘
                                  │
                                  │ P2P Network (libp2p)
                                  │ Protocol: /mvp/query/1.0.0
                                  │
┌─────────────────────────────────────────────────────────────┐
│                        mvp-node                              │
│                                                              │
│  ┌─────────────────┐         ┌──────────────┐              │
│  │  QueryHandler   │◀────────│  P2PNetwork  │              │
│  │                 │         │              │              │
│  │ - GetNodeInfo   │         │  ┌────────┐  │              │
│  │ - GetMetrics    │         │  │ Swarm  │  │              │
│  │ - GetHealth     │         │  │        │  │              │
│  │ - GetJobStatus  │         │  │ - mDNS │  │              │
│  │ - Ping          │         │  │ - req  │  │              │
│  └─────────────────┘         │  │ - res  │  │              │
│                              │  └────────┘  │              │
│                              └──────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

## Installation

```bash
# From source
cd mvp-cli
cargo build --release

# Install globally
cargo install --path .
```

## Quick Start

```bash
# Show help
mvp-cli --help

# List discovered nodes (real P2P discovery)
mvp-cli node list

# Get detailed node info
mvp-cli node info <node-id>

# Check node health
mvp-cli node health <node-id>

# Ping a node
mvp-cli node ping <node-id>

# View node metrics
mvp-cli node metrics <node-id>

# Submit a job
mvp-cli job submit --node <node-id> "What is the capital of France?" --wait

# Start 3 nodes for testing
mvp-cli multi start 3

# Interactive mode
mvp-cli interactive
```

## P2P Communication

The CLI now uses **real P2P communication** with nodes via libp2p:

### Protocol

- **Protocol ID**: `/mvp/query/1.0.0`
- **Discovery**: mDNS (automatic local network discovery)
- **Transport**: TCP with Noise encryption and Yamux multiplexing
- **Timeout**: 30 seconds for queries
- **Serialization**: JSON with length-prefix framing

### Query Types

| Query | Description |
|-------|-------------|
| `GetNodeInfo` | Get detailed node information |
| `GetNodeMetrics` | Get performance metrics (jobs, latency, etc) |
| `GetNodeHealth` | Get health status of all components |
| `ListNodes` | List all known nodes |
| `GetJobStatus` | Get status of a specific job |
| `ListJobs` | List jobs with optional filter |
| `GetJobResult` | Get result of a completed job |
| `Ping` | Check connectivity and measure latency |

### How It Works

1. **CLI starts as P2P peer**: Creates a libp2p swarm with mDNS, identify, and request-response protocols
2. **Node discovery**: Nodes are discovered automatically via mDNS on the local network
3. **Direct queries**: Commands send queries directly to nodes via request-response protocol
4. **Real responses**: All data comes from actual nodes, not simulated

## Commands

### Node Commands

```bash
# List discovered nodes (waits for mDNS discovery)
mvp-cli node list
mvp-cli node list --watch              # Watch for changes
mvp-cli node list --discovery-timeout 5  # Wait longer for discovery

# Get node details (queries node via P2P)
mvp-cli node info <node-id>
mvp-cli node info 12D3KooW...          # Use peer ID

# Check node health status
mvp-cli node health <node-id>

# View node metrics
mvp-cli node metrics <node-id>
mvp-cli node metrics <node-id> --watch  # Watch metrics
mvp-cli node metrics <node-id> --metric latency  # Filter by metric

# Ping a node
mvp-cli node ping <node-id>
mvp-cli node ping <node-id> --count 10  # More pings
```

### Job Commands

```bash
mvp-cli job submit <INPUT>     # Submit inference job
mvp-cli job status <JOB_ID>    # Check job status
mvp-cli job list               # List all jobs
mvp-cli job result <JOB_ID>    # Get job result
mvp-cli job cancel <JOB_ID>    # Cancel a job
```

### Network Commands

```bash
mvp-cli network topology                # Show network topology
mvp-cli network topology --format json  # JSON format
mvp-cli network peers <NODE_ID>         # List node peers
mvp-cli network routes <FROM> <TO>      # Show routing path
mvp-cli network traffic --watch         # Monitor traffic
```

### Multi-Node Commands

```bash
mvp-cli multi start 3              # Start 3 node instances
mvp-cli multi start 5 --base-port 6000  # Custom base port
mvp-cli multi list                 # List running instances
mvp-cli multi stop                 # Stop all instances
mvp-cli multi stop node-1 node-2   # Stop specific nodes
mvp-cli multi logs <NODE_ID>       # View node logs
```

### Configuration Commands

```bash
mvp-cli config show              # Show current config
mvp-cli config set <KEY> <VALUE> # Set config value
mvp-cli config validate          # Validate config
mvp-cli config init              # Create default config file
```

### Script Automation

```bash
mvp-cli script run test.txt              # Run script
mvp-cli script run test.txt -v NODE=node1  # With variables
mvp-cli script validate test.txt         # Validate script
```

## Configuration

Configuration is loaded from multiple sources (in order of precedence):
1. Command-line flags (highest)
2. Environment variables (`MVP_CLI_*` prefix)
3. Config file (`~/.config/mvp-cli/config.toml`)

### Example Config File

```toml
# ~/.config/mvp-cli/config.toml

default_node = ""  # Empty = auto-discover
default_model = "tinyllama"
timeout_secs = 30
retry_attempts = 3

[connection]
default_port = 4001
connection_timeout_secs = 10
max_connections = 50  # Max P2P connections
discovery_timeout_secs = 3  # mDNS discovery wait time

[multi_node]
base_port = 5000
port_range = 100
log_dir = "./logs"

[interactive]
history_file = "~/.mvp_cli_history"
history_size = 1000
auto_complete_enabled = true
prompt = "mvp> "

[output]
table_style = "modern"
date_format = "%Y-%m-%d %H:%M:%S"
truncate_long_output = true
max_output_lines = 100
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `MVP_CLI_CONFIG` | Path to config file |
| `MVP_CLI_DEFAULT_NODE` | Default node address |
| `MVP_CLI_DEFAULT_MODEL` | Default model name |
| `MVP_CLI_TIMEOUT` | Command timeout in seconds |
| `MVP_CLI_DEFAULT_PORT` | Default connection port |
| `MVP_CLI_OUTPUT` | Output format (table, json, csv, plain) |
| `MVP_CLI_VERBOSE` | Enable verbose output |
| `MVP_CLI_NO_COLOR` | Disable colored output |
| `MVP_CLI_LOG_LEVEL` | Log level (trace, debug, info, warn, error) |

## Output Formats

### Table (default)

```
┌─────────┬──────────────┬─────────┬───────────┬─────────────────────┐
│ Node ID ┆ Peer ID      ┆ Status  ┆ Models    ┆ Address             │
╞═════════╪══════════════╪═════════╪═══════════╪═════════════════════╡
│ node-1  ┆ 12D3K...abc1 ┆ online  ┆ tinyllama ┆ /ip4/127.0.0.1/4001 │
│ node-2  ┆ 12D3K...abc2 ┆ online  ┆ tinyllama ┆ /ip4/127.0.0.1/4002 │
└─────────┴──────────────┴─────────┴───────────┴─────────────────────┘
```

### JSON

```bash
mvp-cli node list --output json
```

```json
[
  {"Node ID": "node-1", "Peer ID": "12D3K...abc1", "Status": "online", "Models": "tinyllama"},
  {"Node ID": "node-2", "Peer ID": "12D3K...abc2", "Status": "online", "Models": "tinyllama"}
]
```

### CSV

```bash
mvp-cli node list --output csv
```

```csv
Node ID,Peer ID,Status,Models,Address
node-1,12D3K...abc1,online,tinyllama,/ip4/127.0.0.1/4001
node-2,12D3K...abc2,online,tinyllama,/ip4/127.0.0.1/4002
```

## Interactive Mode

Start an interactive session:

```bash
mvp-cli interactive
```

Features:
- Command history (arrow keys)
- Auto-complete (Tab)
- Persistent P2P connections
- Clear screen with `clear`
- View history with `history`
- Exit with `exit` or `quit`

Example session:

```
╔═══════════════════════════════════════════╗
║        MVP CLI - Decentralized AI         ║
║              Version 0.1.0               ║
╚═══════════════════════════════════════════╝

Type 'help' for commands, 'exit' to quit

mvp> node list
Discovering Nodes...
✓ Found 2 node(s)

┌─────────┬──────────────┬─────────┐
│ Node ID ┆ Peer ID      ┆ Status  │
╞═════════╪══════════════╪═════════╡
│ node-1  ┆ 12D3K...abc1 ┆ online  │
│ node-2  ┆ 12D3K...abc2 ┆ online  │
└─────────┴──────────────┴─────────┘

mvp> node info node-1
Querying node 12D3KooW...

═══ Node: node-1 ═══
Node ID:    node-1
Peer ID:    12D3KooWxxxxxxxxxxxxxxxxxx
Status:     ● online
Version:    0.1.0
Uptime:     2h 35m 12s

═══ Capabilities ═══
  • tinyllama

═══ Statistics ═══
Connected Peers: 3
Queue:          5/100

mvp> node ping node-1
Pinging 12D3KooW... (3 times)
  ping 1: 12ms
  ping 2: 8ms
  ping 3: 10ms

✓ min/avg/max = 8/10ms/12ms (3/3 successful)

mvp> exit
Goodbye!
```

## Error Handling

The CLI provides helpful error messages:

```bash
# No nodes discovered
$ mvp-cli node list
⚠ No nodes discovered.
ℹ Make sure mvp-node is running on the network.

# Node not found
$ mvp-cli node info unknown-node
✗ Node 'unknown-node' not found

# Query timeout
$ mvp-cli node info node-1
✗ Query to 12D3KooW... timed out after 30 seconds
ℹ Try increasing the timeout with --timeout flag
```

## Script Format

Scripts are text files with one command per line:

```bash
# test_scenario.txt - Example test script
# Comments start with #

# Set up test nodes
multi start 3

# Wait for network formation
# (future: wait command)

# List nodes
node list

# Submit test jobs
job submit --node node-1 "Test query 1"
job submit --node node-2 "Test query 2"

# List all jobs
job list

# Clean up
multi stop
```

Run with variables:

```bash
mvp-cli script run test_scenario.txt -v MODEL=tinyllama -v COUNT=5
```

## Shell Completions

Generate shell completions:

```bash
# Bash
mvp-cli completions bash > /etc/bash_completion.d/mvp-cli

# Zsh
mvp-cli completions zsh > /usr/local/share/zsh/site-functions/_mvp-cli

# Fish
mvp-cli completions fish > ~/.config/fish/completions/mvp-cli.fish
```

## Development

### Building

```bash
cargo build           # Debug build
cargo build --release # Release build
```

### Testing

```bash
cargo test            # Run all tests
cargo test --lib      # Library tests only
cargo test --test property_tests  # Property tests only
```

### Running

```bash
cargo run -- --help                    # Show help
cargo run -- node list                 # List nodes
cargo run -- node info <node-id>       # Get node info
cargo run -- node ping <node-id>       # Ping node
```

## Requirements Matrix

| Requirement | Status | Command |
|-------------|--------|---------|
| 1.1 Job submission | ✅ | `job submit` |
| 1.2 Job confirmation | ✅ | `job submit --wait` |
| 1.3 File input | ✅ | `job submit --input-file` |
| 1.4 Parameter validation | ✅ | `job submit` |
| 2.1 Request-Response Protocol | ✅ | P2P queries |
| 2.2 CLI as P2P Peer | ✅ | NodeClient |
| 2.3 Unique PeerId | ✅ | Each execution |
| 2.4 Auto NodeInfo Query | ✅ | On discovery |
| 3.1 Node discovery (mDNS) | ✅ | `node list` |
| 3.2 Node info display | ✅ | `node info` |
| 3.3 Detailed node query | ✅ | `node info` |
| 3.4 Health check | ✅ | `node health` |
| 4.1 Multi-node start | ✅ | `multi start` |
| 4.2 Bootstrap config | ✅ | `multi start` |
| 4.3 Instance info | ✅ | `multi list` |
| 4.4 Graceful shutdown | ✅ | `multi stop` |
| 5.1 Topology visualization | ✅ | `network topology` |
| 5.2 Connection metrics | ✅ | `network peers` |
| 6.1 Interactive mode | ✅ | `interactive` |
| 6.4 Auto-complete | ✅ | REPL |
| 6.5 Command history | ✅ | REPL |
| 7.1 Config file | ✅ | `config` |
| 7.2 Environment variables | ✅ | `MVP_CLI_*` |
| 7.3 CLI flags | ✅ | All commands |
| 8.1 Script execution | ✅ | `script run` |
| 8.2 Variable substitution | ✅ | `script run -v` |
| 8.5 Dry-run | ✅ | `script validate` |
| 10.1 JSON output | ✅ | `--output json` |
| 10.2 CSV output | ✅ | `--output csv` |

## Troubleshooting

### No nodes discovered

```bash
$ mvp-cli node list
⚠ No nodes discovered.
```

**Solutions:**
1. Make sure `mvp-node` is running: `mvp-node start`
2. Check if nodes are on the same network
3. Increase discovery timeout: `mvp-cli node list --discovery-timeout 10`
4. Check firewall settings (mDNS uses UDP port 5353)

### Query timeout

```bash
$ mvp-cli node info node-1
✗ Query timed out
```

**Solutions:**
1. Check if the node is still running
2. Verify network connectivity
3. Increase timeout: `mvp-cli node info node-1 --timeout 60`

### Connection errors

```bash
$ mvp-cli node info node-1
✗ Connection error: dial failed
```

**Solutions:**
1. Check if the node is accessible
2. Verify the node address/peer ID is correct
3. Check firewall settings

## License

MIT OR Apache-2.0
