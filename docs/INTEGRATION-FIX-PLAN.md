# Plano de Correção de Integração

## Data: Janeiro 2025

Este documento descreve o plano detalhado para corrigir os problemas de integração identificados entre os componentes do sistema.

---

## Visão Geral

O sistema atual tem **8 problemas críticos de integração** que impedem o funcionamento end-to-end. Os problemas se dividem em 3 categorias principais:

### 1. **Comunicação CLI ↔ Node** (Problemas #1, #4, #6, #8)
- CLI não se comunica com nodes (tudo é simulado)
- Falta protocolo request-response no P2P
- Não há API para CLI consultar nodes

### 2. **Integração do Orchestrator** (Problemas #2, #7)
- Orchestrator não está integrado com P2P
- Falta mensagens P2P para orquestração distribuída

### 3. **Arquitetura e Design** (Problemas #3, #5)
- Comandos CLI no lugar errado (mvp-node em vez de mvp-cli)
- Natural Language Interface mapeia para comandos CLI inexistentes

---

## Estratégia de Correção

### Abordagem Recomendada: **CLI como Peer P2P**

Vamos implementar o CLI como um peer P2P completo que:
- ✅ Descobre nodes via mDNS automaticamente
- ✅ Usa request-response para queries diretas
- ✅ Pode publicar mensagens via gossipsub
- ✅ Totalmente descentralizado (sem HTTP)

### Alternativas Consideradas

**Opção A: HTTP API**
- ❌ Centralizado (cada node precisa de porta HTTP)
- ❌ Não aproveita infraestrutura P2P
- ✅ Mais simples de implementar

**Opção C: Híbrido (HTTP + P2P)**
- ❌ Complexidade de manter duas interfaces
- ⚠️ Confusão sobre quando usar cada uma


---

## Fase 1: Adicionar Request-Response ao P2P

### Objetivo
Implementar protocolo request-response no P2P para permitir queries diretas entre peers.

### Problema Atual
```rust
// mvp-node/src/network/mod.rs
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    // ❌ Não há request-response!
}
```

### Solução

**1.1. Definir protocolo de queries**

Criar `mvp-node/src/protocol/queries.rs`:

```rust
/// Query types for request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    // Node queries
    GetNodeInfo { node_id: String },
    GetNodeMetrics { node_id: String },
    GetNodeHealth { node_id: String },
    ListNodes,
    
    // Job queries
    GetJobStatus { job_id: String },
    ListJobs { filter: Option<JobFilter> },
    GetJobResult { job_id: String },
    
    // Orchestrator queries
    GetOrchestratorStatus,
    GetOrchestratorMetrics,
    RequestDecision { 
        context: String, 
        decision_type: DecisionType 
    },
    InterpretNaturalLanguage { input: String },
}

/// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryResponse {
    NodeInfo(NodeInfo),
    NodeMetrics(NodeMetrics),
    NodeHealth(HealthStatus),
    NodeList(Vec<NodeInfo>),
    JobStatus(JobStatus),
    JobList(Vec<JobInfo>),
    JobResult(JobResult),
    OrchestratorStatus(OrchestratorStatus),
    OrchestratorMetrics(OrchestratorMetrics),
    Decision(Decision),
    Interpretation(InterpretationResult),
    Error(String),
}
```


**1.2. Adicionar request-response ao P2PBehaviour**

Modificar `mvp-node/src/network/mod.rs`:

```rust
use libp2p::request_response::{self, ProtocolSupport};

// Criar codec para serialização
#[derive(Debug, Clone)]
pub struct QueryCodec;

impl request_response::Codec for QueryCodec {
    type Protocol = String;
    type Request = Query;
    type Response = QueryResponse;
    
    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) 
        -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Implementar leitura de Query
    }
    
    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) 
        -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Implementar leitura de QueryResponse
    }
    
    async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) 
        -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // Implementar escrita de Query
    }
    
    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, res: Self::Response) 
        -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // Implementar escrita de QueryResponse
    }
}

// Adicionar ao behaviour
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response: request_response::Behaviour<QueryCodec>,  // ✅ Novo!
}
```

**1.3. Implementar handlers de queries no Node**

Criar `mvp-node/src/query_handler.rs`:

```rust
/// Handles incoming queries from other peers
pub struct QueryHandler {
    orchestrator: Arc<LLMOrchestrator>,
    health_monitor: Arc<DefaultHealthMonitor>,
    job_queue: Arc<RwLock<JobQueue>>,
}

impl QueryHandler {
    pub async fn handle_query(&self, query: Query) -> QueryResponse {
        match query {
            Query::GetNodeInfo { .. } => {
                // Retornar informações do node
            }
            Query::GetOrchestratorStatus => {
                // Retornar status do orchestrator
            }
            Query::RequestDecision { context, decision_type } => {
                // Fazer decisão via orchestrator
                match self.orchestrator.make_decision(&context, decision_type).await {
                    Ok(decision) => QueryResponse::Decision(decision),
                    Err(e) => QueryResponse::Error(e.to_string()),
                }
            }
            // ... outros handlers
        }
    }
}
```

### Tarefas

- [ ] Criar `protocol/queries.rs` com tipos Query e QueryResponse
- [ ] Implementar QueryCodec para serialização
- [ ] Adicionar request_response ao P2PBehaviour
- [ ] Criar QueryHandler no mvp-node
- [ ] Integrar QueryHandler com event loop do P2P
- [ ] Adicionar testes de request-response

### Estimativa
**3-4 dias** (1 sprint)


---

## Fase 2: Implementar CLI como Peer P2P

### Objetivo
Transformar o CLI em um peer P2P completo que pode descobrir e consultar nodes.

### Problema Atual
```rust
// mvp-cli/src/client/mod.rs
pub async fn connect(&mut self, addr: &str) -> CliResult<PeerId> {
    // For now, simulate connection
    let fake_peer_id = PeerId::random();  // ❌ Fake!
    
    self.active_connections.insert(fake_peer_id, ConnectionInfo {
        peer_id: fake_peer_id,
        addresses: vec![multiaddr],
        connected_at: std::time::Instant::now(),
    });
    
    Ok(fake_peer_id)
}
```

### Solução

**2.1. Criar P2P Swarm no CLI**

Modificar `mvp-cli/src/client/mod.rs`:

```rust
use libp2p::{
    gossipsub, identify, mdns, noise, tcp, yamux,
    request_response,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};

/// CLI P2P Behaviour (similar ao node, mas sem gossipsub publish)
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
    pending_queries: HashMap<RequestId, oneshot::Sender<QueryResponse>>,
}

impl NodeClient {
    /// Create a new P2P-enabled CLI client
    pub async fn new() -> CliResult<Self> {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        // Create behaviours
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
        let identify = identify::Behaviour::new(identify::Config::new(
            "/mvp-cli/1.0.0".to_string(),
            keypair.public(),
        ));
        let request_response = request_response::Behaviour::new(
            QueryCodec,
            [(StreamProtocol::new("/mvp/query/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );
        
        let behaviour = CliP2PBehaviour {
            mdns,
            identify,
            request_response,
        };
        
        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|_| behaviour)?
            .build();
        
        Ok(Self {
            swarm,
            discovered_nodes: HashMap::new(),
            pending_queries: HashMap::new(),
        })
    }
    
    /// Start listening (ephemeral port)
    pub async fn start(&mut self) -> CliResult<()> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok(())
    }
    
    /// Send a query to a specific peer
    pub async fn query_peer(&mut self, peer_id: PeerId, query: Query) 
        -> CliResult<QueryResponse> 
    {
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
}
```

