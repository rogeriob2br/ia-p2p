//! Network layer: libp2p swarm with gossipsub and mDNS

use crate::messages::{topics, P2PMessage};
use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode},
    identify,
    identity::Keypair,
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Events emitted by the network layer
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerDiscovered(PeerId),
    PeerExpired(PeerId),
    MessageReceived {
        topic: String,
        message: P2PMessage,
        source: Option<PeerId>,
    },
    Published {
        topic: String,
    },
}

/// Network node configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: Multiaddr,
    pub bootstrap_peers: Vec<Multiaddr>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
            bootstrap_peers: Vec::new(),
        }
    }
}

/// Combined network behaviour
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

/// P2P Network node
pub struct NetworkNode {
    swarm: Swarm<P2PBehaviour>,
    event_tx: mpsc::Sender<NetworkEvent>,
}

impl NetworkNode {
    /// Create a new network node with the given keypair
    pub async fn new(
        keypair: Keypair,
        config: NetworkConfig,
        event_tx: mpsc::Sender<NetworkEvent>,
    ) -> Result<Self> {
        let peer_id = PeerId::from(keypair.public());
        info!("Local peer ID: {}", peer_id);

        // Gossipsub configuration
        let message_id_fn = |message: &gossipsub::Message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            if let Some(source) = &message.source {
                source.hash(&mut hasher);
            }
            gossipsub::MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Gossipsub error: {}", e))?;

        // mDNS for local discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/p2p-ai/1.0.0".to_string(),
            keypair.public(),
        ));

        let behaviour = P2PBehaviour {
            gossipsub,
            mdns,
            identify,
        };

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on configured address
        swarm.listen_on(config.listen_addr)?;

        // Connect to bootstrap peers
        for addr in config.bootstrap_peers {
            if let Err(e) = swarm.dial(addr.clone()) {
                warn!("Failed to dial bootstrap peer {}: {}", addr, e);
            }
        }

        Ok(Self { swarm, event_tx })
    }

    /// Subscribe to all standard topics
    pub fn subscribe_all(&mut self) -> Result<()> {
        let topics = [topics::ANNOUNCE, topics::JOB_OFFER, topics::RECEIPT];
        for topic_str in topics {
            let topic = IdentTopic::new(topic_str);
            self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
            info!("Subscribed to topic: {}", topic_str);
        }
        Ok(())
    }

    /// Subscribe to a specific topic
    pub fn subscribe(&mut self, topic_str: &str) -> Result<()> {
        let topic = IdentTopic::new(topic_str);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        info!("Subscribed to topic: {}", topic_str);
        Ok(())
    }

    /// Publish a message to a topic
    pub fn publish(&mut self, topic_str: &str, message: &P2PMessage) -> Result<()> {
        let topic = IdentTopic::new(topic_str);
        let data = message.to_bytes()?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, data)?;
        debug!("Published message to topic: {}", topic_str);
        Ok(())
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Get listening addresses
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    /// Run the network event loop with a channel for outgoing messages
    pub async fn run_with_publisher(&mut self, mut publish_rx: mpsc::Receiver<(String, P2PMessage)>) {
        loop {
            tokio::select! {
                Some((topic, msg)) = publish_rx.recv() => {
                    if let Err(e) = self.publish(&topic, &msg) {
                        warn!("Failed to publish message: {}", e);
                    } else {
                        info!("Published message to {}", topic);
                    }
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
            }
        }
    }

    /// Run the network event loop (no outgoing messages)
    pub async fn run(&mut self) {
        loop {
            let event = self.swarm.select_next_some().await;
            self.handle_swarm_event(event).await;
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<P2PBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            })) => {
                let topic = message.topic.to_string();
                match P2PMessage::from_bytes(&message.data) {
                    Ok(msg) => {
                        let event = NetworkEvent::MessageReceived {
                            topic,
                            message: msg,
                            source: Some(propagation_source),
                        };
                        let _ = self.event_tx.send(event).await;
                    }
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                    }
                }
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    info!("mDNS discovered peer: {} at {}", peer_id, addr);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::PeerDiscovered(peer_id))
                        .await;
                }
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, _) in peers {
                    info!("mDNS peer expired: {}", peer_id);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::PeerExpired(peer_id))
                        .await;
                }
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
            })) => {
                debug!(
                    "Identified peer {}: {} with {} addresses",
                    peer_id,
                    info.protocol_version,
                    info.listen_addrs.len()
                );
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
            }
            _ => {}
        }
    }
}

/// Load or generate Ed25519 keypair from file
pub fn load_or_generate_keypair(path: &std::path::Path) -> Result<Keypair> {
    if path.exists() {
        let data = std::fs::read(path)?;
        let keys: serde_json::Value = serde_json::from_slice(&data)?;
        if let Some(secret) = keys.get("secret_key").and_then(|v| v.as_str()) {
            // Parse hex-encoded secret key
            let bytes = hex::decode(secret).or_else(|_| {
                // Try base64 or raw
                Ok::<_, anyhow::Error>(secret.as_bytes().to_vec())
            })?;
            if bytes.len() >= 32 {
                let keypair = Keypair::ed25519_from_bytes(bytes[..32].to_vec())?;
                return Ok(keypair);
            }
        }
    }

    // Generate new keypair
    let keypair = Keypair::generate_ed25519();
    Ok(keypair)
}

/// Convert libp2p Keypair to hex-encoded strings for storage
pub fn keypair_to_hex(keypair: &Keypair) -> Result<(String, String)> {
    match keypair.clone().try_into_ed25519() {
        Ok(ed_keypair) => {
            let secret = hex::encode(ed_keypair.secret().as_ref());
            let public = hex::encode(ed_keypair.public().to_bytes());
            Ok((public, secret))
        }
        Err(_) => anyhow::bail!("Only Ed25519 keys are supported"),
    }
}

