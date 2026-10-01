# Design Document: CLI P2P Integration

## Overview

This design implements full P2P integration for the CLI, transforming it from a stub implementation with fake data into a real P2P peer that can discover and communicate with nodes. The CLI will use libp2p's request-response protocol for direct queries and mDNS for automatic node discovery.

### Key Design Decisions

1. **CLI as Full P2P Peer**: The CLI becomes a lightweight libp2p peer with mDNS, identify, and request-response protocols
2. **Request-Response for Queries**: Direct peer-to-peer queries instead of HTTP or centralized APIs
3. **Automatic Discovery**: mDNS for zero-configuration node discovery on local networks
4. **Background Event Loop**: Async event processing for P2P events while CLI commands execute
5. **JSON Serialization**: Simple, debuggable serialization for queries and responses

## Architecture

### Component Diagram

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
│                           │                 │              │
│                           │  Event Loop     │              │
│                           │  (background)   │              │
│                           └─────────────────┘              │
└─────────────────────────────────────────────────────────────┘
                                  │
                                  │ P2P Network
                                  │ (libp2p)
                                  │
┌─────────────────────────────────────────────────────────────┐
│                        mvp-node                              │
│                                                              │
│  ┌─────────────────┐         ┌──────────────┐              │
│  │  QueryHandler   │◀────────│  P2PNetwork  │              │
│  │                 │         │              │              │
│  │ - handle_query()│         │  ┌────────┐  │              │
│  │ - get_node_info │         │  │ Swarm  │  │              │
│  │ - get_metrics   │         │  │        │  │              │
│  │ - get_health    │         │  │ - mDNS │  │              │
│  │ - get_job_status│         │  │ - req  │  │              │
│  └─────────────────┘         │  │ - res  │  │              │
│                              │  └────────┘  │              │
│                              └──────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### Sequence Diagram: Node Discovery and Query

```
CLI                 mDNS              Node              QueryHandler
 │                   │                 │                     │
 │──start()─────────▶│                 │                     │
 │                   │                 │                     │
 │                   │◀──announce──────│                     │
 │◀──discovered──────│                 │                     │
 │                   │                 │                     │
 │──query(GetNodeInfo)────────────────▶│                     │
 │                   │                 │──handle_query()────▶│
 │                   │                 │◀──NodeInfo──────────│
 │◀──QueryResponse(NodeInfo)───────────│                     │
 │                   │                 │                     │
```

## Components and Interfaces

### 1. Query Protocol Types

Location: `mvp-node/src/protocol/queries.rs` (new file)

```rust
use serde::{Deserialize, Serialize};

/// Query types for request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Query {
    GetNodeInfo { node_id: String },
    GetNodeMetrics { node_id: String },
    GetNodeHealth { node_id: String },
    ListNodes,
    GetJobStatus { job_id: String },
    ListJobs { filter: Option<JobFilter> },
    GetJobResult { job_id: String },
}

/// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryResponse {
    NodeInfo(NodeInfo),
    NodeMetrics(NodeMetrics),
    NodeHealth(HealthStatus),
    NodeList(Vec<NodeInfo>),
    JobStatus(JobStatus),
    JobList(Vec<JobInfo>),
    JobResult(JobResult),
    Error { message: String, code: String },
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub status: String,
    pub models: Vec<String>,
    pub uptime_seconds: u64,
    pub peer_count: usize,
}

// Additional types...
```


### 2. QueryCodec (Serialization)

Location: `mvp-node/src/protocol/query_codec.rs` (new file)

```rust
use libp2p::request_response::Codec;
use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncWrite};

/// Codec for Query/QueryResponse serialization
#[derive(Debug, Clone)]
pub struct QueryCodec;

#[async_trait]
impl Codec for QueryCodec {
    type Protocol = String;
    type Request = Query;
    type Response = QueryResponse;
    
    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) 
        -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Read length-prefixed JSON
        let mut length_bytes = [0u8; 4];
        io.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        // Enforce max size (1MB)
        if length > 1_048_576 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Query too large"
            ));
        }
        
        let mut buffer = vec![0u8; length];
        io.read_exact(&mut buffer).await?;
        
        serde_json::from_slice(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    
    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) 
        -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Similar to read_request
    }
    
    async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) 
        -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let length = (data.len() as u32).to_be_bytes();
        io.write_all(&length).await?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }
    
    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, res: Self::Response) 
        -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // Similar to write_request
    }
}
```

### 3. P2PBehaviour with Request-Response

Location: `mvp-node/src/network/mod.rs` (modify existing)

```rust
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::StreamProtocol;

#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response: request_response::Behaviour<QueryCodec>,  // NEW
}

impl P2PNetworkLayer {
    pub async fn new(keypair: Keypair) -> Result<Self> {
        // ... existing code ...
        
        // Create request-response behaviour
        let request_response = request_response::Behaviour::new(
            QueryCodec,
            [(
                StreamProtocol::new("/mvp/query/1.0.0"),
                ProtocolSupport::Full
            )],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(30)),
        );
        
        let behaviour = P2PBehaviour {
            gossipsub,
            mdns,
            identify,
            request_response,  // NEW
        };
        
        // ... rest of code ...
    }
}
```

### 4. QueryHandler (Node Side)

Location: `mvp-node/src/query_handler.rs` (new file)

```rust
use crate::orchestration::LLMOrchestrator;
use crate::monitoring::DefaultHealthMonitor;
use crate::job_queue::JobQueue;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handles incoming queries from CLI and other peers
pub struct QueryHandler {
    orchestrator: Arc<LLMOrchestrator>,
    health_monitor: Arc<DefaultHealthMonitor>,
    job_queue: Arc<RwLock<JobQueue>>,
    node_id: String,
    peer_id: PeerId,
}

impl QueryHandler {
    pub fn new(
        orchestrator: Arc<LLMOrchestrator>,
        health_monitor: Arc<DefaultHealthMonitor>,
        job_queue: Arc<RwLock<JobQueue>>,
        node_id: String,
        peer_id: PeerId,
    ) -> Self {
        Self {
            orchestrator,
            health_monitor,
            job_queue,
            node_id,
            peer_id,
        }
    }
    
    /// Handle an incoming query
    pub async fn handle_query(&self, query: Query) -> QueryResponse {
        match query {
            Query::GetNodeInfo { .. } => {
                self.handle_get_node_info().await
            }
            Query::GetNodeMetrics { .. } => {
                self.handle_get_metrics().await
            }
            Query::GetNodeHealth { .. } => {
                self.handle_get_health().await
            }
            Query::GetJobStatus { job_id } => {
                self.handle_get_job_status(&job_id).await
            }
            Query::ListJobs { filter } => {
                self.handle_list_jobs(filter).await
            }
            _ => {
                QueryResponse::Error {
                    message: "Unsupported query type".to_string(),
                    code: "UNSUPPORTED_QUERY".to_string(),
                }
            }
        }
    }
    
    async fn handle_get_node_info(&self) -> QueryResponse {
        // Collect node information
        let node_info = NodeInfo {
            node_id: self.node_id.clone(),
            peer_id: self.peer_id.to_string(),
            addresses: vec![], // Get from network layer
            status: "online".to_string(),
            models: vec!["tinyllama".to_string()],
            uptime_seconds: 0, // Get from health monitor
            peer_count: 0, // Get from network layer
        };
        
        QueryResponse::NodeInfo(node_info)
    }
    
    // Additional handlers...
}
```


### 5. NodeClient (CLI Side)

Location: `mvp-cli/src/client/mod.rs` (replace existing stub)

```rust
use libp2p::{
    identify, mdns, noise, tcp, yamux,
    request_response::{self, OutboundRequestId},
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;

/// CLI P2P Behaviour
#[derive(NetworkBehaviour)]
pub struct CliP2PBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response: request_response::Behaviour<QueryCodec>,
}

/// Node client for P2P communication
pub struct NodeClient {
    swarm: Swarm<CliP2PBehaviour>,
    discovered_nodes: HashMap<PeerId, NodeInfo>,
    pending_queries: HashMap<OutboundRequestId, oneshot::Sender<QueryResponse>>,
    event_tx: mpsc::Sender<ClientEvent>,
    event_rx: Option<mpsc::Receiver<ClientEvent>>,
}

/// Events emitted by the client
#[derive(Debug, Clone)]
pub enum ClientEvent {
    NodeDiscovered(PeerId, NodeInfo),
    NodeExpired(PeerId),
    QueryResponse(OutboundRequestId, QueryResponse),
    ConnectionEstablished(PeerId),
    ConnectionClosed(PeerId),
}

impl NodeClient {
    /// Create a new P2P-enabled CLI client
    pub async fn new() -> CliResult<Self> {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        info!("CLI peer ID: {}", peer_id);
        
        // Create event channel
        let (event_tx, event_rx) = mpsc::channel(100);
        
        // Create behaviours
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id
        )?;
        
        let identify = identify::Behaviour::new(identify::Config::new(
            "/mvp-cli/1.0.0".to_string(),
            keypair.public(),
        ));
        
        let request_response = request_response::Behaviour::new(
            QueryCodec,
            [(
                StreamProtocol::new("/mvp/query/1.0.0"),
                ProtocolSupport::Full
            )],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(30)),
        );
        
        let behaviour = CliP2PBehaviour {
            mdns,
            identify,
            request_response,
        };
        
        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();
        
        Ok(Self {
            swarm,
            discovered_nodes: HashMap::new(),
            pending_queries: HashMap::new(),
            event_tx,
            event_rx: Some(event_rx),
        })
    }
    
    /// Start listening on ephemeral port
    pub async fn start(&mut self) -> CliResult<()> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        
        // Wait for listener to be established
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("CLI listening on {}", address);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Send a query to a specific peer
    pub async fn query_peer(
        &mut self,
        peer_id: PeerId,
        query: Query,
    ) -> CliResult<QueryResponse> {
        let (tx, rx) = oneshot::channel();
        
        let request_id = self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, query);
        
        self.pending_queries.insert(request_id, tx);
        
        // Wait for response with timeout
        tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| CliError::Timeout)?
            .map_err(|_| CliError::ConnectionError("Query cancelled".to_string()))
    }
    
    /// Get list of discovered nodes
    pub fn discovered_nodes(&self) -> Vec<&NodeInfo> {
        self.discovered_nodes.values().collect()
    }
    
    /// Find a node by ID
    pub fn find_node(&self, node_id: &str) -> Option<&NodeInfo> {
        self.discovered_nodes.values()
            .find(|n| n.node_id == node_id)
    }
    
    /// Run the event loop (should be spawned as background task)
    pub async fn run_event_loop(&mut self) {
        loop {
            let event = self.swarm.select_next_some().await;
            self.handle_event(event).await;
        }
    }
    
    async fn handle_event(&mut self, event: SwarmEvent<CliP2PBehaviourEvent>) {
        match event {
            // mDNS discovery
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::Mdns(
                mdns::Event::Discovered(peers)
            )) => {
                for (peer_id, _addr) in peers {
                    info!("Discovered node: {}", peer_id);
                    
                    // Automatically query node info
                    if let Ok(response) = self.query_peer(
                        peer_id,
                        Query::GetNodeInfo { node_id: peer_id.to_string() }
                    ).await {
                        if let QueryResponse::NodeInfo(node_info) = response {
                            self.discovered_nodes.insert(peer_id, node_info.clone());
                            let _ = self.event_tx.send(
                                ClientEvent::NodeDiscovered(peer_id, node_info)
                            ).await;
                        }
                    }
                }
            }
            
            // mDNS expiry
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::Mdns(
                mdns::Event::Expired(peers)
            )) => {
                for (peer_id, _) in peers {
                    info!("Node expired: {}", peer_id);
                    self.discovered_nodes.remove(&peer_id);
                    let _ = self.event_tx.send(ClientEvent::NodeExpired(peer_id)).await;
                }
            }
            
            // Request-response
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::RequestResponse(
                request_response::Event::Response { request_id, response, .. }
            )) => {
                if let Some(tx) = self.pending_queries.remove(&request_id) {
                    let _ = tx.send(response);
                }
            }
            
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure { request_id, error, .. }
            )) => {
                warn!("Query failed: {:?}", error);
                if let Some(tx) = self.pending_queries.remove(&request_id) {
                    let _ = tx.send(QueryResponse::Error {
                        message: format!("Query failed: {:?}", error),
                        code: "QUERY_FAILED".to_string(),
                    });
                }
            }
            
            _ => {}
        }
    }
}
```

## Data Models

### Query and Response Types

All query and response types are defined in `protocol/queries.rs` and use JSON serialization for simplicity and debuggability.

**Key Design Choices:**
- Tagged enum for type safety
- snake_case for JSON compatibility
- Optional fields for flexibility
- Error responses include code and message

### Connection Management

The CLI maintains:
- `discovered_nodes: HashMap<PeerId, NodeInfo>` - Discovered nodes with their info
- `pending_queries: HashMap<OutboundRequestId, oneshot::Sender<QueryResponse>>` - Pending queries awaiting responses
- Maximum 50 concurrent connections (enforced by libp2p config)
- 60-second idle timeout for connections


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Query Response Timeliness

*For any* valid Query sent to a responsive node, the node SHALL respond with a QueryResponse within 30 seconds.

**Validates: Requirements 1.1**

### Property 2: Timeout Error Handling

*For any* Query that does not receive a response within 30 seconds, the requesting peer SHALL receive a timeout error.

**Validates: Requirements 1.2**

### Property 3: Invalid Query Error Response

*For any* invalid or malformed Query received by a node, the node SHALL respond with an error QueryResponse containing an error message and code.

**Validates: Requirements 1.4**

### Property 4: Unique PeerId Generation

*For any* two separate executions of the CLI, the generated PeerIds SHALL be different.

**Validates: Requirements 2.3**

### Property 5: Automatic NodeInfo Query on Discovery

*For any* node discovered via mDNS, the CLI SHALL automatically send a GetNodeInfo query to that node.

**Validates: Requirements 2.4**

### Property 6: Discovered Nodes List Maintenance

*For any* node discovered via mDNS, the CLI SHALL add it to the discovered_nodes list and maintain it until the node expires.

**Validates: Requirements 2.5**

### Property 7: New Node Discovery Timeliness

*For any* new node that joins the network, the CLI SHALL discover it via mDNS within 10 seconds.

**Validates: Requirements 3.2**

### Property 8: Node Expiry Cleanup

*For any* node that leaves the network, the CLI SHALL remove it from the discovered_nodes list within 30 seconds.

**Validates: Requirements 3.3**

### Property 9: Discovered Node Information Completeness

*For any* node in the discovered_nodes list, the NodeInfo SHALL contain: peer_id, node_id, addresses, status, and capabilities.

**Validates: Requirements 3.4**

### Property 10: Node Query Response Time

*For any* GetNodeInfo, GetNodeMetrics, or GetNodeHealth query received by a node, the node SHALL respond within 1 second.

**Validates: Requirements 6.1, 6.2, 6.3**

### Property 11: Job Status Query Response

*For any* GetJobStatus query with a job_id, the node SHALL respond with either the job status or an error indicating the job was not found.

**Validates: Requirements 6.4**

### Property 12: Unsupported Query Error Response

*For any* unsupported query type received by a node, the node SHALL respond with an error QueryResponse.

**Validates: Requirements 6.5**

### Property 13: Event Loop Event Handling

*For any* mDNS, request-response, or connection event, the CLI event loop SHALL process the event without blocking.

**Validates: Requirements 7.2, 7.3, 7.4**

### Property 14: Query Failure Logging

*For any* query that fails, the CLI SHALL log the error details including peer_id, query type, and error message.

**Validates: Requirements 8.5**

### Property 15: Serialization Error Handling

*For any* Query or QueryResponse that fails to serialize, the system SHALL return a serialization error.

**Validates: Requirements 9.2**

### Property 16: Deserialization Error Handling

*For any* data that fails to deserialize into a Query or QueryResponse, the system SHALL return a deserialization error with details.

**Validates: Requirements 9.3**

### Property 17: Query Size Limit Enforcement

*For any* Query with serialized size exceeding 1MB, the system SHALL reject it with a size limit error.

**Validates: Requirements 9.5**

### Property 18: Connection Limit Enforcement

*For any* state of the CLI, the number of concurrent connections SHALL NOT exceed 50.

**Validates: Requirements 10.1**

### Property 19: Connection Prioritization

*For any* situation where the connection limit is reached, the CLI SHALL prioritize connections to recently queried nodes over idle connections.

**Validates: Requirements 10.2**

### Property 20: Idle Connection Cleanup

*For any* connection that has been idle for 60 seconds, the CLI SHALL close it.

**Validates: Requirements 10.3**

### Property 21: Connection Retry with Backoff

*For any* connection failure, the CLI SHALL retry up to 3 times with exponential backoff (1s, 2s, 4s).

**Validates: Requirements 10.4**


## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query timeout: node {0} did not respond within 30 seconds")]
    Timeout(PeerId),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Query size exceeds limit: {0} bytes (max 1MB)")]
    QueryTooLarge(usize),
    
    #[error("P2P network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}
```

### Error Handling Strategy

1. **Timeouts**: All queries have a 30-second timeout. On timeout, return `CliError::Timeout` with peer ID
2. **Connection Failures**: Retry up to 3 times with exponential backoff (1s, 2s, 4s)
3. **Serialization Errors**: Return immediately with descriptive error message
4. **Node Not Found**: Check discovered_nodes list before querying, return helpful error if not found
5. **Network Errors**: Log detailed error information and return user-friendly message

### User-Facing Error Messages

- **Timeout**: "Request timed out: node 12D3K...abc1 did not respond within 30 seconds"
- **Not Found**: "Node not found: node-123. Use 'mvp-cli node list' to see available nodes."
- **No Nodes**: "No nodes discovered. Make sure nodes are running on the network."
- **Network Failure**: "Failed to start P2P network: {error}. Check your network configuration."

## Testing Strategy

### Unit Tests

Unit tests will verify specific examples and edge cases:

1. **Query Serialization**: Test each query type serializes to valid JSON
2. **Response Deserialization**: Test each response type deserializes correctly
3. **Error Responses**: Test error responses contain required fields
4. **Size Limits**: Test queries exceeding 1MB are rejected
5. **Timeout Handling**: Test timeout errors are returned after 30 seconds
6. **Connection Limits**: Test connection limit of 50 is enforced
7. **Idle Cleanup**: Test idle connections are closed after 60 seconds

### Property-Based Tests

Property-based tests will verify universal properties across all inputs:

1. **Property 1-21**: Each correctness property will have a corresponding property-based test
2. **Minimum 100 iterations** per property test
3. **Random input generation** for queries, responses, peer IDs, etc.
4. **Invariant checking** for connection limits, timeouts, and state consistency

### Integration Tests

Integration tests will verify end-to-end behavior:

1. **CLI ↔ Node Communication**: Start a node and CLI, verify discovery and queries work
2. **Multiple Nodes**: Start multiple nodes, verify CLI discovers all of them
3. **Node Failure**: Stop a node, verify CLI removes it from discovered list
4. **Job Submission**: Submit a job via CLI, verify it's received by a node
5. **Concurrent Queries**: Send multiple queries simultaneously, verify all responses are received

### Test Configuration

- **Property tests**: 100 iterations minimum
- **Timeout tests**: Use shorter timeouts (5s) for faster test execution
- **Integration tests**: Use ephemeral ports to avoid conflicts
- **Mocking**: Mock libp2p behaviours for unit tests, use real P2P for integration tests

### Test Tags

All property-based tests will be tagged with:
```rust
// Feature: cli-p2p-integration, Property 1: Query Response Timeliness
#[test]
fn property_query_response_timeliness() {
    // Test implementation
}
```

