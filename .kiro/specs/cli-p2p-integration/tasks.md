# Implementation Plan: CLI P2P Integration

## Overview

This plan implements full P2P integration for the CLI, transforming it from stub implementation to a real P2P peer. The implementation is divided into 6 main phases with incremental progress and testing at each step.

## Tasks

- [ ] 1. Create Query Protocol Types
  - Create `mvp-node/src/protocol/queries.rs` with Query and QueryResponse enums
  - Define NodeInfo, NodeMetrics, HealthStatus, JobStatus, JobInfo, JobResult types
  - Add JobFilter type for filtering job lists
  - Implement serialization/deserialization with serde
  - Add version field to Query for protocol compatibility
  - _Requirements: 1.3, 9.1, 9.4_

- [ ] 1.1 Write unit tests for query types
  - Test serialization of each Query variant to JSON
  - Test deserialization of each QueryResponse variant from JSON
  - Test error response serialization
  - Test version field is included in serialized queries
  - _Requirements: 9.1, 9.4_

- [ ] 2. Implement QueryCodec for Serialization
  - Create `mvp-node/src/protocol/query_codec.rs`
  - Implement Codec trait for Query/QueryResponse
  - Use length-prefixed JSON format (4-byte length + JSON data)
  - Enforce 1MB maximum query size
  - Handle serialization/deserialization errors gracefully
  - _Requirements: 1.5, 9.1, 9.2, 9.3, 9.5_

- [ ] 2.1 Write unit tests for QueryCodec
  - Test serialization and deserialization round-trip
  - Test queries exceeding 1MB are rejected
  - Test serialization errors are handled
  - Test deserialization errors are handled with details
  - _Requirements: 9.2, 9.3, 9.5_

- [ ] 2.2 Write property test for serialization round-trip
  - **Property 15 & 16: Serialization/Deserialization Error Handling**
  - **Validates: Requirements 9.2, 9.3**

- [ ] 2.3 Write property test for size limit enforcement
  - **Property 17: Query Size Limit Enforcement**
  - **Validates: Requirements 9.5**

- [ ] 3. Add Request-Response to P2PBehaviour
  - Modify `mvp-node/src/network/mod.rs`
  - Add request_response field to P2PBehaviour struct
  - Create request-response behaviour with QueryCodec
  - Use protocol ID "/mvp/query/1.0.0"
  - Configure 30-second request timeout
  - Handle request-response events in event loop
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 3.1 Write unit tests for P2PBehaviour initialization
  - Test request-response behaviour is created with correct protocol ID
  - Test timeout is configured to 30 seconds
  - _Requirements: 1.5_

- [ ] 4. Implement QueryHandler in Node
  - Create `mvp-node/src/query_handler.rs`
  - Implement QueryHandler struct with orchestrator, health_monitor, job_queue references
  - Implement handle_query() method to route queries to appropriate handlers
  - Implement handle_get_node_info() to return node information
  - Implement handle_get_metrics() to return node metrics
  - Implement handle_get_health() to return health status
  - Implement handle_get_job_status() to return job status or error
  - Implement handle_list_jobs() to return filtered job list
  - Return error responses for unsupported query types
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 4.1 Write unit tests for QueryHandler
  - Test each query type returns appropriate response
  - Test unsupported queries return error response
  - Test job not found returns error response
  - _Requirements: 6.4, 6.5_

- [ ] 4.2 Write property test for query response time
  - **Property 10: Node Query Response Time**
  - **Validates: Requirements 6.1, 6.2, 6.3**

- [ ] 4.3 Write property test for job status query
  - **Property 11: Job Status Query Response**
  - **Validates: Requirements 6.4**

- [ ] 4.4 Write property test for unsupported query error
  - **Property 12: Unsupported Query Error Response**
  - **Validates: Requirements 6.5**


- [ ] 5. Integrate QueryHandler with Node Event Loop
  - Modify `mvp-node/src/node/mod.rs` (or main event loop)
  - Create QueryHandler instance with required dependencies
  - Handle RequestResponse::Message events in P2P event loop
  - Call query_handler.handle_query() for incoming requests
  - Send QueryResponse back via request-response protocol
  - Log query handling for debugging
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 5.1 Write integration test for node query handling
  - Start a node with QueryHandler
  - Send GetNodeInfo query via P2P
  - Verify response is received within 1 second
  - _Requirements: 6.1_

- [ ] 6. Checkpoint - Verify Node Side Implementation
  - Ensure all tests pass
  - Verify nodes can respond to queries
  - Ask user if questions arise

- [ ] 7. Implement CLI P2P Behaviour
  - Create CliP2PBehaviour in `mvp-cli/src/client/mod.rs`
  - Add mdns, identify, and request_response fields
  - Implement NetworkBehaviour derive
  - _Requirements: 2.1_

- [ ] 7.1 Write unit test for CliP2PBehaviour creation
  - Test behaviour is created with all required protocols
  - _Requirements: 2.1_

- [ ] 8. Implement NodeClient with Real P2P Swarm
  - Replace stub implementation in `mvp-cli/src/client/mod.rs`
  - Create Swarm with CliP2PBehaviour
  - Generate unique PeerId on each execution
  - Listen on ephemeral port (port 0)
  - Implement query_peer() method using request-response
  - Maintain discovered_nodes HashMap
  - Maintain pending_queries HashMap for tracking requests
  - Implement 30-second timeout for queries
  - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [ ] 8.1 Write unit tests for NodeClient
  - Test NodeClient creates unique PeerId
  - Test NodeClient listens on ephemeral port
  - Test query_peer() sends request and waits for response
  - Test query timeout after 30 seconds
  - _Requirements: 2.2, 2.3_

- [ ] 8.2 Write property test for unique PeerId generation
  - **Property 4: Unique PeerId Generation**
  - **Validates: Requirements 2.3**

- [ ] 8.3 Write property test for query timeout
  - **Property 1 & 2: Query Response Timeliness and Timeout Error Handling**
  - **Validates: Requirements 1.1, 1.2**

- [ ] 9. Implement CLI Event Loop
  - Implement run_event_loop() method in NodeClient
  - Handle mDNS discovery events (Discovered and Expired)
  - Handle request-response events (Response and OutboundFailure)
  - Handle connection events (ConnectionEstablished and ConnectionClosed)
  - Automatically query NodeInfo when node is discovered
  - Update discovered_nodes HashMap on discovery/expiry
  - Send responses to pending_queries via oneshot channels
  - _Requirements: 2.4, 2.5, 3.2, 3.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 9.1 Write unit tests for event handling
  - Test mDNS discovery adds node to discovered_nodes
  - Test mDNS expiry removes node from discovered_nodes
  - Test request-response events resolve pending queries
  - _Requirements: 2.5, 3.3_

- [ ] 9.2 Write property test for automatic NodeInfo query
  - **Property 5: Automatic NodeInfo Query on Discovery**
  - **Validates: Requirements 2.4**

- [ ] 9.3 Write property test for discovered nodes list maintenance
  - **Property 6: Discovered Nodes List Maintenance**
  - **Validates: Requirements 2.5**

- [ ] 9.4 Write property test for node expiry cleanup
  - **Property 8: Node Expiry Cleanup**
  - **Validates: Requirements 3.3**

- [ ] 9.5 Write property test for event loop event handling
  - **Property 13: Event Loop Event Handling**
  - **Validates: Requirements 7.2, 7.3, 7.4**

- [ ] 10. Implement Connection Management
  - Configure Swarm with max 50 connections
  - Implement 60-second idle connection timeout
  - Implement connection retry with exponential backoff (1s, 2s, 4s, max 3 retries)
  - Implement connection prioritization for recently queried nodes
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 10.1 Write unit tests for connection management
  - Test connection limit is enforced
  - Test idle connections are closed after 60 seconds
  - Test connection retry with backoff
  - _Requirements: 10.1, 10.3, 10.4_

- [ ] 10.2 Write property test for connection limit enforcement
  - **Property 18: Connection Limit Enforcement**
  - **Validates: Requirements 10.1**

- [ ] 10.3 Write property test for connection prioritization
  - **Property 19: Connection Prioritization**
  - **Validates: Requirements 10.2**

- [ ] 10.4 Write property test for idle connection cleanup
  - **Property 20: Idle Connection Cleanup**
  - **Validates: Requirements 10.3**

- [ ] 10.5 Write property test for connection retry
  - **Property 21: Connection Retry with Backoff**
  - **Validates: Requirements 10.4**

- [ ] 11. Checkpoint - Verify CLI P2P Implementation
  - Ensure all tests pass
  - Verify CLI can discover nodes and send queries
  - Ask user if questions arise

- [ ] 12. Update CLI Commands to Use Real P2P
  - Modify `mvp-cli/src/commands/node.rs`
  - Update list_nodes() to use NodeClient.discovered_nodes()
  - Update get_node_info() to use NodeClient.query_peer(GetNodeInfo)
  - Update check_health() to use NodeClient.query_peer(GetNodeHealth)
  - Update show_metrics() to use NodeClient.query_peer(GetNodeMetrics)
  - Remove all hardcoded/fake data
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 12.1 Write integration tests for node commands
  - Test `node list` displays discovered nodes
  - Test `node info` queries and displays node information
  - Test `node health` queries and displays health status
  - Test `node metrics` queries and displays metrics
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 13. Update Job Commands to Use P2P
  - Modify `mvp-cli/src/commands/job.rs`
  - Update submit_job() to publish WorkOffer via gossipsub
  - Update get_job_status() to use NodeClient.query_peer(GetJobStatus)
  - Update get_job_result() to use NodeClient.query_peer(GetJobResult)
  - Update list_jobs() to use NodeClient.query_peer(ListJobs)
  - Remove all hardcoded/fake data
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 13.1 Write integration tests for job commands
  - Test `job submit` publishes WorkOffer message
  - Test `job status` queries job status from node
  - Test `job result` retrieves job result from node
  - Test `job list` queries and displays all jobs
  - _Requirements: 5.1, 5.3, 5.4, 5.5_


- [ ] 14. Implement Error Handling and User Messages
  - Create CliError enum in `mvp-cli/src/error.rs`
  - Add Timeout, NodeNotFound, SerializationError, DeserializationError variants
  - Implement user-friendly error messages
  - Add error logging for debugging
  - Handle timeout errors with peer ID in message
  - Handle node not found errors with helpful suggestions
  - Handle no nodes discovered with helpful message
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 14.1 Write unit tests for error handling
  - Test timeout error message format
  - Test node not found error message format
  - Test no nodes discovered message format
  - Test error logging includes required details
  - _Requirements: 8.1, 8.2, 8.4, 8.5_

- [ ] 14.2 Write property test for query failure logging
  - **Property 14: Query Failure Logging**
  - **Validates: Requirements 8.5**

- [ ] 15. Implement CLI Startup and Shutdown
  - Modify `mvp-cli/src/main.rs` or create CLI runner
  - Initialize NodeClient on CLI startup
  - Start NodeClient listening on ephemeral port
  - Spawn background task for event loop
  - Wait for initial node discovery (2-5 seconds)
  - Implement graceful shutdown on CLI exit
  - Close all connections on shutdown
  - _Requirements: 2.1, 2.2, 3.1, 7.1, 7.5_

- [ ] 15.1 Write integration test for CLI startup
  - Test CLI starts and discovers nodes within 5 seconds
  - Test CLI generates unique PeerId
  - Test CLI listens on ephemeral port
  - _Requirements: 2.2, 2.3, 3.1_

- [ ] 15.2 Write integration test for CLI shutdown
  - Test CLI closes all connections gracefully
  - Test event loop stops on shutdown
  - _Requirements: 7.5_

- [ ] 16. Add Verbose Mode and Logging
  - Add --verbose flag to CLI commands
  - Log connection status in verbose mode
  - Log query requests and responses in verbose mode
  - Log discovery events in verbose mode
  - Use tracing crate for structured logging
  - _Requirements: 10.5_

- [ ] 16.1 Write unit tests for verbose logging
  - Test verbose mode logs connection status
  - Test verbose mode logs query details
  - _Requirements: 10.5_

- [ ] 17. Write Property Tests for Discovered Node Information
  - **Property 9: Discovered Node Information Completeness**
  - **Validates: Requirements 3.4**

- [ ] 18. Write Property Test for Invalid Query Error
  - **Property 3: Invalid Query Error Response**
  - **Validates: Requirements 1.4**

- [ ] 19. Write Property Test for New Node Discovery
  - **Property 7: New Node Discovery Timeliness**
  - **Validates: Requirements 3.2**

- [ ] 20. Write End-to-End Integration Tests
  - Test CLI discovers multiple nodes
  - Test CLI can query each discovered node
  - Test CLI handles node failure gracefully
  - Test CLI handles network partition
  - Test concurrent queries to multiple nodes
  - Test job submission and status query flow
  - _Requirements: All_

- [ ] 21. Final Checkpoint - Complete Integration Testing
  - Run all unit tests and verify they pass
  - Run all property tests (100+ iterations each) and verify they pass
  - Run all integration tests and verify they pass
  - Test CLI with real nodes in local network
  - Verify no hardcoded/fake data remains
  - Ensure all tests pass, ask the user if questions arise

- [ ] 22. Update Documentation
  - Update CLI README with P2P usage instructions
  - Document query protocol in protocol documentation
  - Add examples of CLI commands with real P2P
  - Document error messages and troubleshooting
  - Update architecture diagrams to show P2P integration
  - _Requirements: All_

## Notes

- All property tests should run with minimum 100 iterations
- Integration tests should use ephemeral ports to avoid conflicts
- Tests should be tagged with feature name and property number
- Each task builds on previous tasks - follow order for incremental progress
- Checkpoints ensure validation before proceeding to next phase
- All hardcoded/fake data must be removed from CLI implementation

