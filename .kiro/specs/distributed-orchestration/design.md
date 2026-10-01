# Distributed Orchestration Design Document

## Overview

O Distributed Orchestration é um sistema de orquestração descentralizada inspirado em Swarm Intelligence que transforma o modelo centralizado do LLM Orchestrator em uma arquitetura distribuída verdadeiramente P2P. O sistema implementa três conceitos-chave:

1. **Coordenação Emergente**: Coordenadores surgem naturalmente baseado em affinity scores, sem eleição formal
2. **Consenso Adaptativo**: Decisões coletivas usando Response Threshold Model de insetos sociais
3. **Decision Routing Inteligente**: Escolha automática entre heurística, LLM ou híbrido baseado em custo/qualidade

O sistema suporta **agrupamento dinâmico por contexto**, permitindo que nodes se organizem em grupos especializados com líderes emergentes para máxima eficiência. Múltiplos contextos podem coexistir e colaborar quando necessário.

### Inspiração Biológica

O design é inspirado em comportamentos de insetos sociais:
- **Formigas**: Stigmergy e emergência de caminhos ótimos
- **Abelhas**: Response threshold model para divisão de trabalho
- **Enxames**: Auto-organização sem controle central

### Princípios de Design

1. **Sem Ponto Único de Falha**: Coordenação distribuída e rotativa
2. **Emergência Natural**: Líderes surgem por mérito, não por eleição
3. **Adaptação Contínua**: Thresholds e estratégias aprendem com experiência
4. **Eficiência Econômica**: Heurística para decisões simples, LLM para críticas
5. **Resistência a Censura**: Operação independente em partições de rede

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Application Layer                                │
│                   (CLI, API, User Interface)                            │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                   Distributed Orchestrator Core                         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐     │
│  │ Orchestration    │  │ Context Group    │  │ Partition        │     │
│  │ Manager          │  │ Manager          │  │ Handler          │     │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                   Coordination Layer (Emergent)                         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐     │
│  │ Affinity         │  │ Rotation         │  │ Group Leader     │     │
│  │ Calculator       │  │ Manager          │  │ Selector         │     │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                   Consensus Layer (Adaptive)                            │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐     │
│  │ Adaptive         │  │ Threshold        │  │ Proposal         │     │
│  │ Consensus        │  │ Adapter          │  │ Aggregator       │     │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                   Decision Routing Layer                                │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐     │
│  │ Decision         │  │ Strategy         │  │ Complexity       │     │
│  │ Router           │  │ Selector         │  │ Analyzer         │     │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                   Execution Layer                                       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐     │
│  │ Local LLM        │  │ Heuristic        │  │ Hybrid           │     │
│  │ Backend          │  │ Algorithms       │  │ Validator        │     │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                   Network Layer (P2P)                                   │
│              libp2p + GossipSub + mDNS + Custom Protocols               │
└─────────────────────────────────────────────────────────────────────────┘
```

### Fluxo de Decisão Distribuída

```
1. Job Arrival
   │
   ├─> Decision Router: Avaliar complexidade e valor
   │   │
   │   ├─> Simples/Barato? → Heurística Local (5-10ms)
   │   │
   │   ├─> Crítico/Caro? → LLM + Consenso (1-3s)
   │   │
   │   └─> Complexo? → Híbrido (Heurística + LLM valida)
   │
2. Se Consenso Necessário
   │
   ├─> Identificar Context Group relevante
   │
   ├─> Group Leader coordena proposta
   │
   ├─> Nodes calculam Tendency (stimulus, threshold, demand)
   │
   ├─> Agregação de propostas
   │
   └─> Consenso alcançado? → Executar decisão
       │
       └─> Não? → Fallback heurístico
```

## Components and Interfaces

### 1. Distributed Orchestrator Core

**Purpose**: Gerencia orquestração distribuída e coordena todos os componentes.

**Key Responsibilities**:
- Inicializar e gerenciar modo de orquestração (emergent/permanent/fully_decentralized)
- Coordenar formação e dissolução de Context Groups
- Gerenciar transições entre modos de operação
- Detectar e responder a partições de rede
- Expor API de estado e métricas

**Interface**:
```rust
pub struct DistributedOrchestrator {
    mode: OrchestrationMode,
    affinity_calculator: Arc<AffinityCalculator>,
    rotation_manager: Arc<RotationManager>,
    consensus_manager: Arc<AdaptiveConsensus>,
    decision_router: Arc<DecisionRouter>,
    context_group_manager: Arc<ContextGroupManager>,
    partition_handler: Arc<PartitionHandler>,
    local_llm_orchestrator: Arc<LLMOrchestrator>, // Integração com existente
    config: DistributedOrchestratorConfig,
}

pub trait DistributedOrchestratorTrait {
    async fn make_distributed_decision(
        &self,
        context: &DecisionContext,
        decision_type: DecisionType,
    ) -> Result<DistributedDecision>;
    
    async fn get_current_coordinator(&self) -> Result<Option<NodeId>>;
    async fn get_context_groups(&self) -> Result<Vec<ContextGroup>>;
    async fn get_orchestration_state(&self) -> Result<OrchestrationState>;
    fn get_metrics(&self) -> DistributedMetrics;
}

#[derive(Debug, Clone)]
pub enum OrchestrationMode {
    EmergentCoordinator {
        rotation_trigger: RotationTrigger,
        affinity_recalc_interval: Duration,
    },
    PermanentCoordinator {
        coordinator_id: NodeId,
    },
    FullyDecentralized {
        consensus_for_every_decision: bool,
    },
}

#[derive(Debug, Clone)]
pub enum RotationTrigger {
    Periodic { interval_jobs: u64 },
    EventDriven,
    Adaptive,
}
```

### 2. Affinity Calculator

**Purpose**: Calcula coordination affinity score para determinar qual node deve coordenar.

**Key Responsibilities**:
- Calcular affinity score baseado em múltiplos fatores
- Normalizar scores entre 0.0 e 1.0
- Recalcular periodicamente ou sob demanda
- Considerar especialização, capacidade, latência e uptime

**Interface**:
```rust
pub struct AffinityCalculator {
    weights: AffinityWeights,
    history: Arc<RwLock<HashMap<NodeId, AffinityHistory>>>,
}

pub trait AffinityCalculatorTrait {
    async fn calculate_affinity(&self, node_id: &NodeId) -> Result<f64>;
    async fn calculate_all_affinities(&self, nodes: &[NodeId]) -> Result<HashMap<NodeId, f64>>;
    async fn get_highest_affinity_node(&self, nodes: &[NodeId]) -> Result<NodeId>;
}

#[derive(Debug, Clone)]
pub struct AffinityWeights {
    pub capacity_weight: f64,        // 0.3
    pub latency_weight: f64,         // 0.25
    pub uptime_weight: f64,          // 0.25
    pub specialization_weight: f64,  // 0.2
}

#[derive(Debug, Clone)]
pub struct AffinityHistory {
    pub node_id: NodeId,
    pub recent_scores: VecDeque<(Instant, f64)>,
    pub avg_score: f64,
    pub times_coordinated: u64,
}
```

### 3. Rotation Manager

**Purpose**: Gerencia rotação de coordenadores baseado em triggers configurados.

**Key Responsibilities**:
- Monitorar triggers de rotação (periodic, event-driven, adaptive)
- Iniciar processo de rotação quando apropriado
- Garantir transição suave entre coordenadores
- Registrar histórico de rotações

**Interface**:
```rust
pub struct RotationManager {
    trigger: RotationTrigger,
    jobs_since_rotation: Arc<AtomicU64>,
    last_rotation: Arc<RwLock<Instant>>,
    rotation_history: Arc<RwLock<Vec<RotationEvent>>>,
}

pub trait RotationManagerTrait {
    async fn should_rotate(&self, current_coordinator: &NodeId) -> Result<bool>;
    async fn initiate_rotation(&self) -> Result<NodeId>;
    fn record_rotation(&self, from: NodeId, to: NodeId, reason: RotationReason);
    fn get_rotation_history(&self) -> Vec<RotationEvent>;
}

#[derive(Debug, Clone)]
pub enum RotationReason {
    PeriodicInterval,
    CoordinatorFailure,
    LoadImbalance,
    AffinityChange,
    ManualTrigger,
}

#[derive(Debug, Clone)]
pub struct RotationEvent {
    pub timestamp: DateTime<Utc>,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub reason: RotationReason,
    pub affinity_before: f64,
    pub affinity_after: f64,
}
```

### 4. Adaptive Consensus

**Purpose**: Implementa consenso adaptativo usando Response Threshold Model.

**Key Responsibilities**:
- Calcular tendency de cada node aceitar proposta
- Agregar propostas e determinar consenso
- Adaptar thresholds baseado em experiência
- Suportar múltiplos critérios de consenso

**Interface**:
```rust
pub struct AdaptiveConsensus {
    criteria: ConsensusCriteria,
    base_threshold: f64,
    stimulus_weight: f64,
    demand_weight: f64,
    node_thresholds: Arc<RwLock<HashMap<NodeId, f64>>>,
    threshold_adaptation_rate: f64,
    min_participants: usize,
}

pub trait AdaptiveConsensusTrait {
    async fn propose_decision(
        &self,
        proposal: Proposal,
        target_nodes: &[NodeId],
    ) -> Result<ConsensusResult>;
    
    async fn evaluate_proposal(&self, proposal: &Proposal) -> Result<f64>; // Returns tendency
    
    fn update_threshold(&mut self, node_id: &NodeId, success: bool);
    
    fn get_node_threshold(&self, node_id: &NodeId) -> f64;
}

#[derive(Debug, Clone)]
pub enum ConsensusCriteria {
    SimpleMajority {
        required_percentage: f64,
        min_participants: usize,
    },
    ConfidenceThreshold {
        min_confidence: f64,
        min_participants: usize,
        aggregate_method: AggregateMethod,
    },
    AdaptiveThreshold {
        base_threshold: f64,
        stimulus_weight: f64,
        demand_weight: f64,
        min_participants: usize,
    },
}

#[derive(Debug, Clone)]
pub enum AggregateMethod {
    Average,
    WeightedAverage,
    Median,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: String,
    pub node_id: NodeId,
    pub decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub job_priority: u8,
    pub wait_time_secs: u64,
    pub specialization_match: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub reached: bool,
    pub decision: Option<String>,
    pub confidence: f64,
    pub participants: Vec<NodeId>,
    pub rounds: usize,
    pub reasoning: String,
}

/// Calcula tendency usando Response Threshold Model
pub fn calculate_tendency(stimulus: f64, threshold: f64, demand: f64, sigma: f64) -> f64 {
    let response = (stimulus.powi(2)) / (stimulus.powi(2) + threshold.powi(2));
    let demand_factor = demand;
    sigma * response + (1.0 - sigma) * demand_factor
}
```

### 5. Threshold Adapter

**Purpose**: Adapta thresholds de nodes baseado em experiência e outcomes.

**Key Responsibilities**:
- Atualizar thresholds após decisões
- Limitar thresholds entre valores seguros (0.3-0.9)
- Registrar histórico de adaptações
- Calcular taxas de sucesso por node

**Interface**:
```rust
pub struct ThresholdAdapter {
    adaptation_rate: f64,
    min_threshold: f64,
    max_threshold: f64,
    adaptation_history: Arc<RwLock<HashMap<NodeId, Vec<ThresholdAdaptation>>>>,
}

pub trait ThresholdAdapterTrait {
    fn adapt_threshold(
        &mut self,
        node_id: &NodeId,
        current_threshold: f64,
        success: bool,
    ) -> f64;
    
    fn get_adaptation_history(&self, node_id: &NodeId) -> Vec<ThresholdAdaptation>;
    
    fn calculate_success_rate(&self, node_id: &NodeId) -> f64;
}

#[derive(Debug, Clone)]
pub struct ThresholdAdaptation {
    pub timestamp: DateTime<Utc>,
    pub old_threshold: f64,
    pub new_threshold: f64,
    pub success: bool,
    pub decision_type: String,
}
```

### 6. Decision Router

**Purpose**: Decide qual estratégia usar (heurística, LLM ou híbrido) baseado em contexto.

**Key Responsibilities**:
- Avaliar criticidade e valor de decisões
- Calcular complexidade de decisões
- Selecionar estratégia ótima
- Registrar uso de estratégias para aprendizado

**Interface**:
```rust
pub struct DecisionRouter {
    config: DecisionRoutingConfig,
    strategy_usage: Arc<RwLock<HashMap<String, StrategyStats>>>,
}

pub trait DecisionRouterTrait {
    async fn select_strategy(&self, context: &DecisionContext) -> DecisionStrategy;
    
    async fn execute_strategy(
        &self,
        strategy: &DecisionStrategy,
        context: &DecisionContext,
    ) -> Result<StrategyResult>;
    
    fn record_strategy_outcome(&mut self, strategy: &str, success: bool, latency_ms: u64);
    
    fn get_strategy_stats(&self) -> HashMap<String, StrategyStats>;
}

#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub decision_type: String,
    pub is_critical: bool,
    pub job_value: f64,
    pub complexity_score: f64,
    pub requires_consensus: bool,
    pub context_group: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DecisionStrategy {
    Heuristic {
        algorithm: HeuristicAlgorithm,
        expected_latency_ms: u64,
        cost_per_decision: f64,
    },
    LLMReasoning {
        model: String,
        temperature: f32,
        expected_latency_ms: u64,
        cost_per_decision: f64,
    },
    HybridValidation {
        heuristic: Box<DecisionStrategy>,
        llm_validator: Box<DecisionStrategy>,
    },
}

#[derive(Debug, Clone)]
pub enum HeuristicAlgorithm {
    GreedyAffinity,
    RoundRobin,
    LeastLoaded,
    Random,
}

#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub latency_ms: u64,
    pub cost: f64,
    pub strategy_used: String,
}

#[derive(Debug, Clone)]
pub struct StrategyStats {
    pub total_uses: u64,
    pub successes: u64,
    pub failures: u64,
    pub avg_latency_ms: f64,
    pub total_cost: f64,
}
```

### 7. Context Group Manager

**Purpose**: Gerencia formação, dissolução e coordenação de grupos de contexto.

**Key Responsibilities**:
- Detectar padrões de jobs por domínio
- Formar grupos especializados dinamicamente
- Eleger Group Leaders baseado em affinity
- Coordenar comunicação entre grupos
- Dissolver grupos quando não mais necessários

**Interface**:
```rust
pub struct ContextGroupManager {
    groups: Arc<RwLock<HashMap<String, ContextGroup>>>,
    formation_metrics: Arc<RwLock<GroupFormationMetrics>>,
    config: ContextGroupConfig,
}

pub trait ContextGroupManagerTrait {
    async fn evaluate_group_formation(&self, job_patterns: &JobPatterns) -> Result<Vec<GroupAction>>;
    
    async fn form_group(&mut self, context: &str, nodes: Vec<NodeId>) -> Result<ContextGroup>;
    
    async fn dissolve_group(&mut self, group_id: &str) -> Result<()>;
    
    async fn elect_group_leader(&self, group: &ContextGroup) -> Result<NodeId>;
    
    async fn coordinate_cross_context(
        &self,
        job: &Job,
        required_contexts: &[String],
    ) -> Result<CrossContextDecision>;
    
    fn get_group(&self, context: &str) -> Option<ContextGroup>;
    
    fn get_all_groups(&self) -> Vec<ContextGroup>;
}

#[derive(Debug, Clone)]
pub struct ContextGroup {
    pub id: String,
    pub context: String,
    pub leader: Option<NodeId>,
    pub members: Vec<NodeId>,
    pub specializations: Vec<ModelSpecialization>,
    pub formed_at: DateTime<Utc>,
    pub jobs_processed: u64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone)]
pub enum GroupAction {
    FormGroup { context: String, nodes: Vec<NodeId> },
    DissolveGroup { group_id: String },
    AddMember { group_id: String, node_id: NodeId },
    RemoveMember { group_id: String, node_id: NodeId },
    RotateLeader { group_id: String, new_leader: NodeId },
}

#[derive(Debug, Clone)]
pub struct JobPatterns {
    pub context_frequencies: HashMap<String, u64>,
    pub context_specializations: HashMap<String, Vec<ModelSpecialization>>,
    pub avg_jobs_per_minute: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct CrossContextDecision {
    pub decision: String,
    pub participating_groups: Vec<String>,
    pub group_leaders: Vec<NodeId>,
    pub consensus_reached: bool,
    pub coordination_latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ContextGroupConfig {
    pub min_jobs_for_formation: u64,
    pub min_nodes_per_group: usize,
    pub max_groups_per_node: usize,
    pub dissolution_idle_threshold_secs: u64,
    pub leader_rotation_enabled: bool,
}
```

### 8. Partition Handler

**Purpose**: Detecta e responde a partições de rede, mantendo operação em cada partição.

**Key Responsibilities**:
- Detectar partições de rede via heartbeats
- Eleger coordenadores em cada partição
- Operar independentemente durante partição
- Reconciliar estado quando partição é resolvida

**Interface**:
```rust
pub struct PartitionHandler {
    partition_detector: Arc<PartitionDetector>,
    reconciliation_strategy: ReconciliationStrategy,
    partition_history: Arc<RwLock<Vec<PartitionEvent>>>,
}

pub trait PartitionHandlerTrait {
    async fn detect_partition(&self) -> Result<Option<PartitionInfo>>;
    
    async fn handle_partition(&mut self, partition: PartitionInfo) -> Result<()>;
    
    async fn reconcile_partitions(
        &mut self,
        partition_a: &[NodeId],
        partition_b: &[NodeId],
    ) -> Result<ReconciliationResult>;
    
    fn get_partition_history(&self) -> Vec<PartitionEvent>;
}

#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub detected_at: DateTime<Utc>,
    pub partitions: Vec<Vec<NodeId>>,
    pub partition_risk: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum ReconciliationStrategy {
    LastWriteWins,
    VectorClock,
    ConflictResolution { resolver: String },
}

#[derive(Debug, Clone)]
pub struct PartitionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: PartitionEventType,
    pub affected_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub enum PartitionEventType {
    PartitionDetected,
    PartitionResolved,
    ReconciliationStarted,
    ReconciliationCompleted,
}

#[derive(Debug, Clone)]
pub struct ReconciliationResult {
    pub conflicts_found: u64,
    pub conflicts_resolved: u64,
    pub decisions_merged: u64,
    pub state_synchronized: bool,
}
```

## Data Models

### Core Data Structures

```rust
// Node identification
pub type NodeId = String;

// Distributed decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedDecision {
    pub decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub strategy_used: DecisionStrategy,
    pub consensus_result: Option<ConsensusResult>,
    pub coordinator: Option<NodeId>,
    pub context_group: Option<String>,
    pub latency_ms: u64,
    pub cost: f64,
    pub timestamp: DateTime<Utc>,
}

// Orchestration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationState {
    pub mode: OrchestrationMode,
    pub current_coordinator: Option<NodeId>,
    pub coordinator_affinity: f64,
    pub active_context_groups: Vec<ContextGroup>,
    pub node_thresholds: HashMap<NodeId, f64>,
    pub partition_status: PartitionStatus,
    pub metrics: DistributedMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStatus {
    Healthy,
    Partitioned { partitions: Vec<Vec<NodeId>> },
    Reconciling,
}

// Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedMetrics {
    pub total_decisions: u64,
    pub consensus_decisions: u64,
    pub heuristic_decisions: u64,
    pub llm_decisions: u64,
    pub hybrid_decisions: u64,
    pub avg_consensus_latency_ms: f64,
    pub avg_consensus_rounds: f64,
    pub coordinator_rotations: u64,
    pub context_groups_formed: u64,
    pub context_groups_dissolved: u64,
    pub cross_context_jobs: u64,
    pub partition_events: u64,
}

// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedOrchestratorConfig {
    pub mode: OrchestrationMode,
    pub affinity_weights: AffinityWeights,
    pub consensus_criteria: ConsensusCriteria,
    pub decision_routing: DecisionRoutingConfig,
    pub context_groups: ContextGroupConfig,
    pub partition_handling: PartitionHandlingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRoutingConfig {
    pub high_value_threshold_usd: f64,
    pub complexity_threshold: f64,
    pub critical_decisions: Vec<String>,
    pub llm_large_cost: f64,
    pub llm_small_cost: f64,
    pub heuristic_cost: f64,
    pub llm_large_latency_ms: u64,
    pub llm_small_latency_ms: u64,
    pub heuristic_latency_ms: u64,
    pub default_strategies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionHandlingConfig {
    pub detection_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub reconciliation_strategy: ReconciliationStrategy,
    pub auto_reconcile: bool,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*



## LLM-to-LLM Communication Architecture

**IMPORTANTE**: O sistema opera em **dois modos** dependendo da estratégia selecionada:

### Modo 1: Determinístico (Heurística)
- **Quando**: Decisões simples, baixo valor, alta frequência
- **Como**: Algoritmos determinísticos locais (sem comunicação LLM)
- **Exemplos**: Round-robin, least-loaded, greedy affinity
- **Latência**: <10ms
- **Comunicação**: Apenas troca de métricas (não usa LLM)

### Modo 2: Não-Determinístico (LLM-to-LLM)
- **Quando**: Decisões críticas, alto valor, complexas
- **Como**: Cada node usa LLM local para raciocinar, depois consenso
- **Comunicação**: Propostas LLM trocadas via P2P
- **Latência**: 1-3s (LLM + rede)

### LLM-to-LLM Consensus Flow (Modo 2)

```
┌─────────────────────────────────────────────────────────────┐
│ Fase 1: Geração Local de Propostas (Paralelo)              │
└─────────────────────────────────────────────────────────────┘

Node A                    Node B                    Node C
   │                         │                         │
   ├─> LLM Local            ├─> LLM Local            ├─> LLM Local
   │   analisa contexto     │   analisa contexto     │   analisa contexto
   │                         │                         │
   ├─> Gera Proposta A      ├─> Gera Proposta B      ├─> Gera Proposta C
   │   {decision: "X",      │   {decision: "Y",      │   {decision: "X",
   │    confidence: 0.8,    │    confidence: 0.6,    │    confidence: 0.9,
   │    reasoning: "..."}   │    reasoning: "..."}   │    reasoning: "..."}
   │                         │                         │

┌─────────────────────────────────────────────────────────────┐
│ Fase 2: Troca de Propostas (P2P via libp2p)                │
└─────────────────────────────────────────────────────────────┘

   │─────── Broadcast ──────>│                         │
   │                         │─────── Broadcast ──────>│
   │<────── Broadcast ───────│                         │
   │                         │<────── Broadcast ───────│
   │<──────────────────────────────── Broadcast ───────│
   │─────────────────────────────────> Broadcast ──────>│

Cada node agora tem todas as 3 propostas

┌─────────────────────────────────────────────────────────────┐
│ Fase 3: Avaliação Local (Response Threshold Model)         │
└─────────────────────────────────────────────────────────────┘

Node A                    Node B                    Node C
   │                         │                         │
   ├─> Calcula Tendency     ├─> Calcula Tendency     ├─> Calcula Tendency
   │   para cada proposta   │   para cada proposta   │   para cada proposta
   │                         │                         │
   │   Proposta A: 0.85     │   Proposta A: 0.75     │   Proposta A: 0.90
   │   Proposta B: 0.45     │   Proposta B: 0.80     │   Proposta B: 0.50
   │   Proposta C: 0.88     │   Proposta C: 0.70     │   Proposta C: 0.92
   │                         │                         │
   ├─> Vota em C            ├─> Vota em B            ├─> Vota em C
   │                         │                         │

┌─────────────────────────────────────────────────────────────┐
│ Fase 4: Agregação e Consenso (Coordenador)                 │
└─────────────────────────────────────────────────────────────┘

Coordenador (Node A neste exemplo):
   │
   ├─> Recebe votos: A=0, B=1, C=2
   │
   ├─> Calcula tendência média para C:
   │   (0.88 + 0.70 + 0.92) / 3 = 0.83
   │
   ├─> 0.83 > threshold (0.6) ✓
   │
   ├─> CONSENSO ALCANÇADO: Decisão C
   │
   └─> Broadcast ConsensusReached
```

### Consensus Protocol Messages (Detalhado)

**Message Types**:
```rust
pub enum ConsensusMessage {
    // Fase 1: Node propõe decisão gerada por seu LLM local
    Proposal {
        id: String,
        node_id: NodeId,
        decision: String,           // Decisão sugerida pelo LLM
        confidence: f64,            // Confiança do LLM (0.0-1.0)
        reasoning: String,          // Raciocínio do LLM (texto)
        context: DecisionContext,   // Contexto da decisão
        llm_model: String,          // Qual modelo foi usado
        timestamp: DateTime<Utc>,
    },
    
    // Fase 3: Node vota baseado em tendency calculada
    Vote {
        proposal_id: String,
        node_id: NodeId,
        tendency: f64,              // Calculada via Response Threshold Model
        threshold: f64,             // Threshold atual do node
        preferred_proposal: String, // ID da proposta preferida
    },
    
    // Fase 4: Coordenador anuncia consenso
    ConsensusReached {
        proposal_id: String,
        decision: String,
        confidence: f64,
        participants: Vec<NodeId>,
        winning_reasoning: String,  // Raciocínio da proposta vencedora
    },
    
    // Fase 4: Consenso falhou
    ConsensusFailed {
        proposal_id: String,
        reason: String,
        fallback_decision: Option<String>, // Decisão heurística de fallback
    },
}
```

**Protocol Flow**:
1. Cada node gera proposta usando seu LLM local (paralelo)
2. Propostas são broadcast via GossipSub
3. Cada node calcula tendency para cada proposta (Response Threshold Model)
4. Nodes votam na proposta com maior tendency
5. Coordenador agrega votos e calcula consenso
6. Se consenso alcançado, broadcast ConsensusReached
7. Se não alcançado após max_rounds, broadcast ConsensusFailed
8. Fallback para decisão heurística local

### Exemplo Concreto: Roteamento de Job de Alto Valor

**Cenário**: Job de tradução de 50 páginas ($50) precisa ser roteado

**Passo 1: Decision Router decide usar LLM Strategy**
```rust
// Em cada node
let context = DecisionContext {
    decision_type: "job_routing".to_string(),
    is_critical: false,
    job_value: 50.0,  // Alto valor!
    complexity_score: 0.6,
    requires_consensus: true,
};

let strategy = decision_router.select_strategy(&context).await?;
// Retorna: LLMReasoning (porque job_value > high_value_threshold)
```

**Passo 2: Cada node gera proposta com seu LLM local**
```rust
// Node A (tem modelo de tradução especializado)
let prompt = format!(
    "Analyze this translation job and suggest the best node to execute it.
    
    Job: Translate 50 pages from English to Portuguese
    Value: $50
    
    Available nodes:
    - Node A: Llama-3.2-3B fine-tuned for translation, load: 30%, latency: 50ms
    - Node B: Mistral-7B general, load: 10%, latency: 80ms
    - Node C: Llama-3.2-3B general, load: 20%, latency: 40ms
    
    Respond with JSON:
    {{
        \"decision\": \"node_id\",
        \"confidence\": 0.0-1.0,
        \"reasoning\": \"explanation\"
    }}"
);

let llm_response = local_llm.generate(&prompt).await?;
// Node A propõe: "Node A" (confidence: 0.9, reasoning: "Specialized model...")
// Node B propõe: "Node B" (confidence: 0.7, reasoning: "Lower load...")
// Node C propõe: "Node A" (confidence: 0.8, reasoning: "Specialization important...")
```

**Passo 3: Propostas são trocadas via P2P**
```rust
// Cada node envia sua proposta via libp2p GossipSub
let proposal = Proposal {
    id: uuid::Uuid::new_v4().to_string(),
    node_id: self.node_id.clone(),
    decision: llm_response.decision,
    confidence: llm_response.confidence,
    reasoning: llm_response.reasoning,
    context: context.clone(),
    llm_model: "llama-3.2-3b-translation".to_string(),
    timestamp: Utc::now(),
};

gossipsub.publish(CONSENSUS_TOPIC, serde_json::to_vec(&proposal)?)?;
```

**Passo 4: Cada node calcula tendency para cada proposta**
```rust
// Node A avalia as 3 propostas
for proposal in proposals {
    let stimulus = calculate_stimulus(&proposal); // Baseado em confidence, specialization_match
    let threshold = self.get_threshold(&proposal.node_id);
    let demand = proposals.iter().filter(|p| p.decision == proposal.decision).count() as f64 / proposals.len() as f64;
    
    let tendency = calculate_tendency(stimulus, threshold, demand, 0.6);
    
    // Node A: Proposta "Node A" tem tendency 0.85
    // Node A: Proposta "Node B" tem tendency 0.45
}

// Node A vota na proposta com maior tendency
let vote = Vote {
    proposal_id: best_proposal.id,
    node_id: self.node_id.clone(),
    tendency: best_tendency,
    threshold: self.threshold,
    preferred_proposal: best_proposal.id,
};
```

**Passo 5: Coordenador agrega e decide**
```rust
// Coordenador recebe votos e calcula consenso
let votes_for_node_a = 2; // Node A e Node C votaram em "Node A"
let votes_for_node_b = 1; // Node B votou em "Node B"

let avg_tendency_node_a = (0.85 + 0.80) / 3.0 = 0.55; // Média incluindo nodes que não votaram
let avg_tendency_node_b = 0.45 / 3.0 = 0.15;

// Consenso alcançado para "Node A" (tendency 0.55 > threshold 0.6? Não!)
// Precisa de mais uma rodada ou fallback
```

### Comunicação P2P: Protocolos libp2p

**Tópicos GossipSub**:
```rust
const CONSENSUS_TOPIC: &str = "/distributed-orchestration/consensus/1.0.0";
const GROUP_TOPIC: &str = "/distributed-orchestration/groups/1.0.0";
const COORDINATION_TOPIC: &str = "/distributed-orchestration/coordination/1.0.0";
```

**Request-Response para comunicação direta**:
```rust
// Para comunicação direta entre Group Leaders
const CROSS_CONTEXT_PROTOCOL: &str = "/distributed-orchestration/cross-context/1.0.0";
```

### Quando LLM-to-LLM é Usado vs Determinístico

| Tipo de Decisão | Estratégia | Comunicação | Latência |
|-----------------|-----------|-------------|----------|
| Job routing (baixo valor) | Heurística | Métricas apenas | <10ms |
| Job routing (alto valor) | LLM + Consenso | Propostas LLM via P2P | 1-3s |
| Coordinator election | LLM + Consenso | Propostas LLM via P2P | 2-5s |
| Job acceptance | Heurística | Local apenas | <5ms |
| Job prioritization | Híbrido | Heurística + LLM valida | 300ms |
| Group formation | Heurística | Métricas apenas | <50ms |
| Cross-context coordination | LLM + Consenso | Propostas entre líderes | 2-5s |

### Resumo: Duas Camadas Complementares

**Camada Determinística (Infraestrutura)**:
- Protocolos de rede (libp2p)
- Cálculos matemáticos (affinity, tendency)
- Agregação de votos
- Detecção de falhas
- **Sempre ativa, sempre funcionando**

**Camada Não-Determinística (LLM-to-LLM)**:
- Raciocínio distribuído via LLMs locais
- Propostas geradas por IA
- Consenso sobre decisões complexas
- **Ativada apenas quando necessário (decisões críticas/caras)**

A camada determinística **orquestra** a camada não-determinística, decidindo quando usar LLM-to-LLM e como agregar as decisões.

### Arquitetura por Node

**Cada node na rede tem a estrutura completa**:

```
Node A                          Node B                          Node C
┌─────────────────────┐        ┌─────────────────────┐        ┌─────────────────────┐
│ Distributed Orch    │        │ Distributed Orch    │        │ Distributed Orch    │
│ ├─ Affinity Calc    │        │ ├─ Affinity Calc    │        │ ├─ Affinity Calc    │
│ ├─ Consensus        │        │ ├─ Consensus        │        │ ├─ Consensus        │
│ ├─ Decision Router  │        │ ├─ Decision Router  │        │ ├─ Decision Router  │
│ ├─ Group Manager    │        │ ├─ Group Manager    │        │ ├─ Group Manager    │
│ └─ Local LLM        │        │ └─ Local LLM        │        │ └─ Local LLM        │
└─────────────────────┘        └─────────────────────┘        └─────────────────────┘
         │                              │                              │
         └──────────────────────────────┴──────────────────────────────┘
                              libp2p P2P Network
```

**Não há servidor central** - cada node é igual e autônomo. A coordenação emerge naturalmente através de:
1. Cálculo local de affinity (cada node calcula seu próprio score)
2. Consenso distribuído (todos participam da votação)
3. Comunicação P2P direta (sem intermediário)
