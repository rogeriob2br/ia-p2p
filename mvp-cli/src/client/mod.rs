//! Node Client for P2P Communication
//!
//! Provides real P2P communication with MVP Nodes via libp2p request-response protocol.
//!
//! Implements Requirements 2.1-2.5, 7.1-7.5, 10.1-10.5

use libp2p::{
    identify, mdns, noise, tcp, yamux,
    request_response::{self, OutboundRequestId, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm, StreamProtocol,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::error::{CliResult, CliError};

// Import protocol types from mvp-node
use mvp_node::protocol::{
    Query, QueryResponse, QueryCodec, NodeInfo, NodeMetrics,
    HealthStatus, JobStatusInfo, JobInfo, JobResultInfo,
    JobSubmitResponse, QUERY_PROTOCOL_ID,
};

/// Connection information for a peer
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub connected_at: std::time::Instant,
    pub node_info: Option<NodeInfo>,
}

/// CLI P2P Behaviour
#[derive(NetworkBehaviour)]
pub struct CliP2PBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response: request_response::Behaviour<QueryCodec>,
}

/// Events from the P2P client
#[derive(Debug)]
pub enum ClientEvent {
    /// A node was discovered via mDNS
    NodeDiscovered(PeerId, Vec<Multiaddr>),
    /// A node expired (no longer available)
    NodeExpired(PeerId),
    /// Connection established with a peer
    ConnectionEstablished(PeerId),
    /// Connection closed with a peer
    ConnectionClosed(PeerId),
}

/// Pending query waiting for response
struct PendingQuery {
    sender: oneshot::Sender<CliResult<QueryResponse>>,
}

/// Node client for real P2P communication
pub struct NodeClient {
    /// libp2p Swarm
    swarm: Swarm<CliP2PBehaviour>,
    /// Local peer ID
    peer_id: PeerId,
    /// Discovered nodes with their information
    discovered_nodes: HashMap<PeerId, ConnectionInfo>,
    /// Pending queries waiting for responses
    pending_queries: HashMap<OutboundRequestId, PendingQuery>,
    /// Event sender for external listeners
    event_tx: mpsc::Sender<ClientEvent>,
    /// Event receiver (can be taken)
    event_rx: Option<mpsc::Receiver<ClientEvent>>,
    /// Query timeout
    query_timeout: Duration,
    /// Whether the client is running
    is_running: bool,
}

impl NodeClient {
    /// Create a new P2P-enabled CLI client
    /// 
    /// Property 4: Unique PeerId Generation
    pub async fn new(timeout_secs: u64) -> CliResult<Self> {
        // Generate unique keypair for this CLI instance
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        info!("CLI peer ID: {}", peer_id);
        
        // Create event channel
        let (event_tx, event_rx) = mpsc::channel(100);
        
        // Create mDNS behaviour
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        ).map_err(|e| CliError::NetworkError(format!("mDNS init failed: {}", e)))?;
        
        // Create identify behaviour
        let identify = identify::Behaviour::new(identify::Config::new(
            "/mvp-cli/1.0.0".to_string(),
            keypair.public(),
        ));
        
        // Create request-response behaviour
        // Property 1: 30 second timeout for queries
        let request_response = request_response::Behaviour::new(
            [(
                StreamProtocol::new(QUERY_PROTOCOL_ID),
                ProtocolSupport::Full,
            )],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(30)),
        );
        
        let behaviour = CliP2PBehaviour {
            mdns,
            identify,
            request_response,
        };
        
        // Build swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| CliError::NetworkError(format!("Transport error: {}", e)))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| CliError::NetworkError(format!("Behaviour error: {}", e)))?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();
        
        Ok(Self {
            swarm,
            peer_id,
            discovered_nodes: HashMap::new(),
            pending_queries: HashMap::new(),
            event_tx,
            event_rx: Some(event_rx),
            query_timeout: Duration::from_secs(timeout_secs),
            is_running: false,
        })
    }
    
    /// Start listening on ephemeral port
    /// 
    /// Property 2.2: CLI listens on ephemeral port
    pub async fn start(&mut self) -> CliResult<Multiaddr> {
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()
            .map_err(|e| CliError::ParseError(format!("Invalid address: {}", e)))?;
        
        self.swarm.listen_on(listen_addr)
            .map_err(|e| CliError::NetworkError(format!("Listen failed: {}", e)))?;
        
        self.is_running = true;
        
        // Wait for listener to be established
        let start = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(5);
        
        while start.elapsed() < timeout_duration {
            if let Some(event) = self.poll_event().await {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    info!("CLI listening on {}", address);
                    return Ok(address);
                }
            }
        }
        
        Err(CliError::NetworkError("Failed to establish listener".to_string()))
    }
    
    /// Take the event receiver (can only be called once)
    pub fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<ClientEvent>> {
        self.event_rx.take()
    }
    
    /// Get local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.peer_id
    }
    
    /// Get list of discovered nodes
    pub fn discovered_nodes(&self) -> Vec<&ConnectionInfo> {
        self.discovered_nodes.values().collect()
    }
    
    /// Find a node by peer ID
    pub fn find_node_by_peer_id(&self, peer_id: &PeerId) -> Option<&ConnectionInfo> {
        self.discovered_nodes.get(peer_id)
    }
    
    /// Find a node by node ID
    pub fn find_node_by_id(&self, node_id: &str) -> Option<(&PeerId, &ConnectionInfo)> {
        self.discovered_nodes.iter()
            .find(|(_, info)| {
                info.node_info.as_ref()
                    .map(|n| n.node_id == node_id)
                    .unwrap_or(false)
            })
    }
    
    /// Send a query to a peer and wait for response
    /// 
    /// Property 1 & 2: Query Response Timeliness and Timeout
    pub async fn query_peer(&mut self, peer_id: &PeerId, query: Query) -> CliResult<QueryResponse> {
        info!("Sending query to {}: {:?}", peer_id, query);
        
        // Create response channel
        let (tx, mut rx) = oneshot::channel();
        
        // Send request
        let request_id = self.swarm
            .behaviour_mut()
            .request_response
            .send_request(peer_id, query);
        
        self.pending_queries.insert(request_id, PendingQuery { sender: tx });
        
        // Wait for response with timeout
        let response = timeout(self.query_timeout, async {
            // Poll events until we get the response
            loop {
                self.poll_and_handle_events().await;
                
                // Check if response is ready
                match rx.try_recv() {
                    Ok(result) => return result,
                    Err(oneshot::error::TryRecvError::Empty) => {
                        // Continue polling
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    Err(oneshot::error::TryRecvError::Closed) => {
                        return Err(CliError::ConnectionError("Query cancelled".to_string()));
                    }
                }
            }
        }).await;
        
        match response {
            Ok(result) => result,
            Err(_) => {
                // Clean up pending query
                self.pending_queries.remove(&request_id);
                Err(CliError::Timeout(format!("Query to {} timed out", peer_id)))
            }
        }
    }
    
    /// Poll and handle a single event
    async fn poll_event(&mut self) -> Option<SwarmEvent<CliP2PBehaviourEvent>> {
        use futures::StreamExt;
        
        tokio::select! {
            event = self.swarm.select_next_some() => Some(event),
            _ = tokio::time::sleep(Duration::from_millis(10)) => None,
        }
    }
    
    /// Poll and handle events
    async fn poll_and_handle_events(&mut self) {
        if let Some(event) = self.poll_event().await {
            self.handle_swarm_event(event).await;
        }
    }
    
    /// Handle swarm events
    async fn handle_swarm_event(&mut self, event: SwarmEvent<CliP2PBehaviourEvent>) {
        match event {
            // mDNS discovery - Property 5 & 6
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::Mdns(
                mdns::Event::Discovered(peers)
            )) => {
                for (peer_id, addr) in peers {
                    if peer_id == self.peer_id {
                        continue; // Skip self
                    }
                    
                    info!("Discovered node: {} at {}", peer_id, addr);
                    
                    // Add to discovered nodes
                    let entry = self.discovered_nodes.entry(peer_id).or_insert(ConnectionInfo {
                        peer_id,
                        addresses: Vec::new(),
                        connected_at: std::time::Instant::now(),
                        node_info: None,
                    });
                    
                    if !entry.addresses.contains(&addr) {
                        entry.addresses.push(addr);
                    }
                    
                    // Notify listeners
                    let _ = self.event_tx.send(ClientEvent::NodeDiscovered(
                        peer_id,
                        entry.addresses.clone(),
                    )).await;
                }
            }
            
            // mDNS expiry - Property 8
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::Mdns(
                mdns::Event::Expired(peers)
            )) => {
                for (peer_id, _) in peers {
                    info!("Node expired: {}", peer_id);
                    self.discovered_nodes.remove(&peer_id);
                    let _ = self.event_tx.send(ClientEvent::NodeExpired(peer_id)).await;
                }
            }
            
            // Identify
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::Identify(
                identify::Event::Received { peer_id, info, .. }
            )) => {
                debug!("Identified peer {}: {}", peer_id, info.protocol_version);
                
                if let Some(conn) = self.discovered_nodes.get_mut(&peer_id) {
                    conn.addresses = info.listen_addrs;
                }
            }
            
            // Request-Response: Response received
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::RequestResponse(
                request_response::Event::Message { message, peer, .. }
            )) => {
                if let request_response::Message::Response { request_id, response } = message {
                    debug!("Received response from {} for {:?}", peer, request_id);
                    
                    if let Some(pending) = self.pending_queries.remove(&request_id) {
                        let _ = pending.sender.send(Ok(response));
                    }
                }
            }
            
            // Request-Response: Outbound failure
            SwarmEvent::Behaviour(CliP2PBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure { request_id, error, peer, .. }
            )) => {
                warn!("Query to {} failed: {:?}", peer, error);
                
                if let Some(pending) = self.pending_queries.remove(&request_id) {
                    let _ = pending.sender.send(Err(CliError::ConnectionError(
                        format!("Query failed: {:?}", error)
                    )));
                }
            }
            
            // Connection established
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connection established with {}", peer_id);
                let _ = self.event_tx.send(ClientEvent::ConnectionEstablished(peer_id)).await;
            }
            
            // Connection closed
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Connection closed with {}", peer_id);
                let _ = self.event_tx.send(ClientEvent::ConnectionClosed(peer_id)).await;
            }
            
            // New listen address
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            
            _ => {}
        }
    }
    
    /// Run event loop for a short duration to process discovery events
    pub async fn poll_for_duration(&mut self, duration: Duration) {
        let start = std::time::Instant::now();
        while start.elapsed() < duration && self.is_running {
            self.poll_and_handle_events().await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    
    /// Dial a specific peer address
    pub fn dial(&mut self, addr: &Multiaddr) -> CliResult<()> {
        self.swarm.dial(addr.clone())
            .map_err(|e| CliError::ConnectionError(format!("Dial failed: {}", e)))
    }
    
    /// Disconnect from a peer
    pub fn disconnect(&mut self, peer_id: &PeerId) {
        let _ = self.swarm.disconnect_peer_id(*peer_id);
        self.discovered_nodes.remove(peer_id);
    }
    
    /// Get number of connected peers
    pub fn connected_peers_count(&self) -> usize {
        self.discovered_nodes.len()
    }
    
    /// Stop the client
    pub fn stop(&mut self) {
        self.is_running = false;
        
        // Disconnect from all peers
        let peer_ids: Vec<_> = self.discovered_nodes.keys().cloned().collect();
        for peer_id in peer_ids {
            let _ = self.swarm.disconnect_peer_id(peer_id);
        }
        
        self.discovered_nodes.clear();
        self.pending_queries.clear();
    }
    
    // ========================================================================
    // High-level query methods
    // ========================================================================
    
    /// Get node information from a peer
    pub async fn get_node_info(&mut self, peer_id: &PeerId) -> CliResult<NodeInfo> {
        let query = Query::get_node_info(peer_id.to_string());
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::NodeInfo(info) => {
                // Update cached node info
                if let Some(conn) = self.discovered_nodes.get_mut(peer_id) {
                    conn.node_info = Some(info.clone());
                }
                Ok(info)
            }
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Get metrics from a peer
    pub async fn get_node_metrics(&mut self, peer_id: &PeerId) -> CliResult<NodeMetrics> {
        let query = Query::get_node_metrics(peer_id.to_string());
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::NodeMetrics(metrics) => Ok(metrics),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Get health status from a peer
    pub async fn get_node_health(&mut self, peer_id: &PeerId) -> CliResult<HealthStatus> {
        let query = Query::get_node_health(peer_id.to_string());
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::NodeHealth(health) => Ok(health),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Get job status from a peer
    pub async fn get_job_status(&mut self, peer_id: &PeerId, job_id: &str) -> CliResult<JobStatusInfo> {
        let query = Query::get_job_status(job_id);
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::JobStatus(status) => Ok(status),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// List jobs from a peer
    pub async fn list_jobs(&mut self, peer_id: &PeerId) -> CliResult<Vec<JobInfo>> {
        let query = Query::list_jobs(None);
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::JobList(jobs) => Ok(jobs),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Ping a peer
    pub async fn ping(&mut self, peer_id: &PeerId) -> CliResult<u64> {
        let start = std::time::Instant::now();
        let query = Query::ping();
        let response = self.query_peer(peer_id, query).await?;
        let rtt = start.elapsed().as_millis() as u64;
        
        match response {
            QueryResponse::Pong { .. } => Ok(rtt),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Get job result from a peer
    pub async fn get_job_result(&mut self, peer_id: &PeerId, job_id: &str) -> CliResult<JobResultInfo> {
        let query = Query::get_job_result(job_id);
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::JobResult(result) => Ok(result),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Submit a job to a peer
    pub async fn submit_job(&mut self, peer_id: &PeerId, model: &str, input: &str) -> CliResult<JobSubmitResponse> {
        let query = Query::submit_job(model, input);
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::JobSubmitted(result) => Ok(result),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
    
    /// Cancel a job on a peer
    pub async fn cancel_job(&mut self, peer_id: &PeerId, job_id: &str) -> CliResult<bool> {
        let query = Query::cancel_job(job_id);
        let response = self.query_peer(peer_id, query).await?;
        
        match response {
            QueryResponse::JobCancelled { success, .. } => Ok(success),
            QueryResponse::Error { message, code } => {
                Err(CliError::QueryError { message, code })
            }
            _ => Err(CliError::ParseError("Unexpected response type".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = NodeClient::new(30).await.unwrap();
        assert!(!client.local_peer_id().to_string().is_empty());
    }
    
    #[tokio::test]
    async fn test_unique_peer_id() {
        // Property 4: Each CLI execution gets unique PeerId
        let client1 = NodeClient::new(30).await.unwrap();
        let client2 = NodeClient::new(30).await.unwrap();
        
        assert_ne!(client1.local_peer_id(), client2.local_peer_id());
    }
    
    #[tokio::test]
    async fn test_discovered_nodes_empty_initially() {
        let client = NodeClient::new(30).await.unwrap();
        assert!(client.discovered_nodes().is_empty());
    }
}
