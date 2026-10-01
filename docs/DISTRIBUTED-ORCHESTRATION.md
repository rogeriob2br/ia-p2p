# Distributed Orchestration

Sistema de orquestração distribuída inspirado em Swarm Intelligence para a rede P2P de IA.

## Visão Geral

O sistema implementa uma abordagem bio-inspirada para coordenação de nós, substituindo algoritmos de consenso tradicionais por:

- **Coordenação Emergente**: Coordenadores surgem baseados em pontuação de afinidade
- **Consenso Adaptativo**: Modelo de Response Threshold (inspirado em insetos sociais)
- **Decision Routing Inteligente**: Seleção dinâmica entre estratégias heurísticas, LLM ou híbridas
- **Context Groups**: Grupos dinâmicos de especialização
- **Partition Handling**: Detecção e reconciliação de partições de rede

## Arquitetura

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DistributedOrchestrator                        │
├─────────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌────────────────┐  ┌─────────────────────────┐ │
│  │ Affinity      │  │  Adaptive      │  │   Decision Router       │ │
│  │ Calculator    │  │  Consensus     │  │                         │ │
│  └───────────────┘  └────────────────┘  └─────────────────────────┘ │
│  ┌───────────────┐  ┌────────────────┐  ┌─────────────────────────┐ │
│  │ Context Group │  │   Partition    │  │   P2P Protocols         │ │
│  │ Manager       │  │   Handler      │  │                         │ │
│  └───────────────┘  └────────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Componentes

### 1. Affinity Calculator

Calcula pontuação de afinidade para cada nó baseado em:

- **Capacidade disponível** (weight: 0.3)
- **Latência de rede** (weight: 0.25)
- **Tempo de uptime** (weight: 0.2)
- **Match de especialização** (weight: 0.25)

```rust
// Fórmula de afinidade
affinity = w_c * capacity_score + w_l * latency_score + w_u * uptime_score + w_s * specialization_score
```

### 2. Adaptive Consensus (Response Threshold Model)

Implementa o modelo matemático de limiar adaptativo:

```rust
// Fórmula de tendência
T(θ) = s² / (s² + θ²) + σ × d

// Onde:
// s = stimulus (urgência do job)
// θ = threshold individual do nó
// σ = fator de demanda global
// d = demanda atual
```

**Características:**
- Thresholds adaptativos por nó (0.3 - 0.9)
- Aumentam após falhas, diminuem após sucessos
- Diferentes critérios: Supermajority, Unanimity, Adaptive

### 3. Decision Router

Seleciona automaticamente a estratégia de decisão:

| Condição | Estratégia |
|----------|------------|
| Job crítico | LLM Reasoning |
| Job de alto valor | LLM Reasoning |
| Alta complexidade | Hybrid Validation |
| Baixo valor, simples | Heuristic (GreedyAffinity) |

### 4. Context Groups

Grupos dinâmicos de nós especializados em contextos específicos:

```rust
// Formação automática após N jobs do mesmo tipo
manager.record_job("translation").await;
manager.record_job("translation").await;
manager.record_job("translation").await;
// Grupo "translation" pode ser formado
```

**Features:**
- Formação automática baseada em demanda
- Líderes emergentes por grupo
- Dissolução por inatividade
- Rotação de líderes

### 5. Partition Handler

Gerencia partições de rede:

- **Detecção**: Via heartbeat timeout
- **Estratégias de reconciliação**:
  - `LastWriteWins`: Última escrita vence
  - `VectorClocks`: Merge baseado em versões
  - `LLMMediated`: LLM resolve conflitos

## Modos de Operação

### Emergent Coordinator (Padrão)

```toml
[distributed_orchestration]
mode = "emergent"
rotation_trigger = "adaptive"
```

Coordenadores emergem baseados em afinidade e rotacionam periodicamente ou por eventos.

### Permanent Coordinator

```toml
[distributed_orchestration]
mode = "permanent"
```

Coordenador fixo (útil para testes ou cenários simples).

### Fully Decentralized

```toml
[distributed_orchestration]
mode = "fully_decentralized"
```

Cada decisão requer consenso distribuído (mais lento, mais resiliente).

## Fluxo de Decisão

```
┌─────────────┐
│ Job Arrives │
└──────┬──────┘
       ▼
┌──────────────────┐
│ Decision Router  │
│ analyzes context │
└──────┬───────────┘
       ▼
┌──────────────────────────────────────────────────────────────────┐
│                        Strategy Selection                         │
├──────────────────┬───────────────────┬───────────────────────────┤
│    Heuristic     │    LLM Local      │       Consensus           │
│   (< 10ms)       │   (100-500ms)     │     (1-5 seconds)         │
│                  │                   │                           │
│ GreedyAffinity   │ Local LLM makes   │ All nodes propose +       │
│ selects best     │ decision with     │ Adaptive Threshold        │
│ available node   │ full context      │ determines result         │
└────────┬─────────┴─────────┬─────────┴─────────────┬─────────────┘
         ▼                   ▼                       ▼
    ┌────────────────────────────────────────────────────────┐
    │                    Execute Decision                     │
    └────────────────────────────────────────────────────────┘
```

## Métricas

O sistema coleta automaticamente:

```rust
pub struct DistributedMetrics {
    pub total_decisions: u64,
    pub heuristic_decisions: u64,
    pub llm_decisions: u64,
    pub hybrid_decisions: u64,
    pub consensus_decisions: u64,
    pub coordinator_rotations: u64,
    pub context_groups_formed: u64,
    pub partitions_detected: u64,
    pub successful_reconciliations: u64,
    pub avg_consensus_latency_ms: u64,
}
```

## Configuração

```toml
[distributed_orchestration]
enabled = true
mode = "emergent"                    # emergent | permanent | fully_decentralized
rotation_trigger = "adaptive"        # adaptive | periodic | event_driven
high_value_threshold = 10.0
complexity_threshold = 0.7
min_nodes_per_group = 2
max_groups_per_node = 3
heartbeat_timeout_secs = 15
consensus_timeout_ms = 5000
max_consensus_rounds = 3
base_consensus_threshold = 0.6
```

## Simulation Mode

Para testes sem efeitos colaterais, use o modo de simulação:

```rust
use mvp_node::distributed_orchestration::DistributedOrchestratorConfig;

let config = DistributedOrchestratorConfig::simulation();
assert!(config.is_simulation());
```

No modo de simulação:
- Mensagens P2P não são enviadas
- Decisões são logadas mas não executadas
- Serviços externos são mockados
- Útil para testes unitários e integração

## Integração P2P

### Topics GossipSub

O sistema usa os seguintes topics para comunicação:

| Topic | Propósito |
|-------|-----------|
| `p2p-ai/orchestration/consensus/1.0` | Proposals e votos de consenso |
| `p2p-ai/orchestration/groups/1.0` | Formação e gestão de grupos |
| `p2p-ai/orchestration/coordination/1.0` | Eleição e rotação de coordenador |
| `p2p-ai/orchestration/heartbeat/1.0` | Heartbeats para detecção de partição |

### P2PAdapter

O `P2PAdapter` faz a ponte entre o sistema de orquestração e o network layer:

```rust
use mvp_node::distributed_orchestration::{P2PAdapter, OrchestrationNetworkMessage};

// Criar adapter
let (adapter, outbound_rx) = P2PAdapter::new("node-1".to_string());

// Registrar peers
adapter.register_peer("node-2".to_string()).await;

// Broadcast mensagem de consenso
adapter.broadcast_consensus(consensus_msg).await?;
```

### Network Integration

O `OrchestrationNetworkBridge` conecta o P2PAdapter ao P2PNetworkLayer:

```rust
use mvp_node::distributed_orchestration::create_orchestration_network_integration;
use mvp_node::network::NetworkEvent;

// Criar canal de eventos do network layer
let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);

// Criar a integração
let (adapter, event_task, publish_tx) = create_orchestration_network_integration(
    "node-1".to_string(),
    event_rx,
);

// Spawnar a task de processamento
tokio::spawn(event_task);

// Agora eventos de OrchestrationMessageReceived serão
// automaticamente roteados para o adapter
```

O bridge processa automaticamente:
- `PeerDiscovered` → registra peer no adapter
- `PeerExpired` → remove peer do adapter
- `OrchestrationMessageReceived` → parse e encaminha para o adapter

## Testes

### Unit Tests
- **64 testes** para componentes individuais
- Cobertura de todos os módulos incluindo network_integration

### Integration Tests
- **17 testes** end-to-end simulando múltiplos nós
- Cenários: fallback, concorrência, lifecycle completo

### Property Tests
- **28 testes** baseados em propriedades universais:
  - Affinities sempre normalizadas (0.0-1.0)
  - Thresholds dentro de bounds (0.3-0.9)
  - Consensus unânime sempre bem-sucedido
  - Heuristic sempre rápido (< 100ms)
  - Métricas nunca decrescem
  - Estado sempre consistente

## Exemplos Práticos

### Decisão Simples (Heurística)

```rust
let orchestrator = DistributedOrchestrator::new(config, "node-1".to_string(), None);

let context = DecisionContext {
    decision_type: "job_routing".to_string(),
    is_critical: false,
    job_value: 1.0,
    complexity_score: 0.2,
    ..Default::default()
};

let result = orchestrator
    .make_distributed_decision(&context, DecisionType::JobRouting)
    .await?;

// Heurística é usada automaticamente (baixo valor, baixa complexidade)
assert!(matches!(result.strategy_used, DecisionStrategy::Heuristic { .. }));
```

### Decisão Crítica (Consenso)

```rust
let context = DecisionContext {
    decision_type: "coordinator_election".to_string(),
    is_critical: true,
    job_value: 0.0,
    complexity_score: 0.5,
    ..Default::default()
};

let result = orchestrator
    .make_distributed_decision(&context, DecisionType::CoordinatorElection)
    .await?;

// LLM/Consenso é usado automaticamente (decisão crítica)
```

## Roadmap

- [x] Integração com gossipsub para broadcast de proposals
- [x] Simulation mode para testes
- [x] Multi-round consensus
- [x] Auto-formation de grupos
- [x] Network Integration Bridge (P2PAdapter ↔ P2PNetworkLayer)
- [ ] Persistência de estado entre reinícios
- [ ] Dashboard de métricas em tempo real
- [ ] Benchmarks de latência por modo de operação

