


## Decision Making & Orchestration

### Reusing Existing Intelligence

**CRITICAL PRINCIPLE**: This feature creates **ZERO new algorithms**. All decision-making leverages the existing LLM Orchestrator and Distributed Orchestration infrastructure.

### Existing Components Used

```rust
// All components already implemented in mvp-node/src/

1. LLMOrchestrator           // orchestration/orchestrator.rs
2. DecisionRouter            // distributed_orchestration/decision_router.rs
3. AffinityCalculator        // distributed_orchestration/affinity.rs
4. AdaptiveConsensus         // distributed_orchestration/consensus.rs
5. ContextGroupManager       // distributed_orchestration/context_groups.rs
6. PartitionHandler          // distributed_orchestration/partition.rs
```

### Decision Types and Strategies

#### 1. Coordinator Selection (Heuristic)

**Decision**: "Which node should coordinate this distributed inference?"

**Strategy**: `DecisionStrategy::Heuristic` (fast, metrics-based)

**Process**:
```
1. All nodes see job via gossipsub
2. Each node calculates affinity score:
   - Proximity to required shards (0-1)
   - Current load (0-1, inverted)
   - Historical success rate (0-1)
   - Network latency to other shard hosts (0-1, inverted)
   
3. Affinity = weighted_sum([proximity, load, success, latency])
4. Node with highest affinity offers to coordinate
5. If no response in 2s, second-highest offers
```

**No LLM needed** - pure metrics, sub-second decision.

**Code Reference**:
```rust
// distributed_orchestration/affinity.rs
pub struct AffinityCalculator {
    // Already implemented
}

impl AffinityCalculator {
    pub fn calculate_coordinator_affinity(
        &self,
        node_state: &NodeState,
        required_shards: &[ShardId],
    ) -> f64 {
        // Existing implementation
    }
}
```

#### 2. Partitioning Strategy Selection (LLM)

**Decision**: "How should we partition this new model?"

**Strategy**: `DecisionStrategy::LLM` (complex, requires analysis)

**Process**:
```
1. Model arrives: "mixtral-8x7b-instruct"
2. LLM Orchestrator analyzes:
   - Model architecture (MoE with 8 experts)
   - Available nodes (15 nodes, varying VRAM)
   - Network topology (3 geographic clusters)
   - Bandwidth measurements
   
3. LLM generates decision:
   {
     "decision": "expert_wise_partitioning",
     "reasoning": "MoE architecture with 8 experts maps naturally to 8 shards...",
     "confidence": 0.85,
     "actions": [
       {"action": "create_shard", "params": {"expert_id": 0, "target_node": "node-5"}},
       ...
     ]
   }
   
4. DecisionValidator validates feasibility
5. Partitioner executes strategy
```

**Code Reference**:
```rust
// orchestration/orchestrator.rs
impl LLMOrchestrator {
    pub async fn make_decision(
        &self,
        context: &str,
        decision_type: DecisionType,
    ) -> Result<Decision> {
        // Already implemented
    }
}

// Usage for partitioning:
let context = format!(
    "Model: {}, Architecture: {}, Available nodes: {}, Network: {}",
    model_name, architecture, nodes, topology
);

let decision = orchestrator.make_decision(
    &context,
    DecisionType::Infrastructure
).await?;
```

#### 3. Shard Placement Optimization (Hybrid)

**Decision**: "Where should we place these shards for optimal performance?"

**Strategy**: `DecisionStrategy::Hybrid` (heuristic first, LLM if complex)

**Process**:
```
1. Heuristic attempts placement:
   - Rule: Place dependent shards on nearby nodes
   - Rule: Balance load across nodes
   - Rule: Prefer nodes with available VRAM
   
2. If heuristic succeeds (simple case):
   - Fast placement (< 100ms)
   - No LLM invocation
   
3. If heuristic fails (complex constraints):
   - LLM analyzes constraints
   - Suggests creative placement
   - Example: "Split shard 3 into sub-shards to fit available VRAM"
```

**Code Reference**:
```rust
// distributed_orchestration/decision_router.rs
impl DecisionRouter {
    pub async fn route_decision(
        &self,
        context: &DecisionContext,
    ) -> Result<DecisionResult> {
        match self.select_strategy(context) {
            DecisionStrategy::Heuristic => {
                self.heuristic_strategy.decide(context).await
            }
            DecisionStrategy::LLM => {
                self.llm_orchestrator.make_decision(context).await
            }
            DecisionStrategy::Hybrid => {
                // Try heuristic first
                match self.heuristic_strategy.decide(context).await {
                    Ok(decision) => Ok(decision),
                    Err(_) => {
                        // Fall back to LLM
                        self.llm_orchestrator.make_decision(context).await
                    }
                }
            }
        }
    }
}
```

#### 4. Failure Recovery (Hybrid + Consensus)

**Decision**: "Shard 3 failed mid-inference, what should we do?"

**Strategy**: `DecisionStrategy::Hybrid` + `AdaptiveConsensus` (if critical)

**Process**:
```
1. Failure detected (shard 3 on node-7 unresponsive)

2. Heuristic attempts recovery:
   - Check Shard Registry for alternative hosts
   - If found: Redirect to alternative (fast!)
   - If not found: Escalate to LLM
   
3. LLM analyzes situation:
   - Context: "Shard 3 failed, no alternatives, 60% through inference"
   - Options: 
     a) Restart from checkpoint
     b) Abort and retry later
     c) Wait for node-7 to recover
   - Decision: "Restart from checkpoint on node-12"
   
4. If decision is critical (affects multiple jobs):
   - Adaptive Consensus invoked
   - Multiple nodes vote on decision
   - Threshold reached → execute
   - Threshold not reached → escalate to higher consensus
```

**Code Reference**:
```rust
// distributed_orchestration/consensus.rs
impl AdaptiveConsensus {
    pub async fn reach_consensus(
        &self,
        proposal: &Proposal,
        participants: &[NodeId],
    ) -> Result<ConsensusResult> {
        // Response Threshold Model (bio-inspired)
        // Already implemented
    }
}

// Usage for critical decisions:
if decision.is_critical() {
    let proposal = Proposal::from_decision(&decision);
    let result = consensus.reach_consensus(
        &proposal,
        &affected_nodes
    ).await?;
    
    if result.accepted {
        execute_decision(&decision).await?;
    }
}
```

### Emergent Coordination (No Central Authority)

**Key Principle**: Coordinators **emerge** based on objective metrics, not fixed assignment.

#### Coordinator Selection Algorithm

```
For each distributed inference job:

1. Job broadcast via gossipsub to all nodes
2. Each node independently calculates affinity:
   
   affinity = w1 * shard_proximity +
              w2 * (1 - current_load) +
              w3 * success_rate +
              w4 * (1 - avg_latency)
   
   where:
   - shard_proximity: % of required shards hosted locally
   - current_load: CPU/GPU utilization (0-1)
   - success_rate: historical success rate (0-1)
   - avg_latency: average latency to other shard hosts (normalized)
   
3. Node with affinity > threshold offers to coordinate
4. If multiple offers, highest affinity wins
5. If no offers, job requester selects from available nodes
6. Coordinator executes job
7. Next job may have different coordinator
```

**Guarantees**:
- ✅ No single point of failure (any node can coordinate)
- ✅ No central authority (selection is distributed)
- ✅ Self-organizing (coordinators emerge based on capability)
- ✅ Fault-tolerant (coordinator failure → re-selection)
- ✅ Censorship-resistant (client can submit to any node)

#### Context Groups for Shard Clusters

**Existing Component**: `ContextGroupManager`

**Usage**: Nodes hosting related shards form dynamic groups

```
Example: "llama-70b" with 8 shards

1. Nodes hosting shards 1-4 form "llama-70b-group-A"
2. Nodes hosting shards 5-8 form "llama-70b-group-B"
3. Groups coordinate internally for efficiency
4. Cross-group communication for full inference
5. Groups reform dynamically as nodes join/leave
```

**Code Reference**:
```rust
// distributed_orchestration/context_groups.rs
impl ContextGroupManager {
    pub async fn form_shard_group(
        &self,
        model_id: &str,
        shard_ids: &[ShardId],
    ) -> Result<GroupId> {
        // Already implemented
    }
}
```

### Decision Flow Examples

#### Example 1: Simple Inference (Heuristic Only)

```
Job: Inference on "llama-13b" (2 shards, both available)

1. Job arrives → DecisionRouter
2. Router: "Simple case, use Heuristic"
3. AffinityCalculator: Node-5 has both shards, affinity=0.95
4. Node-5 coordinates
5. Inference completes
6. Total decision time: 50ms
```

#### Example 2: Complex Partitioning (LLM)

```
Job: Partition "gpt-neox-20b" for network

1. Partitioning request → DecisionRouter
2. Router: "Complex, use LLM"
3. LLM analyzes:
   - Model: 44 layers, 20B parameters
   - Network: 12 nodes, mixed VRAM (8GB-24GB)
   - Bandwidth: 3 clusters with high intra-cluster bandwidth
4. LLM decides: "Hybrid strategy - pipeline + tensor parallelism"
5. Validator confirms feasibility
6. Partitioner executes
7. Total decision time: 2.5s
```

#### Example 3: Failure Recovery (Hybrid + Consensus)

```
Job: Inference on "llama-70b" fails at shard 5

1. Failure detected → DecisionRouter
2. Router: "Try Heuristic first"
3. Heuristic: Check registry for shard 5 alternatives
4. Registry: No alternatives available
5. Router: "Escalate to LLM"
6. LLM analyzes:
   - Progress: 62% complete
   - Checkpoint: Available at shard 4
   - Options: Restart vs Abort
7. LLM decides: "Restart from shard 4 checkpoint"
8. Decision is critical (affects 3 queued jobs)
9. AdaptiveConsensus invoked with 5 nodes
10. Consensus reached (4/5 agree)
11. Restart executed
12. Total recovery time: 4.2s
```

### Integration with Existing Systems

#### With LLM Orchestrator

```rust
// Distributed supermodel decisions use same interface
let decision = llm_orchestrator.make_decision(
    &context,
    DecisionType::Infrastructure
).await?;

// Decision types extended for distributed inference:
pub enum DecisionType {
    Infrastructure,      // Existing
    JobRouting,         // Existing
    ContextManagement,  // Existing
    ShardPlacement,     // NEW - but uses existing infrastructure
    PartitionStrategy,  // NEW - but uses existing infrastructure
}
```

#### With Distributed Orchestration

```rust
// Shard coordination uses existing affinity calculation
let affinity = affinity_calculator.calculate_affinity(
    &node_state,
    &decision_context
);

// Shard groups use existing context group manager
let group_id = context_group_manager.form_group(
    "llama-70b-shards",
    &shard_nodes
).await?;

// Critical decisions use existing consensus
let consensus = adaptive_consensus.reach_consensus(
    &proposal,
    &participants
).await?;
```

### No New Algorithms Summary

| Functionality | Existing Component | Location |
|--------------|-------------------|----------|
| Coordinator selection | AffinityCalculator | distributed_orchestration/affinity.rs |
| Complex decisions | LLMOrchestrator | orchestration/orchestrator.rs |
| Strategy routing | DecisionRouter | distributed_orchestration/decision_router.rs |
| Critical consensus | AdaptiveConsensus | distributed_orchestration/consensus.rs |
| Shard grouping | ContextGroupManager | distributed_orchestration/context_groups.rs |
| Failure detection | PartitionHandler | distributed_orchestration/partition.rs |
| Coordinator rotation | CoordinatorRotation | distributed_orchestration/rotation.rs |

**Result**: 100% reuse of existing intelligence infrastructure. Zero new decision-making algorithms.



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Partition Completeness

*For any* model with N parameters, partitioning SHALL produce shards that collectively contain all N parameters with no loss or duplication.

**Validates: Requirements 1.1, 1.5**

**Verification Strategy**: Generate random model configurations, partition them, and verify that sum of shard parameters equals original model parameters.

### Property 2: Shard Round-Trip Integrity

*For any* partitioned model, loading all shards and combining their outputs SHALL produce results equivalent to the original unpartitioned model.

**Validates: Requirements 1.5**

**Verification Strategy**: Partition model, execute inference on both partitioned and original versions with same input, compare outputs within tolerance.

### Property 3: Registry Consistency

*For any* shard loaded on a node, the Shard Registry SHALL contain an entry mapping that shard to that node within 5 seconds.

**Validates: Requirements 2.2, 5.1**

**Verification Strategy**: Load random shards on random nodes, query registry, verify all entries exist within time bound.

### Property 4: Coordinator Discovery Completeness

*For any* distributed inference request, if all required shards are available in the network, the Shard Coordinator SHALL successfully identify all shard locations.

**Validates: Requirements 3.1, 5.1**

**Verification Strategy**: Create network with random shard distribution, submit jobs, verify coordinator finds all required shards.

### Property 5: Tensor Compression Threshold

*For any* tensor with size > 1MB, the Tensor Router SHALL apply compression before transmission.

**Validates: Requirements 4.2**

**Verification Strategy**: Generate tensors of varying sizes, route them, verify compression is applied only when size exceeds threshold.

### Property 6: Retry Exhaustion

*For any* tensor transfer that fails, the Tensor Router SHALL retry exactly 3 times before reporting failure.

**Validates: Requirements 4.4**

**Verification Strategy**: Inject transfer failures, count retry attempts, verify exactly 3 retries occur.

### Property 7: Affinity Monotonicity

*For any* two nodes A and B, if node A has higher affinity score than node B for a job, then node A SHALL be selected as coordinator over node B.

**Validates: Requirements 5.2, 5.4**

**Verification Strategy**: Generate nodes with known affinity scores, verify selection always picks highest affinity.

### Property 8: Registry Update Timeliness

*For any* network topology change, the Shard Registry SHALL reflect the change within 5 seconds.

**Validates: Requirements 5.3**

**Verification Strategy**: Simulate node joins/leaves, measure time until registry updates, verify all updates occur within bound.

### Property 9: Failover Activation

*For any* shard host that becomes unavailable during inference, the Shard Coordinator SHALL detect the failure within 5 seconds and attempt failover.

**Validates: Requirements 5.5, 10.1**

**Verification Strategy**: Kill shard hosts mid-inference, measure detection time, verify failover attempts.

### Property 10: Metadata Completeness

*For any* partitioning operation, the generated manifest SHALL contain complete metadata describing all shards, dependencies, and tensor flow.

**Validates: Requirements 1.4, 6.5**

**Verification Strategy**: Partition random models, validate manifest contains all required fields and relationships.

### Property 11: Format Compatibility Validation

*For any* set of shards belonging to the same model, if they have incompatible formats, the Inference Engine SHALL reject loading them together.

**Validates: Requirements 8.5**

**Verification Strategy**: Create shards with mixed formats, attempt to load, verify rejection with clear error.

### Property 12: Decision Router Strategy Selection

*For any* decision context, the Decision Router SHALL select a strategy (Heuristic/LLM/Hybrid) based on decision complexity and criticality.

**Validates: Requirements 9.5**

**Verification Strategy**: Generate decision contexts with varying complexity, verify strategy selection matches expected criteria.

### Property 13: Receipt Completeness

*For any* successful distributed inference, the generated receipt SHALL include all participating nodes with their execution times and signatures.

**Validates: Requirements 10.4**

**Verification Strategy**: Execute distributed inference, verify receipt contains entries for all nodes that processed shards.

### Property 14: Resource Cleanup on Failure

*For any* distributed inference that fails, the Shard Coordinator SHALL release all reserved resources on all participating nodes.

**Validates: Requirements 10.3**

**Verification Strategy**: Inject failures at various stages, verify no resources remain locked after failure.

### Property 15: Payment Distribution Fairness

*For any* completed distributed inference via pool, the payment SHALL be split across all participating shard hosts proportional to their contribution.

**Validates: Requirements 11.3**

**Verification Strategy**: Execute inference with varying shard execution times, verify payment splits match contribution ratios.

### Property 16: Coordinator Rotation

*For any* sequence of N distributed inference jobs, if node A coordinates job i, there is no guarantee node A coordinates job i+1 (coordinators rotate based on affinity).

**Validates: Requirements 9.1, 9.2**

**Verification Strategy**: Execute multiple jobs, track coordinator assignments, verify rotation occurs based on changing affinity scores.

### Property 17: Consensus Threshold Satisfaction

*For any* critical decision requiring consensus, the decision SHALL only execute if the number of agreeing nodes exceeds the adaptive threshold.

**Validates: Requirements 9.2**

**Verification Strategy**: Propose critical decisions with varying agreement levels, verify execution only when threshold exceeded.

### Property 18: Shard Deregistration Cleanup

*For any* shard that is unloaded from a node, the Shard Registry SHALL remove the corresponding entry and the node SHALL notify all peers.

**Validates: Requirements 2.5**

**Verification Strategy**: Load then unload shards, verify registry cleanup and peer notifications occur.

### Property 19: Error Information Completeness

*For any* failed distributed inference, the error report SHALL identify which specific shard failed and the failure reason.

**Validates: Requirements 3.5, 10.5**

**Verification Strategy**: Inject failures at specific shards, verify error reports correctly identify failed shard and cause.

### Property 20: Metrics Collection Completeness

*For any* distributed inference execution, the system SHALL log per-shard execution time, tensor transfer sizes, and total end-to-end latency.

**Validates: Requirements 7.1, 7.2, 7.3**

**Verification Strategy**: Execute inference, verify all required metrics are present in logs with valid values.



## Error Handling

### Error Categories

#### 1. Partitioning Errors

**Causes**:
- Invalid model format
- Insufficient metadata
- Unsupported architecture

**Handling**:
```rust
pub enum PartitionError {
    InvalidFormat { format: String, reason: String },
    InsufficientMetadata { missing_fields: Vec<String> },
    UnsupportedArchitecture { architecture: String },
    ValidationFailed { details: String },
}

impl PartitionError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::InvalidFormat { .. } => RecoveryStrategy::Abort,
            Self::InsufficientMetadata { .. } => RecoveryStrategy::RequestMetadata,
            Self::UnsupportedArchitecture { .. } => RecoveryStrategy::Abort,
            Self::ValidationFailed { .. } => RecoveryStrategy::RetryWithDifferentStrategy,
        }
    }
}
```

#### 2. Shard Loading Errors

**Causes**:
- Corrupted shard file
- Hash mismatch
- Insufficient memory

**Handling**:
```rust
pub enum ShardLoadError {
    CorruptedFile { shard_id: String, expected_hash: String, actual_hash: String },
    InsufficientMemory { required_mb: u64, available_mb: u64 },
    FormatMismatch { expected: CheckpointFormat, found: CheckpointFormat },
}

impl ShardLoadError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::CorruptedFile { .. } => RecoveryStrategy::DownloadFromAlternativeHost,
            Self::InsufficientMemory { .. } => RecoveryStrategy::UnloadOtherShards,
            Self::FormatMismatch { .. } => RecoveryStrategy::Abort,
        }
    }
}
```

#### 3. Coordination Errors

**Causes**:
- Missing shards
- Coordinator failure
- Timeout

**Handling**:
```rust
pub enum CoordinationError {
    MissingShards { missing: Vec<ShardId> },
    CoordinatorUnreachable { coordinator_id: String },
    Timeout { stage: String, elapsed_ms: u64 },
    ConsensusFailure { proposal: String, votes_for: usize, votes_against: usize },
}

impl CoordinationError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::MissingShards { .. } => RecoveryStrategy::WaitForShards { timeout_secs: 30 },
            Self::CoordinatorUnreachable { .. } => RecoveryStrategy::SelectNewCoordinator,
            Self::Timeout { .. } => RecoveryStrategy::RetryWithIncreasedTimeout,
            Self::ConsensusFailure { .. } => RecoveryStrategy::EscalateToHigherConsensus,
        }
    }
}
```

#### 4. Tensor Routing Errors

**Causes**:
- Network partition
- Transfer timeout
- Compression failure

**Handling**:
```rust
pub enum TensorRoutingError {
    NetworkPartition { from_node: String, to_node: String },
    TransferTimeout { tensor_id: String, elapsed_ms: u64 },
    CompressionFailed { reason: String },
    DecompressionFailed { reason: String },
}

impl TensorRoutingError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::NetworkPartition { .. } => RecoveryStrategy::RouteViaAlternativePath,
            Self::TransferTimeout { .. } => RecoveryStrategy::RetryWithBackoff,
            Self::CompressionFailed { .. } => RecoveryStrategy::SendUncompressed,
            Self::DecompressionFailed { .. } => RecoveryStrategy::RequestRetransmission,
        }
    }
}
```

### Recovery Strategies

```rust
pub enum RecoveryStrategy {
    Abort,
    Retry { max_attempts: usize, backoff_ms: u64 },
    RetryWithBackoff,
    RetryWithDifferentStrategy,
    SelectNewCoordinator,
    RouteViaAlternativePath,
    WaitForShards { timeout_secs: u64 },
    DownloadFromAlternativeHost,
    UnloadOtherShards,
    RequestMetadata,
    SendUncompressed,
    RequestRetransmission,
    EscalateToHigherConsensus,
}
```

### Error Propagation

```
Shard Execution Error
        ↓
Coordinator detects failure
        ↓
Coordinator attempts recovery (3 retries)
        ↓
If recovery fails:
        ↓
Generate detailed error report
        ↓
Return to client with:
  - Failed shard ID
  - Failure reason
  - Attempted recovery steps
  - Suggestions for retry
```

## Testing Strategy

### Dual Testing Approach

This feature requires both **unit tests** and **property-based tests** for comprehensive coverage:

- **Unit tests**: Verify specific examples, edge cases, and error conditions
- **Property tests**: Verify universal properties across all inputs

Both are complementary and necessary. Unit tests catch concrete bugs, property tests verify general correctness.

### Property-Based Testing

**Framework**: Use `proptest` crate (Rust's property-based testing library)

**Configuration**: Minimum 100 iterations per property test

**Test Tagging**: Each property test must reference its design document property

**Tag Format**: `// Feature: distributed-supermodel, Property {number}: {property_text}`

### Test Categories

#### 1. Partitioning Tests

**Unit Tests**:
- Partition specific models (TinyLlama, Mistral-7B)
- Test each partitioning strategy individually
- Verify manifest generation for known models

**Property Tests**:
- Property 1: Partition completeness (all parameters preserved)
- Property 2: Round-trip integrity (partition → combine → equivalent)
- Property 10: Metadata completeness (manifest always complete)

**Example**:
```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    // Feature: distributed-supermodel, Property 1: Partition Completeness
    proptest! {
        #[test]
        fn test_partition_preserves_all_parameters(
            param_count in 8_000_000_000u64..100_000_000_000u64,
            shard_count in 2usize..16usize,
        ) {
            let model = generate_mock_model(param_count);
            let result = partitioner.partition(&model, shard_count).await?;
            
            let total_shard_params: u64 = result.shards
                .iter()
                .map(|s| s.parameter_count)
                .sum();
            
            assert_eq!(total_shard_params, param_count);
        }
    }
}
```

#### 2. Registry Tests

**Unit Tests**:
- Register/deregister specific shards
- Test registry queries with known data
- Verify update notifications

**Property Tests**:
- Property 3: Registry consistency (load → registry entry)
- Property 8: Update timeliness (change → update within 5s)
- Property 18: Deregistration cleanup (unload → registry cleanup)

#### 3. Coordination Tests

**Unit Tests**:
- Coordinate inference with 2-4 shards
- Test specific failure scenarios
- Verify receipt generation

**Property Tests**:
- Property 4: Discovery completeness (all shards found)
- Property 7: Affinity monotonicity (highest affinity wins)
- Property 9: Failover activation (failure → detection → failover)
- Property 13: Receipt completeness (all nodes in receipt)

#### 4. Tensor Routing Tests

**Unit Tests**:
- Route tensors between specific nodes
- Test compression with known tensor sizes
- Verify retry logic with injected failures

**Property Tests**:
- Property 5: Compression threshold (>1MB → compressed)
- Property 6: Retry exhaustion (exactly 3 retries)
- Property 20: Metrics collection (all metrics logged)

#### 5. Integration Tests

**Scenarios**:
- Full distributed inference with 4-8 shards
- Coordinator failure mid-inference
- Network partition during tensor transfer
- Concurrent inference requests
- Shard host failure and recovery

**Example**:
```rust
#[tokio::test]
async fn test_full_distributed_inference_llama_70b() {
    // Setup: 8 nodes, each hosting 1 shard of llama-70b
    let network = setup_test_network(8).await;
    let model = load_model_shards("llama-70b", &network).await;
    
    // Execute inference
    let request = InferenceRequest {
        model_id: "llama-70b".to_string(),
        input: "Hello, world!".to_string(),
    };
    
    let result = network.execute_distributed_inference(request).await?;
    
    // Verify
    assert!(result.output.len() > 0);
    assert_eq!(result.participating_nodes.len(), 8);
    assert!(result.total_latency_ms < 10000); // < 10s
}
```

#### 6. Fault Injection Tests

**Scenarios**:
- Kill shard host at random stage
- Inject network latency (100ms-5s)
- Corrupt tensor data mid-transfer
- Exhaust memory on node
- Partition network into clusters

**Example**:
```rust
#[tokio::test]
async fn test_shard_failure_recovery() {
    let network = setup_test_network(8).await;
    let model = load_model_shards("llama-70b", &network).await;
    
    // Start inference
    let inference_task = tokio::spawn(async move {
        network.execute_distributed_inference(request).await
    });
    
    // Inject failure at random stage
    tokio::time::sleep(Duration::from_millis(500)).await;
    network.kill_node(3).await; // Kill node hosting shard 3
    
    // Verify recovery
    let result = inference_task.await??;
    assert!(result.is_ok());
    assert!(result.recovery_attempts > 0);
}
```

### Test Coverage Goals

| Component | Unit Tests | Property Tests | Integration Tests |
|-----------|-----------|----------------|-------------------|
| Model Partitioner | 15 | 5 | 3 |
| Shard Registry | 12 | 4 | 2 |
| Shard Coordinator | 20 | 6 | 5 |
| Tensor Router | 18 | 5 | 4 |
| Distributed Inference Engine | 15 | 4 | 3 |
| Error Handling | 10 | 3 | 2 |
| **Total** | **90** | **27** | **19** |

### Simulation Mode

For local testing without multiple machines:

```rust
pub struct SimulatedNetwork {
    nodes: Vec<SimulatedNode>,
    artificial_latency_ms: u64,
    failure_injection: FailureInjector,
}

impl SimulatedNetwork {
    pub fn new(node_count: usize) -> Self {
        // Create simulated nodes on single machine
    }
    
    pub fn set_latency(&mut self, latency_ms: u64) {
        // Add artificial delay to tensor transfers
    }
    
    pub fn inject_failure(&mut self, node_id: usize, failure_type: FailureType) {
        // Simulate node/network failures
    }
}
```

## Performance Considerations

### Latency Optimization

**Target**: Distributed inference latency < 3x single-node inference

**Strategies**:
1. **Tensor Compression**: Reduce bandwidth usage by 60-80%
2. **Pipeline Parallelism**: Overlap computation and communication
3. **Shard Placement**: Co-locate dependent shards when possible
4. **Connection Pooling**: Reuse P2P connections across requests

### Bandwidth Optimization

**Target**: < 100MB total bandwidth per inference for 70B model

**Strategies**:
1. **Quantized Tensors**: Use FP16 or INT8 for intermediate tensors
2. **Sparse Tensors**: Only transfer non-zero values
3. **Delta Encoding**: Transfer only changes from previous layer
4. **Batching**: Combine multiple requests when possible

### Memory Optimization

**Target**: Each node hosts 1-3 shards with < 16GB VRAM

**Strategies**:
1. **Shard Quantization**: Use Q4_K_M or Q8_0 for shards
2. **Lazy Loading**: Load shards on-demand
3. **Memory Pooling**: Share memory across shards when possible
4. **Offloading**: Move inactive shards to CPU memory

### Scalability

**Target**: Support models up to 405B parameters (Llama 3.1)

**Approach**:
- 405B model → 64 shards of ~6.3B each
- Each shard fits in 16GB VRAM (quantized)
- Network of 64 nodes can host entire model
- Inference latency: ~15-30s (acceptable for batch processing)

## Security Considerations

### Shard Integrity

**Threat**: Malicious node provides corrupted shard

**Mitigation**:
- Cryptographic hash verification on shard load
- Multiple nodes host same shard (redundancy)
- Canary tasks to detect corruption

### Tensor Tampering

**Threat**: Malicious node modifies intermediate tensors

**Mitigation**:
- Redundant computation (2-3 nodes compute same shard)
- Output comparison and voting
- Reputation system penalizes inconsistent nodes

### Coordinator Manipulation

**Threat**: Malicious coordinator routes to compromised nodes

**Mitigation**:
- Coordinator selection is distributed (affinity-based)
- Client can specify trusted nodes
- Reputation system tracks coordinator behavior

### Data Privacy

**Threat**: Sensitive input data exposed to multiple nodes

**Mitigation**:
- Homomorphic encryption for sensitive inputs (future)
- Trusted execution environments (TEE) for shards (future)
- Client can select nodes in trusted jurisdictions

## Future Enhancements

### Phase 1 (Current Spec)
- ✅ Pipeline parallelism
- ✅ Model parallelism
- ✅ MoE routing
- ✅ Basic tensor compression
- ✅ Fault tolerance

### Phase 2 (Future)
- Advanced quantization (INT4, INT2)
- Speculative execution (predict next tokens)
- Cross-shard caching
- Dynamic shard migration
- Load balancing across replicas

### Phase 3 (Future)
- Homomorphic encryption
- Trusted execution environments
- Zero-knowledge proofs for computation
- Federated learning integration
- Model fine-tuning across shards

## Conclusion

The Distributed Supermodel feature extends the decentralized AI network to support large models (>7B parameters) by:

1. **Leveraging existing infrastructure**: 100% reuse of P2P, orchestration, and monitoring
2. **Maintaining decentralization**: No central authority, emergent coordination
3. **Ensuring correctness**: 20 formal properties with property-based testing
4. **Optimizing performance**: Compression, pipelining, intelligent placement
5. **Handling failures**: Comprehensive error handling and recovery

This implementation fulfills the "Hybrid Execution Layer" vision from the whitepaper, enabling the network to serve both micromodels (≤7B) and supermodels (>7B) seamlessly.
