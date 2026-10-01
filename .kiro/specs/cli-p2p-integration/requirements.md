# Requirements Document: CLI P2P Integration

## Introduction

Este documento especifica os requisitos para integrar o CLI (`mvp-cli`) com a rede P2P, permitindo comunicação real entre CLI e nodes. Atualmente, o CLI retorna apenas dados simulados/hardcoded. Esta spec implementa a solução para os problemas #1, #4, #6 e #8 identificados em `docs/INTEGRATION-ISSUES.md`.

## Glossary

- **CLI**: Command-Line Interface - ferramenta de linha de comando para interagir com nodes (`mvp-cli`)
- **Node**: Daemon P2P que executa inferência de IA (`mvp-node`)
- **P2P_Network**: Rede peer-to-peer baseada em libp2p
- **Request_Response**: Protocolo libp2p para queries diretas entre peers
- **Gossipsub**: Protocolo libp2p para broadcast de mensagens (pub/sub)
- **mDNS**: Multicast DNS para descoberta automática de peers na rede local
- **Query**: Requisição direta de um peer para outro
- **QueryResponse**: Resposta a uma Query
- **Peer**: Participante da rede P2P (pode ser CLI ou Node)
- **PeerId**: Identificador único de um peer na rede P2P

## Requirements

### Requirement 1: Request-Response Protocol

**User Story:** As a developer, I want nodes to support direct queries via request-response protocol, so that CLI can query specific information from individual nodes.

#### Acceptance Criteria

1. WHEN a peer sends a Query to a node, THEN THE Node SHALL respond with a QueryResponse within 30 seconds
2. WHEN a Query times out, THEN THE requesting peer SHALL receive a timeout error
3. THE P2P_Network SHALL support the following query types: GetNodeInfo, GetNodeMetrics, GetNodeHealth, ListNodes, GetJobStatus, ListJobs, GetJobResult
4. WHEN a node receives an invalid Query, THEN THE Node SHALL respond with an error QueryResponse
5. THE Request_Response protocol SHALL use protocol ID "/mvp/query/1.0.0"

### Requirement 2: CLI as P2P Peer

**User Story:** As a user, I want the CLI to act as a P2P peer, so that it can discover and communicate with nodes without centralized infrastructure.

#### Acceptance Criteria

1. WHEN the CLI starts, THEN THE CLI SHALL create a libp2p Swarm with mDNS, identify, and request-response protocols
2. WHEN the CLI starts, THEN THE CLI SHALL listen on an ephemeral port (port 0)
3. THE CLI SHALL generate a unique PeerId on each execution
4. WHEN a node is discovered via mDNS, THEN THE CLI SHALL automatically query its NodeInfo
5. THE CLI SHALL maintain a list of discovered nodes with their information

### Requirement 3: Node Discovery

**User Story:** As a CLI user, I want to automatically discover nodes on the local network, so that I don't need to manually configure node addresses.

#### Acceptance Criteria

1. WHEN the CLI starts, THEN THE CLI SHALL discover nodes via mDNS within 5 seconds
2. WHEN a new node joins the network, THEN THE CLI SHALL discover it within 10 seconds
3. WHEN a node leaves the network, THEN THE CLI SHALL remove it from the discovered list within 30 seconds
4. THE CLI SHALL display discovered nodes with their PeerId, address, status, and capabilities
5. WHEN no nodes are discovered, THEN THE CLI SHALL display a helpful message indicating no nodes are available


### Requirement 4: Node Information Queries

**User Story:** As a CLI user, I want to query detailed information about specific nodes, so that I can monitor node status and capabilities.

#### Acceptance Criteria

1. WHEN I run `mvp-cli node info <node-id>`, THEN THE CLI SHALL query the node via request-response and display: node ID, peer ID, status, address, uptime, capabilities, and statistics
2. WHEN I run `mvp-cli node list`, THEN THE CLI SHALL display all discovered nodes with their basic information
3. WHEN I run `mvp-cli node health <node-id>`, THEN THE CLI SHALL query and display the node's health status including: network health, model health, queue health, memory usage, and CPU usage
4. WHEN I run `mvp-cli node metrics <node-id>`, THEN THE CLI SHALL query and display the node's metrics
5. WHEN a queried node is not available, THEN THE CLI SHALL display an error message with timeout information

### Requirement 5: Job Submission via P2P

**User Story:** As a CLI user, I want to submit jobs to nodes via P2P, so that I can execute inference tasks without HTTP APIs.

#### Acceptance Criteria

1. WHEN I run `mvp-cli job submit`, THEN THE CLI SHALL publish a WorkOffer message via gossipsub
2. WHEN a job is submitted, THEN THE CLI SHALL return a job ID immediately
3. WHEN I run `mvp-cli job status <job-id>`, THEN THE CLI SHALL query the job status from the executing node
4. WHEN I run `mvp-cli job result <job-id>`, THEN THE CLI SHALL retrieve the job result from the executing node
5. WHEN I run `mvp-cli job list`, THEN THE CLI SHALL query and display all jobs from all discovered nodes

### Requirement 6: Query Handler in Nodes

**User Story:** As a node operator, I want my node to respond to CLI queries, so that users can monitor and interact with my node.

#### Acceptance Criteria

1. WHEN a Node receives a GetNodeInfo query, THEN THE Node SHALL respond with current node information within 1 second
2. WHEN a Node receives a GetNodeMetrics query, THEN THE Node SHALL respond with current metrics within 1 second
3. WHEN a Node receives a GetNodeHealth query, THEN THE Node SHALL respond with health status within 1 second
4. WHEN a Node receives a GetJobStatus query, THEN THE Node SHALL respond with job status or error if job not found
5. WHEN a Node receives an unsupported query type, THEN THE Node SHALL respond with an error message

### Requirement 7: CLI Event Loop

**User Story:** As a developer, I want the CLI to run a background event loop, so that it can handle P2P events asynchronously.

#### Acceptance Criteria

1. WHEN the CLI starts, THEN THE CLI SHALL spawn a background task for the P2P event loop
2. THE CLI event loop SHALL handle mDNS discovery events
3. THE CLI event loop SHALL handle request-response events
4. THE CLI event loop SHALL handle connection events
5. WHEN the CLI exits, THEN THE CLI SHALL gracefully shutdown the P2P swarm

### Requirement 8: Error Handling and Timeouts

**User Story:** As a CLI user, I want clear error messages when operations fail, so that I can understand what went wrong.

#### Acceptance Criteria

1. WHEN a query times out after 30 seconds, THEN THE CLI SHALL display "Request timed out: node <peer-id> did not respond"
2. WHEN a node is not found, THEN THE CLI SHALL display "Node not found: <node-id>"
3. WHEN the P2P network fails to start, THEN THE CLI SHALL display a clear error message and exit gracefully
4. WHEN no nodes are discovered, THEN THE CLI SHALL display "No nodes discovered. Make sure nodes are running on the network."
5. WHEN a query fails, THEN THE CLI SHALL log the error details for debugging

### Requirement 9: Query Protocol Serialization

**User Story:** As a developer, I want queries to be efficiently serialized, so that network communication is fast and reliable.

#### Acceptance Criteria

1. THE Query and QueryResponse types SHALL be serialized using JSON
2. WHEN serialization fails, THEN THE system SHALL return a serialization error
3. WHEN deserialization fails, THEN THE system SHALL return a deserialization error with details
4. THE serialized Query SHALL include a version field for protocol compatibility
5. THE maximum Query size SHALL be 1MB to prevent abuse

### Requirement 10: CLI Connection Management

**User Story:** As a CLI user, I want the CLI to manage connections efficiently, so that it doesn't consume excessive resources.

#### Acceptance Criteria

1. THE CLI SHALL maintain at most 50 concurrent connections to nodes
2. WHEN the connection limit is reached, THEN THE CLI SHALL prioritize connections to recently queried nodes
3. THE CLI SHALL close idle connections after 60 seconds
4. WHEN a connection fails, THEN THE CLI SHALL retry up to 3 times with exponential backoff
5. THE CLI SHALL display connection status in verbose mode

