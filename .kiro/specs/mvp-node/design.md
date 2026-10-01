# MVP Node Design Document

## Overview

The MVP Node is a minimal viable product implementation of a decentralized AI network node that can bootstrap the network independently and provide AI inference services. It combines P2P networking capabilities with AI model hosting to create the foundation for a distributed AI computation network.

The node serves as both a bootstrap node (the first node in the network) and an execution node (capable of running AI inference jobs). It uses libp2p for peer-to-peer communication, implements the established protocol schemas for job management, and hosts a lightweight micromodel for AI inference tasks.

## Architecture

The MVP Node follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    MVP Node Application                     │
├─────────────────────────────────────────────────────────────┤
│  Configuration Manager  │  Health Monitor  │  CLI Interface │
├─────────────────────────────────────────────────────────────┤
│                    Job Executor                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Job Queue     │  │ Inference Engine│  │ Job Tracker │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    P2P Network Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Gossipsub     │  │      mDNS       │  │  Identity   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Protocol Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │ Message Handler │  │ Schema Validator│  │ Crypto Utils│ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Configuration Manager
**Purpose**: Manages node configuration, identity, and runtime parameters.

**Key Responsibilities**:
- Load/generate Ed25519 keypairs for node identity
- Parse configuration files and command-line arguments
- Manage network settings (ports, bootstrap peers)
- Handle multi-instance configuration (unique ports, identities)

**Interface**:
```rust
pub struct NodeConfig {
    pub node_id: String,
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub model_path: String,
    pub max_queue_size: usize,
    pub log_level: String,
}

pub trait ConfigManager {
    fn load_config(path: &str) -> Result<NodeConfig>;
    fn generate_identity() -> Result<Keypair>;
    fn validate_config(config: &NodeConfig) -> Result<()>;
}
```

### 2. P2P Network Layer
**Purpose**: Handles peer discovery, connection management, and message routing.

**Key Responsibilities**:
- Implement libp2p swarm with gossipsub, mDNS, and identify protocols
- Manage peer connections and network topology
- Route messages according to protocol specifications
- Handle network events and maintain peer lists

**Interface**:
```rust
pub trait NetworkLayer {
    async fn start(&mut self, config: NetworkConfig) -> Result<()>;
    async fn publish_message(&mut self, topic: &str, message: P2PMessage) -> Result<()>;
    async fn subscribe_topic(&mut self, topic: &str) -> Result<()>;
    fn get_peer_count(&self) -> usize;
    fn get_local_peer_id(&self) -> PeerId;
}
```

### 3. Job Executor
**Purpose**: Manages job lifecycle from receipt to completion.

**Key Responsibilities**:
- Receive and validate incoming job requests
- Queue jobs for sequential processing
- Execute inference using the loaded micromodel
- Generate and publish job receipts

**Interface**:
```rust
pub trait JobExecutor {
    async fn submit_job(&mut self, job: JobOffer) -> Result<String>;
    async fn process_next_job(&mut self) -> Result<Option<JobResult>>;
    fn get_queue_status(&self) -> QueueStatus;
    async fn cancel_job(&mut self, job_id: &str) -> Result<()>;
}
```

### 4. Inference Engine
**Purpose**: Loads and executes the micromodel for AI inference tasks.

**Key Responsibilities**:
- Load and initialize micromodel (≤7B parameters)
- Validate model integrity and compatibility
- Execute inference requests with input validation
- Manage model memory and basic resource optimization

**Interface**:
```rust
pub trait InferenceEngine {
    async fn load_model(&mut self, model_path: &str) -> Result<()>;
    async fn infer(&self, input: &str) -> Result<InferenceResult>;
    fn get_model_info(&self) -> ModelInfo;
    fn is_model_loaded(&self) -> bool;
}
```

### 5. Protocol Handler
**Purpose**: Implements the established protocol schemas and message formats.

**Key Responsibilities**:
- Validate messages against JSON schemas
- Generate compliant announcements, receipts, and manifests
- Handle protocol versioning and backward compatibility
- Manage cryptographic signatures

**Interface**:
```rust
pub trait ProtocolHandler {
    fn create_manifest(&self, capabilities: &Capabilities) -> Result<ExecutionManifest>;
    fn create_receipt(&self, job_result: &JobResult) -> Result<JobReceipt>;
    fn validate_job_offer(&self, offer: &JobOffer) -> Result<()>;
    fn sign_message(&self, message: &[u8]) -> Result<String>;
}
```

## Data Models

### Core Data Structures

```rust
// Node identity and capabilities
pub struct NodeIdentity {
    pub peer_id: PeerId,
    pub keypair: Keypair,
    pub manifest_id: String,
}

// Job processing
pub struct JobOffer {
    pub job_id: String,
    pub model: String,
    pub mode: JobMode,
    pub reward: f64,
    pub currency: String,
    pub requirements: Requirements,
    pub input_data: String,
}

pub struct JobResult {
    pub job_id: String,
    pub status: JobStatus,
    pub output: Option<String>,
    pub metrics: ExecutionMetrics,
    pub error: Option<String>,
}

// Queue management
pub struct QueueStatus {
    pub pending_jobs: usize,
    pub processing_job: Option<String>,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
}

// Model information
pub struct ModelInfo {
    pub name: String,
    pub parameters: u64,
    pub memory_usage_mb: u64,
    pub supported_tasks: Vec<String>,
}
```

### State Management

The node maintains several key state components:

1. **Network State**: Active peers, connection status, subscription topics
2. **Job State**: Queue contents, processing status, completed jobs
3. **Model State**: Loaded model, memory usage, performance metrics
4. **Configuration State**: Runtime settings, identity, capabilities

## Error Handling

### Error Categories

1. **Network Errors**: Connection failures, message parsing errors, peer discovery issues
2. **Job Errors**: Invalid input, processing failures, timeout conditions
3. **Model Errors**: Loading failures, inference errors, memory issues
4. **Configuration Errors**: Invalid settings, missing files, permission issues

### Error Recovery Strategies

```rust
pub enum NodeError {
    NetworkError(NetworkError),
    JobError(JobError),
    ModelError(ModelError),
    ConfigError(ConfigError),
}

// Recovery mechanisms
impl NodeError {
    pub fn is_recoverable(&self) -> bool;
    pub fn retry_strategy(&self) -> Option<RetryStrategy>;
    pub fn fallback_action(&self) -> Option<FallbackAction>;
}
```

### Graceful Degradation

- **Network Isolation**: Continue processing local jobs when network is unavailable
- **Model Failures**: Return appropriate error responses while maintaining network connectivity
- **Resource Exhaustion**: Implement backpressure and job rejection when resources are limited

## Testing Strategy

The testing approach combines unit testing for individual components with property-based testing for system-wide correctness properties.

### Unit Testing Framework
- **Framework**: Rust's built-in `cargo test` with `tokio-test` for async testing
- **Coverage**: All public interfaces and critical error paths
- **Mocking**: Use `mockall` crate for external dependencies

### Property-Based Testing Framework
- **Framework**: `proptest` crate for Rust property-based testing
- **Configuration**: Minimum 100 iterations per property test
- **Generators**: Smart generators that constrain inputs to valid ranges

### Testing Categories

**Unit Tests**:
- Component initialization and configuration
- Message serialization/deserialization
- Job queue operations
- Model loading and inference
- Error handling and recovery

**Property-Based Tests**:
- Protocol compliance across all message types
- Job processing correctness under various conditions
- Network behavior with multiple node instances
- Resource management and memory safety


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Node Initialization and Lifecycle

**Property 1: Network initialization on startup**
*For any* MVP_Node startup, the node should initialize the P2P_Network and become discoverable by other nodes through mDNS and gossipsub protocols.
**Validates: Requirements 1.1**

**Property 2: Bootstrap node operation**
*For any* MVP_Node started with an empty bootstrap peer list, the node should successfully operate as a Bootstrap_Node and accept incoming connections from other nodes.
**Validates: Requirements 1.2**

**Property 3: Model loading on startup**
*For any* MVP_Node startup with a valid model path, the Inference_Engine should successfully load the Micromodel and be ready to process inference requests.
**Validates: Requirements 1.3**

**Property 4: Capability announcement**
*For any* running MVP_Node, the node should publish capability announcement messages to the P2P_Network containing its manifest and available models.
**Validates: Requirements 1.4**

**Property 5: Graceful shutdown**
*For any* MVP_Node receiving a shutdown signal, the node should close all network connections and save state within 30 seconds.
**Validates: Requirements 1.5**

### Job Processing

**Property 6: Valid job acceptance**
*For any* valid inference job submitted to the MVP_Node, the job should be accepted and added to the Job_Queue.
**Validates: Requirements 2.1**

**Property 7: Input validation during processing**
*For any* job being processed by the Job_Executor, the input should be validated against the Micromodel requirements before inference execution.
**Validates: Requirements 2.2**

**Property 8: Inference execution**
*For any* job with validated input, the Inference_Engine should use the loaded Micromodel to generate a response.
**Validates: Requirements 2.3**

**Property 9: Result metadata inclusion**
*For any* completed inference job, the MVP_Node should return a result containing execution metadata (duration, tokens processed, resource usage).
**Validates: Requirements 2.4**

**Property 10: Error response formatting**
*For any* failed job, the MVP_Node should return an error response containing detailed failure information.
**Validates: Requirements 2.5**

### Network Communication

**Property 11: Peer discovery response**
*For any* peer discovery attempt from another node, the MVP_Node should respond with its network identity and capabilities.
**Validates: Requirements 3.1**

**Property 12: Connection maintenance**
*For any* established peer connection, the MVP_Node should maintain the connection and enable bidirectional message exchange.
**Validates: Requirements 3.2**

**Property 13: Topology update handling**
*For any* network topology change (peer join/leave), the MVP_Node should update its peer list and routing information accordingly.
**Validates: Requirements 3.3**

**Property 14: Protocol-compliant message processing**
*For any* network message received, the MVP_Node should process it according to the protocol specifications defined in the schema files.
**Validates: Requirements 3.4**

**Property 15: Connection cleanup**
*For any* peer disconnection event, the MVP_Node should clean up the connection resources and update routing tables.
**Validates: Requirements 3.5**

### Model Management

**Property 16: Model size constraint**
*For any* Micromodel being loaded, the Inference_Engine should verify that the model has ≤7B parameters before completing initialization.
**Validates: Requirements 4.1**

**Property 17: Model integrity verification**
*For any* Micromodel being loaded, the Inference_Engine should verify model integrity and compatibility before making it available for inference.
**Validates: Requirements 4.2**

**Property 18: Inference timeout compliance**
*For any* standard inference request, the Inference_Engine should complete processing within 30 seconds.
**Validates: Requirements 4.3**

**Property 19: Model memory persistence**
*For any* idle period, the Inference_Engine should maintain the Micromodel in memory to enable fast subsequent requests.
**Validates: Requirements 4.4**

**Property 20: Memory management under pressure**
*For any* memory pressure condition, the MVP_Node should implement memory management strategies to maintain stability.
**Validates: Requirements 4.5**

### Monitoring and Logging

**Property 21: Job execution logging**
*For any* job processed by the MVP_Node, the node should log execution details including timing and resource usage.
**Validates: Requirements 5.1**

**Property 22: Network event logging**
*For any* network event (peer connection, disconnection, message exchange), the MVP_Node should log the event with relevant details.
**Validates: Requirements 5.2**

**Property 23: Error logging completeness**
*For any* error that occurs, the MVP_Node should log detailed error information sufficient for debugging.
**Validates: Requirements 5.3**

**Property 24: Health status reporting**
*For any* health query to the MVP_Node, the node should provide current health and status information.
**Validates: Requirements 5.4**

**Property 25: Metrics exposure**
*For any* running MVP_Node, the node should expose metrics about job processing and network activity.
**Validates: Requirements 5.5**

### Concurrency and Load Management

**Property 26: Concurrent job queuing**
*For any* set of jobs arriving simultaneously, the Job_Queue should queue all jobs for sequential processing without loss.
**Validates: Requirements 6.1**

**Property 27: Fair job scheduling**
*For any* sequence of jobs in the Job_Queue, the Job_Executor should process them in fair order without starvation.
**Validates: Requirements 6.2**

**Property 28: Queue overflow handling**
*For any* job submission when the Job_Queue is full, the MVP_Node should reject the job with an appropriate error message.
**Validates: Requirements 6.3**

**Property 29: Queue status reporting**
*For any* time when jobs are waiting in the Job_Queue, the MVP_Node should provide accurate queue status information to clients.
**Validates: Requirements 6.4**

**Property 30: Resource-based load management**
*For any* condition where system resources are limited, the MVP_Node should implement load management to prevent system failure.
**Validates: Requirements 6.5**

### Protocol Compliance

**Property 31: Job schema compliance**
*For any* job announcement sent by the MVP_Node, the message should conform to the standard job schema format defined in job.schema.json.
**Validates: Requirements 7.1**

**Property 32: Receipt format and signature compliance**
*For any* receipt published by the MVP_Node, the message should include all required fields and contain a valid cryptographic signature.
**Validates: Requirements 7.2**

**Property 33: Manifest schema compliance**
*For any* capability announcement, the MVP_Node should use the manifest schema with accurate hardware information as defined in manifest.schema.json.
**Validates: Requirements 7.3**

**Property 34: Message encoding compliance**
*For any* P2P message exchanged, the MVP_Node should follow the protocol's message encoding standards (JSON serialization with proper type tags).
**Validates: Requirements 7.4**

**Property 35: Protocol version compatibility**
*For any* protocol message with a supported version, the MVP_Node should maintain backward compatibility in message handling.
**Validates: Requirements 7.5**

### Multi-Instance Support

**Property 36: Port conflict avoidance**
*For any* set of MVP_Node instances starting on the same machine, each Node_Instance should use different network ports to avoid conflicts.
**Validates: Requirements 8.1**

**Property 37: Instance configuration uniqueness**
*For any* Node_Instance startup, the MVP_Node should accept and apply configuration parameters that specify unique identity and network settings.
**Validates: Requirements 8.2**

**Property 38: Multi-node connectivity**
*For any* set of Node_Instance that attempt to connect to each other, the P2P_Network should establish connections and enable message routing between all nodes.
**Validates: Requirements 8.3**

**Property 39: Instance isolation during job processing**
*For any* job received by one Node_Instance, the MVP_Node should process it locally without interfering with other running instances.
**Validates: Requirements 8.4**

**Property 40: Independent state management**
*For any* set of Node_Instance running simultaneously, each MVP_Node should maintain independent state and Job_Queue management.
**Validates: Requirements 8.5**

## Implementation Details

### Technology Stack

**Core Language**: Rust (for performance, safety, and async support)

**Key Dependencies**:
- `libp2p` (v0.53+): P2P networking with gossipsub, mDNS, and identify protocols
- `tokio` (v1.37+): Async runtime for concurrent operations
- `serde` + `serde_json`: Message serialization/deserialization
- `candle` or `llama.cpp` bindings: Micromodel inference engine
- `tracing` + `tracing-subscriber`: Structured logging
- `clap`: CLI argument parsing
- `config`: Configuration file management

### Directory Structure

```
mvp-node/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── node.rs              # Main node orchestration
│   ├── network/
│   │   ├── mod.rs           # Network layer module
│   │   ├── p2p.rs           # libp2p integration
│   │   └── protocol.rs      # Protocol message handling
│   ├── executor/
│   │   ├── mod.rs           # Job executor module
│   │   ├── queue.rs         # Job queue implementation
│   │   └── tracker.rs       # Job tracking and status
│   ├── inference/
│   │   ├── mod.rs           # Inference engine module
│   │   ├── model.rs         # Model loading and management
│   │   └── runner.rs        # Inference execution
│   ├── protocol/
│   │   ├── mod.rs           # Protocol types module
│   │   ├── schemas.rs       # Schema validation
│   │   └── crypto.rs        # Cryptographic utilities
│   └── monitoring/
│       ├── mod.rs           # Monitoring module
│       ├── logger.rs        # Logging configuration
│       └── metrics.rs       # Metrics collection
├── tests/
│   ├── integration/         # Integration tests
│   └── property/            # Property-based tests
├── config/
│   └── default.toml         # Default configuration
└── Cargo.toml
```

### Configuration Format

```toml
[node]
id = "auto"  # Auto-generate or specify
listen_port = 0  # 0 for auto-assign
log_level = "info"

[network]
bootstrap_peers = []  # Empty for bootstrap node
enable_mdns = true
gossipsub_heartbeat_interval_secs = 1

[model]
path = "./models/micromodel.gguf"
max_context_length = 2048
inference_timeout_secs = 30

[executor]
max_queue_size = 100
max_concurrent_jobs = 1  # Sequential processing for MVP

[monitoring]
enable_metrics = true
metrics_port = 9090
log_format = "json"
```

### Startup Sequence

1. **Load Configuration**: Parse config file and CLI arguments
2. **Initialize Identity**: Load or generate Ed25519 keypair
3. **Start Network Layer**: Initialize libp2p swarm, subscribe to topics
4. **Load Model**: Initialize inference engine and load micromodel
5. **Publish Capabilities**: Announce node availability to network
6. **Start Event Loop**: Begin processing network events and jobs
7. **Health Check**: Verify all components are operational

### Message Flow Example

```
Client → Network → MVP Node
                    ↓
              [Validate Job]
                    ↓
              [Add to Queue]
                    ↓
              [Process Job]
                    ↓
           [Run Inference]
                    ↓
          [Generate Receipt]
                    ↓
MVP Node → Network → Client
```

## Security Considerations

### Identity and Authentication
- Ed25519 keypairs for node identity
- Message signing for authenticity verification
- Peer identity validation before accepting connections

### Input Validation
- Schema validation for all incoming messages
- Input sanitization before inference execution
- Resource limit enforcement to prevent DoS

### Network Security
- Encrypted connections using libp2p noise protocol
- Peer reputation tracking (future enhancement)
- Rate limiting for message processing

### Data Privacy
- No persistent storage of job inputs/outputs in MVP
- Secure key storage with appropriate file permissions
- Optional TEE support for future versions

## Performance Considerations

### Resource Management
- Single-threaded inference execution for MVP simplicity
- Memory-mapped model loading for efficiency
- Connection pooling for network efficiency

### Optimization Strategies
- Model quantization for reduced memory footprint
- Batch processing for multiple small requests (future)
- Caching of frequently used model outputs (future)

### Scalability Limits
- MVP targets single-node operation with basic multi-instance support
- Queue size limits prevent memory exhaustion
- Timeout mechanisms prevent resource starvation

## Deployment

### System Requirements
- **OS**: Linux (Ubuntu 22.04+), macOS (12+), or Windows 10+
- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 10GB for model and application
- **Network**: Stable internet connection for P2P communication

### Installation Steps
1. Install Rust toolchain (1.75+)
2. Clone repository and build: `cargo build --release`
3. Download micromodel to `./models/` directory
4. Configure node settings in `config.toml`
5. Run node: `./target/release/mvp-node --config config.toml`

### Multi-Instance Testing
```bash
# Terminal 1: Start bootstrap node
./mvp-node --config node1.toml --port 4001

# Terminal 2: Start second node
./mvp-node --config node2.toml --port 4002 --bootstrap /ip4/127.0.0.1/tcp/4001

# Terminal 3: Start third node
./mvp-node --config node3.toml --port 4003 --bootstrap /ip4/127.0.0.1/tcp/4001
```

## Future Enhancements

### Phase 2 Features
- Multi-threaded job processing
- Advanced load balancing
- Persistent job history
- Enhanced monitoring dashboard

### Phase 3 Features
- Support for larger models (>7B parameters)
- Model hot-swapping
- Distributed inference across multiple nodes
- Advanced reputation system

### Long-term Vision
- Full TEE integration for secure computation
- Cross-chain payment support
- Federated learning capabilities
- Dynamic model marketplace
