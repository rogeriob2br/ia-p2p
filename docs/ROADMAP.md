# 🗺️ Roadmap de Desenvolvimento - IA Descentralizada P2P

> **Última Atualização**: Dezembro 2024  
> **Status do Projeto**: Em Desenvolvimento Ativo

## 📋 Índice

1. [Visão Geral](#visão-geral)
2. [Arquitetura do Sistema](#arquitetura-do-sistema)
3. [Features Implementadas](#features-implementadas)
4. [Features em Desenvolvimento](#features-em-desenvolvimento)
5. [Roadmap de Implementação](#roadmap-de-implementação)
6. [Especificações Completas](#especificações-completas)

---

## 🎯 Visão Geral

Este projeto implementa uma rede descentralizada de IA peer-to-peer (P2P) com orquestração cognitiva usando LLMs locais. O sistema permite que múltiplos nodes compartilhem recursos computacionais para executar modelos de IA de forma distribuída, com uma camada de raciocínio inteligente para gerenciar infraestrutura e contextos.

### Princípios Fundamentais

1. **Descentralização Total**: Sem pontos únicos de falha
2. **Autonomia Local**: Nodes funcionam sem dependências externas
3. **Orquestração Cognitiva**: LLM decide O QUE fazer, infraestrutura decide COMO fazer
4. **Privacidade**: Todos os dados e modelos permanecem locais
5. **Escalabilidade**: Crescimento horizontal através de P2P

---

## 🏗️ Arquitetura do Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                        MVP CLI (Interface)                       │
│  • Job Management  • Node Monitoring  • Network Explorer        │
│  • Natural Language Commands  • Multi-node Testing              │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    LLM Orchestrator (Cognição)                   │
│  • Decision Engine  • Metrics Aggregator  • Validator           │
│  • Training Collector  • Natural Language Interface             │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                      MVP Node (Infraestrutura)                   │
│  • P2P Network  • Job Queue  • Inference Engine                 │
│  • Health Monitor  • Context Management                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✅ Features Implementadas

### 1. MVP Node (Base Infrastructure)
**Status**: ✅ Completo (80% testado)  
**Localização**: `mvp-node/`

#### Funcionalidades
- ✅ **P2P Networking** (libp2p)
  - Gossipsub para broadcast de mensagens
  - Request-response para comunicação direta
  - mDNS para descoberta local de peers
  - Identificação e autenticação de peers

- ✅ **Job Management**
  - Fila de jobs com priorização
  - Distribuição de jobs entre peers
  - Tracking de status e resultados
  - Retry automático em falhas

- ✅ **Inference Engine**
  - Suporte a modelos Candle (HuggingFace)
  - Execução local de inferência
  - Gerenciamento de memória
  - Batch processing

- ✅ **Health Monitoring**
  - Coleta de métricas (jobs, network, inference)
  - Health checks automáticos
  - Logging estruturado (tracing)
  - Detecção de erros e alertas

#### Métricas Coletadas
```rust
JobMetrics {
    jobs_submitted, jobs_completed, jobs_failed,
    average_processing_time_ms, total_tokens_processed
}

NetworkMetrics {
    peer_connections, active_peers, messages_sent/received,
    bytes_sent/received
}

InferenceMetrics {
    inferences_executed, inferences_failed,
    average_inference_time_ms, model_loaded
}

HealthStatus {
    is_healthy, uptime_seconds, queue_size,
    error_count, last_error
}
```

#### Arquivos Principais
- `mvp-node/src/lib.rs` - Core traits e interfaces
- `mvp-node/src/monitoring/mod.rs` - Sistema de monitoramento
- `mvp-node/src/main.rs` - Entry point do node

---

### 2. MVP CLI (Command Line Interface)
**Status**: ✅ Completo (80% implementado)  
**Localização**: `mvp-cli/`  
**Spec**: `.kiro/specs/mvp-cli/`

#### Funcionalidades
- ✅ **Job Management**
  - Submit jobs para a rede
  - Listar jobs (pending, running, completed)
  - Cancelar jobs em execução
  - Ver resultados e logs

- ✅ **Node Monitoring**
  - Listar nodes descobertos
  - Ver informações detalhadas de nodes
  - Health checks de nodes
  - Visualizar métricas em tempo real

- ✅ **Network Explorer**
  - Visualizar topologia da rede
  - Ver estatísticas de peers
  - Monitorar conectividade

- ✅ **Multi-node Testing**
  - Iniciar múltiplos nodes localmente
  - Configurar bootstrap automático
  - Testes de carga e stress

- ✅ **Interactive REPL**
  - Modo interativo para exploração
  - Auto-complete de comandos
  - Histórico de comandos

#### Comandos Disponíveis
```bash
# Job Management
mvp-cli job submit --prompt "Hello" --model tinyllama
mvp-cli job list --status running
mvp-cli job cancel <job-id>
mvp-cli job result <job-id>

# Node Monitoring
mvp-cli node list --watch
mvp-cli node info <node-id>
mvp-cli node health <node-id>
mvp-cli node metrics <node-id> --watch

# Network
mvp-cli network topology
mvp-cli network stats

# Multi-node Testing
mvp-cli test multi-node --count 3
mvp-cli test load --jobs 100

# Interactive
mvp-cli repl
```

#### Especificação Completa
- **Requirements**: 10 user stories, 50 critérios de aceitação
- **Design**: 50 propriedades de corretude
- **Tasks**: 19 tarefas principais, 100+ subtarefas
- **Testes**: 20 property tests implementados

---

## 🚧 Features em Desenvolvimento

### 3. LLM Orchestrator (Cognitive Layer)
**Status**: 🚧 Em Planejamento (Spec Completa)  
**Localização**: `mvp-node/src/orchestration/`  
**Spec**: `.kiro/specs/llm-orchestrator/`

#### Visão Geral
Camada de raciocínio cognitivo que usa modelos de linguagem locais para orquestração inteligente de infraestrutura e contextos. Implementa o princípio de separação entre cognição (LLM decide) e execução (infraestrutura executa).

#### Componentes Principais

##### 3.1 Local LLM Backend
**Responsabilidade**: Gerenciar modelos locais e executar inferência

**Funcionalidades**:
- Carregar modelos locais (Llama 3.2 3B, Mistral 7B, Phi-3 Mini)
- Verificar integridade de modelos (checksums)
- Executar inferência com parâmetros otimizados
- Suporte a múltiplos formatos (GGUF, SafeTensors)
- Detecção automática de device (CPU, CUDA, Metal)
- Quantização para eficiência de memória

**Modelos Suportados**:
```rust
SupportedModel {
    Llama3_2_3B,      // Recomendado para MVP (3B params)
    Mistral7B,        // Maior qualidade (7B params)
    Phi3Mini,         // Microsoft, eficiente (3.8B params)
    FineTuned,        // Modelos especializados
}
```

##### 3.2 Metrics Aggregator
**Responsabilidade**: Coletar e resumir estado do sistema para o LLM

**Funcionalidades**:
- Integrar com DefaultHealthMonitor existente
- Formatar estado como LLMState estruturado
- Calcular métricas derivadas (capacity, health, risk levels)
- Detectar mudanças significativas de estado
- Cache de estado com TTL

**Formato de Estado**:
```json
{
  "node": {
    "id": "node-12",
    "capacity": "high",
    "latency_avg": 40,
    "uptime_seconds": 3600,
    "error_count": 2
  },
  "context": {
    "id": "inference-queue",
    "health": "healthy",
    "queue_size": 5,
    "max_queue": 100,
    "processing_jobs": 3
  },
  "network": {
    "partition_risk": "low",
    "peers_available": 8,
    "active_connections": 5,
    "message_latency_avg": 45
  },
  "model": {
    "loaded": true,
    "avg_inference_ms": 1200,
    "memory_usage_mb": 4096,
    "recent_failures": 0
  }
}
```

##### 3.3 Decision Validator
**Responsabilidade**: Validar decisões do LLM antes da execução

**Funcionalidades**:
- Validar formato JSON das decisões
- Verificar ações contra allowlist
- Validar parâmetros dentro de limites seguros
- Registrar decisões rejeitadas
- Fallback para heurísticas determinísticas

**Validações**:
```rust
ValidationChecks {
    ✓ JSON schema válido
    ✓ Ações na allowlist
    ✓ Parâmetros dentro de bounds
    ✓ Confidence 0.0-1.0
    ✓ Reasoning não vazio
}
```

##### 3.4 Training Data Collector
**Responsabilidade**: Coletar dados para fine-tuning futuro

**Funcionalidades**:
- Registrar decisões com contexto completo
- Coletar feedback humano (approved/rejected/modified)
- Registrar outcomes e métricas resultantes
- Exportar dataset em formato JSONL
- Filtrar apenas exemplos validados e bem-sucedidos

**Formato de Treino**:
```json
{
  "id": "dec-uuid-1234",
  "state": { /* LLMState completo */ },
  "context": "Node overloaded, latency increasing",
  "decision_type": "infrastructure",
  "llm_decision": { /* Decision completa */ },
  "human_feedback": "approved",
  "outcome": {
    "type": "success",
    "improvement": 0.25,
    "description": "Latency reduced by 25%"
  },
  "metrics_before": { /* métricas */ },
  "metrics_after": { /* métricas */ },
  "timestamp": "2024-12-30T10:00:00Z"
}
```

##### 3.5 Natural Language Interface
**Responsabilidade**: Interpretar comandos em linguagem natural

**Funcionalidades**:
- Interpretar intenções do usuário
- Mapear para comandos CLI estruturados
- Solicitar confirmação para ações destrutivas
- Fornecer feedback em linguagem natural

**Exemplos de Uso**:
```bash
$ mvp-cli ask "start 3 nodes for testing"
🤖 I understand you want to start 3 test nodes. This will:
   • Start 3 MVP Node instances
   • Configure automatic bootstrap connections
   • Use ports 5000-5002
   
   Proceed? (y/N): y

✅ Started 3 nodes successfully

$ mvp-cli ask "show me unhealthy nodes"
🤖 Found 2 unhealthy nodes:
   • node-5: High error count (45 errors)
   • node-8: Model not loaded

$ mvp-cli ask "what should I do about high latency?"
🤖 Analyzing system state...
   
   Decision: migrate_context
   Confidence: 0.87
   Reasoning: Queue near capacity with high latency indicates 
              overload. Migration will distribute load.
   
   Suggested actions:
   1. Start election for context "inference-queue"
   2. Migrate to node with lower load
   
   Execute? (y/N):
```

##### 3.6 Decision Cache
**Responsabilidade**: Cache inteligente de decisões

**Funcionalidades**:
- Cache baseado em hash do estado
- LRU eviction policy
- TTL configurável (default: 5 minutos)
- Invalidação automática em mudanças de estado
- Cache hits < 10ms

---

#### Tipos de Decisão Especializados

O LLM Orchestrator suporta 3 tipos de decisão com configurações diferentes:

##### 🏗️ Infrastructure (Conservador)
**Foco**: Estabilidade e performance  
**Temperatura**: 0.3 (baixa criatividade)  
**Casos de Uso**:
- Sobrecarga de nodes
- Falhas de rede
- Migração de contextos
- Ajuste de parâmetros de gossip

**Exemplo**:
```json
{
  "decision": "migrate_context",
  "confidence": 0.87,
  "reasoning": "Queue at 95% capacity with 3000ms latency. Migration will distribute load.",
  "actions": [
    {
      "action_type": "start_election",
      "parameters": {"context": "inference-queue"},
      "priority": 8,
      "requires_confirmation": false
    }
  ],
  "estimated_impact": "Reduce latency by 40-60%",
  "rollback_plan": "Revert to original leader if migration fails"
}
```

##### 🎭 Context (Moderado)
**Foco**: Semântica e coerência  
**Temperatura**: 0.7 (criatividade moderada)  
**Casos de Uso**:
- Interpretação de eventos
- Correlação de estados
- Detecção de inconsistências
- Decisões de propagação

##### ⚖️ Mediation (Criativo)
**Foco**: Resolução de conflitos  
**Temperatura**: 0.9 (alta criatividade)  
**Casos de Uso**:
- Conflitos entre agentes
- Mediação de recursos
- Negociação de prioridades
- Resolução de deadlocks

---

#### Especificação Completa

**Requirements**: 10 user stories, 50 critérios de aceitação
- Orquestração com LLM local (sem APIs externas)
- Coleta de dados para fine-tuning
- Validação e segurança de decisões
- Interface de linguagem natural
- Integração com MVP Node existente

**Design**: 44 propriedades de corretude
- Decision Generation and Validation (5 properties)
- Data Collection and Training (4 properties)
- Validation and Safety (5 properties)
- Specialized Decision Types (5 properties)
- Performance and Monitoring (5 properties)
- Natural Language Interface (5 properties)
- Model Management (5 properties)
- Caching and Performance (5 properties)
- Integration with MVP Node (5 properties)

**Tasks**: 18 tarefas principais, 70+ subtarefas
- Fase 1: MVP com modelo base (Tasks 1-15)
- Fase 2: Coleta de dados (Tasks 16-17)
- Fase 3: Fine-tuning (Task 18 + futuro)

---

### 4. Inference Pool (Layer 2 API Gateway)
**Status**: 📋 Especificação Completa  
**Localização**: `inference-pool/` (a criar)  
**Spec**: `.kiro/specs/api-integration/`

#### Visão Geral
Componente Layer 2 que agrega nodes da rede P2P e expõe uma API unificada compatível com OpenAI. Pools são operadores independentes que facilitam o acesso à rede para desenvolvedores que preferem não gerenciar conexões P2P diretamente. Múltiplas pools podem coexistir e competir, preservando a natureza descentralizada do sistema.

#### Arquitetura Layer 2

```
┌─────────────────────────────────────────────────────────────────┐
│                         Clients                                  │
│  LangChain  •  CrewAI  •  n8n  •  AutoGPT  •  Custom Apps       │
└────────────────────────────┬────────────────────────────────────┘
                             │ OpenAI-Compatible API
┌────────────────────────────▼────────────────────────────────────┐
│                    Inference Pool (Layer 2)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ HTTP API    │  │ Job Router  │  │ Pool Orchestrator (LLM) │  │
│  │ Server      │  │             │  │                         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Auth        │  │ Pool        │  │ Network Discovery       │  │
│  │ Middleware  │  │ Registry    │  │ (P2P)                   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Usage       │  │ Heartbeat   │  │ Billing System          │  │
│  │ Tracker     │  │ Monitor     │  │ (Lightning Network)     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │ P2P Protocol
┌────────────────────────────▼────────────────────────────────────┐
│                      P2P Network (Layer 1)                       │
│  Node 1  •  Node 2  •  Node 3  •  ...  •  Other Pools           │
└─────────────────────────────────────────────────────────────────┘
```

#### Componentes Principais

##### 4.1 HTTP API Server (OpenAI-Compatible)
**Endpoints**:
- `POST /v1/chat/completions` - Chat completions (streaming e não-streaming)
- `POST /v1/embeddings` - Geração de embeddings
- `GET /v1/models` - Lista de modelos disponíveis
- `GET /v1/pool/status` - Status da pool
- `GET /v1/pool/info` - Informações públicas (fees, SLAs)
- `GET /v1/usage` - Consumo do cliente
- `GET /v1/billing/invoices` - Faturas do cliente
- `GET /v1/billing/earnings` - Ganhos dos Pool_Members

##### 4.2 Pool Registry
**Funcionalidades**:
- Registro de Pool_Members (nodes)
- Armazenamento de capacidades (modelos, GPU, memória)
- Tracking de status (Active, Inactive, Draining)
- Métricas por membro (latência, success rate, load)

##### 4.3 Job Router
**Estratégias de Roteamento**:
- Round Robin
- Least Loaded
- Lowest Latency
- Weighted Score (combinação de métricas)

**Funcionalidades**:
- Seleção de membro baseada em modelo e métricas
- Retry automático em falha (max 3 tentativas)
- Failover para membros alternativos

##### 4.4 Network Discovery (P2P)
**Funcionalidades**:
- Conectar à rede P2P como peer (libp2p)
- Escutar anúncios de nodes via gossipsub
- Convidar nodes descobertos para registro
- Descobrir outras pools para colaboração

##### 4.5 Pool Orchestrator (LLM-Powered)
**Funcionalidades**:
- Decisões inteligentes de roteamento via LLM local
- Análise de performance e sugestões de otimização
- Planejamento de alocação de recursos para SLAs
- Histórico de decisões para fine-tuning

**Tipos de Decisão**:
```rust
OrchestratorAction {
    RouteToMember { member_id, priority },
    ScaleUp { model, target_capacity },
    ScaleDown { member_id },
    AlertOperator { message, severity },
    AdjustRateLimit { client_id, new_limit },
}
```

##### 4.6 Billing System
**Funcionalidades**:
- Criação de tickets por inferência
- Cálculo de split (Pool_Fee vs pagamento ao node)
- Geração de invoices consolidadas
- Tracking de earnings para Pool_Members
- Suporte multi-moeda (USD, EUR, BRL, BTC, SATS)
- Integração com Lightning Network

**Modelo Econômico**:
```rust
BillingTicket {
    client_cost: Money,      // Custo total para o cliente
    member_payment: Money,   // Pagamento ao node
    pool_fee: Money,         // Taxa da pool (ex: 10%)
}
```

#### Especificação Completa

**Requirements**: 12 user stories, 40+ critérios de aceitação
1. Pool Membership - Registro e monitoramento de nodes
2. Chat Completions API - Compatibilidade OpenAI
3. Embeddings API - Geração de vetores
4. Models Discovery - Lista de modelos disponíveis
5. Job Routing - Distribuição inteligente
6. Pool Authentication - API Keys e rate limiting
7. Usage Tracking - Rastreamento de consumo
8. Pool Status - Transparência e monitoramento
9. Error Handling - Formato OpenAI
10. P2P Network Discovery - Descoberta automática
11. LLM-Powered Orchestration - Decisões inteligentes
12. Billing System - Cobrança e pagamentos

**Design**: 23 propriedades de corretude
- Node Registration Round-Trip (P1)
- Heartbeat Timeout Marks Inactive (P2)
- OpenAI Request Format Acceptance (P3)
- Message Roles Parsing (P4)
- Streaming Mode Determines Response Format (P5)
- Response Contains Required OpenAI Fields (P6)
- Embedding Input Flexibility (P7)
- Embedding Order Preservation (P8)
- Models Aggregation from Members (P9)
- Router Selects Model-Compatible Member (P10)
- Router Considers Load and Latency (P11)
- Router Retries on Failure (P12)
- API Key Validation Determines Access (P13)
- Inference Completion Triggers Tracking (P14)
- Error Responses Follow OpenAI Format (P15)
- Invalid Payload Returns 400 (P16)
- Network Discovery Connects to P2P (P17)
- Discovered Nodes Receive Invitations (P18)
- LLM Orchestrator Provides Routing Decisions (P19)
- Orchestrator Records Decision History (P20)
- Billing Ticket Creation (P21)
- Invoice Generation (P22)
- Multi-Currency Support (P23)

**Tasks**: 21 tarefas principais, 60+ subtarefas
- Fase 1: Core Infrastructure (Tasks 1-4)
- Fase 2: OpenAI API (Tasks 5-13)
- Fase 3: P2P Discovery e Orchestrator (Tasks 14-16)
- Fase 4: Billing System (Tasks 17-19)
- Fase 5: Integration (Tasks 20-21)

#### Tecnologias
- **HTTP Server**: Axum (async Rust)
- **P2P**: libp2p (gossipsub, mDNS)
- **LLM**: Candle ou llama.cpp
- **Payments**: Lightning Network (LND/CLN)
- **Storage**: SQLite ou PostgreSQL

---

## 📅 Roadmap de Implementação

### Fase 1: MVP com Modelo Base (2-4 semanas)
**Objetivo**: Sistema funcional com modelo Llama 3.2 3B base

#### Sprint 1: Core Infrastructure (Semana 1)
- [x] Task 1: Setup core infrastructure and dependencies
  - [x] Adicionar dependências ao Cargo.toml
  - [x] Criar estrutura do módulo orchestration
  - [ ] Definir traits e interfaces principais
  - [ ] Setup de tipos de erro

- [ ] Task 2: Implement Local LLM Backend
  - [ ] 2.1: LocalLLMBackend struct com candle
  - [ ] 2.2: Verificação de integridade de modelos
  - [ ] 2.3: Inference engine
  - [ ] 2.4: Suporte multi-modelo
  - [ ] 2.5-2.7: Property tests

- [ ] Task 3: Implement Metrics Aggregator
  - [ ] 3.1: MetricsAggregator struct
  - [ ] 3.2: Formatação de LLMState
  - [ ] 3.3: Cache de estado com TTL
  - [ ] 3.4-3.5: Property tests

#### Sprint 2: Decision Making (Semana 2)
- [ ] Task 4: Implement Decision Validator
  - [ ] 4.1: DecisionValidator struct
  - [ ] 4.2: Safety limits enforcement
  - [ ] 4.3: Error handling e fallbacks
  - [ ] 4.4-4.7: Property tests

- [ ] Task 5: Implement Training Data Collector
  - [ ] 5.1: TrainingDataCollector struct
  - [ ] 5.2: Decision recording
  - [ ] 5.3: Feedback integration
  - [ ] 5.4: Dataset export
  - [ ] 5.5-5.7: Property tests

- [ ] Task 6: Implement Core Orchestrator
  - [ ] 6.1: LLMOrchestrator struct
  - [ ] 6.2: Decision types e routing
  - [ ] 6.3: Prompt engineering
  - [ ] 6.4: Metrics e monitoring
  - [ ] 6.5-6.7: Property tests

#### Sprint 3: Advanced Features (Semana 3)
- [ ] Task 7: Implement Decision Cache
  - [ ] 7.1: DecisionCache struct com LRU
  - [ ] 7.2: Cache operations
  - [ ] 7.3: Performance optimization
  - [ ] 7.4-7.6: Property tests

- [ ] Task 8: Implement Natural Language Interface
  - [ ] 8.1: NaturalLanguageInterface struct
  - [ ] 8.2: Command mapping
  - [ ] 8.3: Confirmation handling
  - [ ] 8.4: Natural language feedback
  - [ ] 8.5-8.7: Property tests

#### Sprint 4: Integration (Semana 4)
- [ ] Task 9: Integrate with MVP Node
  - [ ] 9.1: Módulo opcional no MVP Node
  - [ ] 9.2: HTTP endpoints
  - [ ] 9.3: Logging integration
  - [ ] 9.4: Graceful degradation
  - [ ] 9.5-9.6: Property tests

- [ ] Task 10: Extend MVP CLI
  - [ ] 10.1: Orchestrator command group
  - [ ] 10.2: `ask` command
  - [ ] 10.3: Management commands
  - [ ] 10.4: Metrics display

- [ ] Task 11: Configuration and Model Management
  - [ ] 11.1: Extend config.toml
  - [ ] 11.2: Model download utilities
  - [ ] 11.3: Model switching e fallback
  - [ ] 11.4-11.5: Property tests

- [ ] Task 12-14: Testing, Examples, Documentation
- [ ] Task 15: Checkpoint - Ensure all tests pass

**Entregáveis**:
- ✅ Sistema funcional com Llama 3.2 3B
- ✅ Decisões de infraestrutura básicas
- ✅ Interface de linguagem natural
- ✅ Coleta de dados habilitada
- ✅ Documentação completa

---

### Fase 2: Coleta de Dados (2-3 meses)
**Objetivo**: Coletar 1000+ decisões validadas para fine-tuning

#### Atividades Contínuas
- [ ] Task 16: End-to-end testing com cenários reais
  - [ ] 16.1: Cenários de infraestrutura
  - [ ] 16.2: Interface de linguagem natural
  - [ ] 16.3: Coleta de training data

- [ ] Task 17: Performance optimization
  - [ ] 17.1: Otimizar model loading e inference
  - [ ] 17.2: Otimizar cache performance
  - [ ] 17.3: Otimizar decision pipeline

#### Cenários de Teste
1. **Node Overload**: Sobrecarga de queue e latência
2. **Network Partition**: Perda de conectividade
3. **Model Failure**: Falha de modelo e fallback
4. **Resource Contention**: Competição por recursos
5. **Cascading Failures**: Falhas em cascata

#### Métricas de Coleta
- 🎯 Target: 1000+ decisões validadas
- 📊 Diversidade de cenários (sobrecarga, falhas, conflitos)
- 👥 Feedback humano em decisões críticas
- 📈 Tracking de outcomes reais

**Entregáveis**:
- ✅ Dataset de 1000+ exemplos
- ✅ Feedback humano integrado
- ✅ Métricas de qualidade das decisões
- ✅ Análise de performance do modelo base

---

### Fase 3: Fine-tuning (1-2 semanas)
**Objetivo**: Modelo especializado para orquestração de infraestrutura

#### Preparação (Dias 1-2)
- [ ] Limpar e validar dataset
- [ ] Filtrar apenas exemplos bem-sucedidos
- [ ] Formatar para fine-tuning (JSONL)
- [ ] Dividir em train/validation/test

#### Fine-tuning (Dias 3-5)
- [ ] Setup ambiente de treino (GPU cloud)
- [ ] Configurar LoRA (Low-Rank Adaptation)
- [ ] Executar fine-tuning
- [ ] Validar modelo treinado

**Configuração de Treino**:
```yaml
base_model: meta-llama/Llama-3.2-3B-Instruct
method: LoRA
rank: 16
alpha: 32
target_modules: [q_proj, v_proj, k_proj, o_proj]
learning_rate: 2e-4
batch_size: 4
epochs: 3
```

**Custo Estimado**: $10-25 (GPU cloud)

#### Avaliação (Dias 6-7)
- [ ] Benchmark vs modelo base
- [ ] Validar qualidade das decisões
- [ ] Testar em cenários reais
- [ ] A/B testing

#### Deploy (Dias 8-10)
- [ ] Quantizar modelo (4-bit, ~2GB)
- [ ] Integrar com MVP Node
- [ ] Documentar processo
- [ ] Monitorar qualidade

**Entregáveis**:
- ✅ Modelo fine-tuned especializado
- ✅ Modelo quantizado para produção
- ✅ Métricas de melhoria vs base
- ✅ Documentação de fine-tuning

---

### Fase 4: Inference Pool - Layer 2 (4-6 semanas)
**Objetivo**: API Gateway descentralizado com billing e orquestração LLM

#### Sprint 1: Core Infrastructure (Semana 1)
- [ ] Task 1: Setup do projeto e infraestrutura base
  - [ ] 1.1: Criar estrutura do projeto Rust
  - [ ] 1.2: Implementar tipos de erro e configuração
  - [ ] 1.3: Property test para error types

- [ ] Task 2: Implementar Pool Registry
  - [ ] 2.1: Criar estruturas de dados
  - [ ] 2.2: Implementar PoolRegistry trait
  - [ ] 2.3: Property test para registry round-trip

- [ ] Task 3: Implementar Heartbeat Monitor
  - [ ] 3.1: Criar HeartbeatMonitor
  - [ ] 3.2: Integrar com Pool Registry
  - [ ] 3.3: Property test para heartbeat timeout

- [ ] Task 4: Checkpoint - Registry e Heartbeat

#### Sprint 2: OpenAI API (Semanas 2-3)
- [ ] Task 5: Implementar Data Models OpenAI-Compatible
- [ ] Task 6: Implementar Authentication Middleware
- [ ] Task 7: Implementar Job Router
- [ ] Task 8: Checkpoint - Core Components
- [ ] Task 9: Implementar HTTP API Server
- [ ] Task 10: Implementar Embeddings API
- [ ] Task 11: Implementar Models API
- [ ] Task 12: Implementar Pool Status Endpoints
- [ ] Task 13: Checkpoint - API Completa

#### Sprint 3: P2P e Orchestrator (Semana 4)
- [ ] Task 14: Implementar Network Discovery
  - [ ] 14.1: Setup libp2p para a pool
  - [ ] 14.2: Implementar descoberta de nodes
  - [ ] 14.3: Implementar convite de nodes
  - [ ] 14.4: Implementar descoberta de outras pools
  - [ ] 14.5-14.6: Property tests

- [ ] Task 15: Implementar Pool Orchestrator (LLM)
  - [ ] 15.1: Criar estruturas do Orchestrator
  - [ ] 15.2: Integrar com LLM local
  - [ ] 15.3: Implementar PoolOrchestrator trait
  - [ ] 15.4: Implementar histórico de decisões
  - [ ] 15.5-15.6: Property tests

- [ ] Task 16: Checkpoint - Discovery e Orchestrator

#### Sprint 4: Billing e Integration (Semanas 5-6)
- [ ] Task 17: Implementar Billing System
  - [ ] 17.1: Criar estruturas de billing
  - [ ] 17.2: Implementar criação de tickets
  - [ ] 17.3: Implementar endpoints de billing
  - [ ] 17.4: Implementar geração de invoices
  - [ ] 17.5: Implementar suporte Lightning Network
  - [ ] 17.6-17.8: Property tests

- [ ] Task 18: Implementar Usage Tracker
- [ ] Task 19: Checkpoint - Billing Completo
- [ ] Task 20: Integração e Wiring
- [ ] Task 21: Checkpoint Final

**Entregáveis**:
- ✅ API compatível com OpenAI (chat, embeddings, models)
- ✅ Descoberta automática de nodes via P2P
- ✅ Orquestração inteligente com LLM
- ✅ Sistema de billing com Lightning Network
- ✅ 23 property tests passando
- ✅ Documentação completa

---

## 📊 Especificações Completas

### Estrutura de Specs

Todas as especificações seguem a metodologia spec-driven development:
1. **Requirements** (User Stories + Acceptance Criteria)
2. **Design** (Architecture + Correctness Properties)
3. **Tasks** (Implementation Plan)

```
.kiro/specs/
├── mvp-cli/
│   ├── requirements.md      # 10 user stories, 50 critérios
│   ├── design.md            # 50 propriedades de corretude
│   ├── tasks.md             # 19 tarefas, 100+ subtarefas
│   └── implementation-review.md  # Análise de implementação
│
├── llm-orchestrator/
│   ├── requirements.md      # 10 user stories, 50 critérios
│   ├── design.md            # 44 propriedades de corretude
│   ├── tasks.md             # 18 tarefas, 70+ subtarefas
│   └── README.md            # Overview e guia rápido
│
└── api-integration/         # Inference Pool (Layer 2)
    ├── requirements.md      # 12 user stories, 40+ critérios
    ├── design.md            # 23 propriedades de corretude
    └── tasks.md             # 21 tarefas, 60+ subtarefas
```

---

### Spec 1: MVP CLI

**Localização**: `.kiro/specs/mvp-cli/`

#### Requirements (10 User Stories)
1. **Job Management**: Submit, list, cancel, view results
2. **Node Monitoring**: List, info, health, metrics
3. **Network Explorer**: Topology, stats, peer discovery
4. **Multi-node Testing**: Local testing, bootstrap, load tests
5. **Interactive REPL**: Interactive mode, auto-complete
6. **Script Automation**: Batch operations, scripting
7. **Data Export**: Export metrics, logs, results
8. **Configuration**: Profiles, defaults, customization
9. **Error Handling**: Clear messages, recovery suggestions
10. **Performance**: Fast responses, efficient queries

#### Design (50 Correctness Properties)
- Job submission and tracking (10 properties)
- Node discovery and monitoring (10 properties)
- Network topology visualization (8 properties)
- Multi-node orchestration (8 properties)
- REPL functionality (6 properties)
- Script execution (4 properties)
- Data export (4 properties)

#### Implementation Status
- ✅ 80% implementado
- ✅ 20 property tests passando
- 🚧 Gaps: Network commands integration, end-to-end testing

---

### Spec 2: LLM Orchestrator

**Localização**: `.kiro/specs/llm-orchestrator/`

#### Requirements (10 User Stories)
1. **Decision Making**: Analyze state, suggest decisions
2. **Local LLM**: No external APIs, autonomous operation
3. **Training Data**: Collect decisions and outcomes
4. **Validation**: Validate all decisions before execution
5. **Decision Types**: Infrastructure, Context, Mediation
6. **Monitoring**: Metrics, performance, quality tracking
7. **Natural Language**: Interpret commands, provide feedback
8. **Model Management**: Multiple models, switching, fallback
9. **Caching**: Fast responses for similar queries
10. **Integration**: Optional module in MVP Node

#### Design (44 Correctness Properties)

**Decision Generation and Validation** (5 properties)
- Property 1: Decision format compliance
- Property 2: Decision validation before return
- Property 3: Local model operation
- Property 4: Model integrity verification
- Property 5: Offline operation continuity

**Data Collection and Training** (4 properties)
- Property 6: Decision recording completeness
- Property 7: Outcome association
- Property 8: Feedback integration
- Property 9: Dataset export quality

**Validation and Safety** (5 properties)
- Property 10: JSON format validation
- Property 11: Action allowlist enforcement
- Property 12: Parameter bounds checking
- Property 13: Invalid decision rejection
- Property 14: Fallback activation

**Specialized Decision Types** (5 properties)
- Property 15: Infrastructure decision configuration
- Property 16: Context decision configuration
- Property 17: Mediation decision configuration
- Property 18: Parameter adjustment by type
- Property 19: Decision type routing

**Performance and Monitoring** (5 properties)
- Property 20: Inference latency recording
- Property 21: Validation success rate tracking
- Property 22: Decision success rate calculation
- Property 23: Metrics interface exposure
- Property 24: Problem detection alerting

**Natural Language Interface** (5 properties)
- Property 25: Intent interpretation
- Property 26: Command mapping accuracy
- Property 27: Destructive action confirmation
- Property 28: Confirmed command execution
- Property 29: Natural language feedback

**Model Management** (5 properties)
- Property 30: Multi-model support
- Property 31: Optimized model configuration
- Property 32: Fine-tuned model preference
- Property 33: Resource-based model selection
- Property 34: Model failure fallback

**Caching and Performance** (5 properties)
- Property 35: Decision caching
- Property 36: Cache hit detection
- Property 37: Cache response latency (<10ms)
- Property 38: Cache invalidation on state change
- Property 39: LRU cache management

**Integration with MVP Node** (5 properties)
- Property 40: Optional module loading
- Property 41: Health monitor integration
- Property 42: Logging system integration
- Property 43: CLI endpoint compatibility
- Property 44: Graceful disable

#### Implementation Status
- 🚧 Em planejamento
- ✅ Spec completa e aprovada
- 🚧 Task 1 iniciada (core infrastructure)

---

### Spec 3: Inference Pool (API Integration)

**Localização**: `.kiro/specs/api-integration/`

#### Requirements (12 User Stories)
1. **Pool Membership**: Registro e monitoramento de nodes
2. **Chat Completions API**: Compatibilidade OpenAI com streaming
3. **Embeddings API**: Geração de vetores semânticos
4. **Models Discovery**: Lista agregada de modelos disponíveis
5. **Job Routing**: Distribuição inteligente entre Pool_Members
6. **Pool Authentication**: API Keys e rate limiting
7. **Usage Tracking**: Rastreamento de consumo por cliente
8. **Pool Status**: Transparência e monitoramento público
9. **Error Handling**: Formato de erro OpenAI
10. **P2P Network Discovery**: Descoberta automática de nodes
11. **LLM-Powered Orchestration**: Decisões inteligentes de roteamento
12. **Billing System**: Cobrança de clientes e pagamento de nodes

#### Design (23 Correctness Properties)

**Pool Registry e Membership** (2 properties)
- Property 1: Node Registration Round-Trip
- Property 2: Heartbeat Timeout Marks Inactive

**OpenAI API Compatibility** (6 properties)
- Property 3: OpenAI Request Format Acceptance
- Property 4: Message Roles Parsing
- Property 5: Streaming Mode Determines Response Format
- Property 6: Response Contains Required OpenAI Fields
- Property 7: Embedding Input Flexibility
- Property 8: Embedding Order Preservation

**Job Routing** (4 properties)
- Property 9: Models Aggregation from Members
- Property 10: Router Selects Model-Compatible Member
- Property 11: Router Considers Load and Latency
- Property 12: Router Retries on Failure

**Authentication e Usage** (2 properties)
- Property 13: API Key Validation Determines Access
- Property 14: Inference Completion Triggers Tracking

**Error Handling** (2 properties)
- Property 15: Error Responses Follow OpenAI Format
- Property 16: Invalid Payload Returns 400

**Network Discovery** (2 properties)
- Property 17: Network Discovery Connects to P2P
- Property 18: Discovered Nodes Receive Invitations

**LLM Orchestrator** (2 properties)
- Property 19: LLM Orchestrator Provides Routing Decisions
- Property 20: Orchestrator Records Decision History

**Billing System** (3 properties)
- Property 21: Billing Ticket Creation
- Property 22: Invoice Generation
- Property 23: Multi-Currency Support

#### Implementation Status
- 📋 Especificação completa
- 📋 Aguardando início de implementação
- 📋 21 tarefas planejadas

---

## 🎯 Métricas de Sucesso

### MVP Node
- ✅ Uptime > 99%
- ✅ Job completion rate > 95%
- ✅ Average latency < 2s
- ✅ Network connectivity > 90%

### MVP CLI
- ✅ Command response time < 100ms
- ✅ Test coverage > 80%
- ✅ Property tests passing: 20/20
- 🚧 End-to-end tests: 0/10

### LLM Orchestrator (Targets)
- 🎯 Inference latency < 500ms
- 🎯 Validation success rate > 95%
- 🎯 Decision success rate > 80%
- 🎯 Cache hit rate > 60%
- 🎯 Training examples collected: 1000+
- 🎯 Human feedback ratio > 20%
- 🎯 Safety violations: 0

### Inference Pool (Targets)
- 🎯 API response time < 200ms (excluding inference)
- 🎯 OpenAI compatibility: 100%
- 🎯 Pool member uptime > 99%
- 🎯 Routing success rate > 99%
- 🎯 Billing accuracy: 100%
- 🎯 Property tests passing: 23/23
- 🎯 Lightning payment success > 95%

---

## 🔧 Configuração do Sistema

### MVP Node Config
```toml
[node]
node_id = "node-1"
listen_address = "/ip4/0.0.0.0/tcp/4001"
bootstrap_peers = []

[inference]
model_path = "./models/tinyllama"
max_batch_size = 4
max_queue_size = 100

[network]
max_peers = 50
gossip_interval_ms = 1000
heartbeat_interval_ms = 5000

[monitoring]
metrics_enabled = true
log_level = "info"
health_check_interval_ms = 10000
```

### Orchestrator Config
```toml
[orchestrator]
enabled = true
model_path = "./models/llama3-2-3b-instruct"
model_type = "llama3_2_3b"
device = "auto"  # auto, cpu, cuda, metal

[orchestrator.generation]
max_tokens = 512
temperature = 0.7
top_p = 0.9
repetition_penalty = 1.1

[orchestrator.cache]
enabled = true
max_entries = 1000
ttl_seconds = 300

[orchestrator.training]
collect_data = true
export_path = "./training_data"
auto_export_threshold = 1000

[orchestrator.safety]
allowed_actions = [
    "start_election",
    "migrate_context",
    "adjust_gossip",
    "scale_queue",
    "wait",
    "restart_component"
]
max_confidence_threshold = 0.95
require_confirmation_above = 0.8
```

---

## 📚 Documentação Adicional

### Arquitetura
- `docs/distributed-models.md` - Modelos distribuídos
- `llm-orquestrationtx` - Conceito de orquestração com LLM
- `whitepaper.md` - Whitepaper do projeto

### Specs Detalhadas
- `.kiro/specs/mvp-cli/requirements.md` - Requirements do CLI
- `.kiro/specs/mvp-cli/design.md` - Design do CLI
- `.kiro/specs/mvp-cli/tasks.md` - Tasks do CLI
- `.kiro/specs/llm-orchestrator/requirements.md` - Requirements do Orchestrator
- `.kiro/specs/llm-orchestrator/design.md` - Design do Orchestrator
- `.kiro/specs/llm-orchestrator/tasks.md` - Tasks do Orchestrator
- `.kiro/specs/llm-orchestrator/README.md` - Overview do Orchestrator
- `.kiro/specs/api-integration/requirements.md` - Requirements da Inference Pool
- `.kiro/specs/api-integration/design.md` - Design da Inference Pool
- `.kiro/specs/api-integration/tasks.md` - Tasks da Inference Pool

### Código
- `mvp-node/` - Implementação do MVP Node
- `mvp-cli/` - Implementação do MVP CLI
- `mvp-node/src/orchestration/` - Implementação do Orchestrator (em desenvolvimento)

---

## 🚀 Como Começar

### 1. Setup do Ambiente

```bash
# Clone o repositório
git clone <repo-url>
cd ia-p2p

# Instalar Rust (se necessário)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build do projeto
cd mvp-node
cargo build --release

cd ../mvp-cli
cargo build --release
```

### 2. Executar MVP Node

```bash
# Node 1 (bootstrap)
./target/release/mvp-node --config config.toml

# Node 2 (conecta ao bootstrap)
./target/release/mvp-node --config config2.toml
```

### 3. Usar MVP CLI

```bash
# Listar nodes
./target/release/mvp-cli node list

# Submit job
./target/release/mvp-cli job submit --prompt "Hello" --model tinyllama

# Ver métricas
./target/release/mvp-cli node metrics node-1 --watch
```

### 4. Desenvolver LLM Orchestrator

```bash
# Seguir tasks em .kiro/specs/llm-orchestrator/tasks.md
cd mvp-node

# Task 1: Setup infrastructure (✅ Completo)
# Task 2: Implement Local LLM Backend (🚧 Próximo)
# ...
```

---

## 🤝 Contribuindo

### Workflow de Desenvolvimento
1. Escolher uma task da spec
2. Implementar seguindo o design
3. Escrever property tests
4. Executar testes
5. Documentar mudanças
6. Commit e push

### Padrões de Código
- Rust idiomático
- Documentação inline
- Property-based testing
- Error handling robusto
- Logging estruturado

### Testing
```bash
# Unit tests
cargo test

# Property tests
cargo test --features proptest

# Integration tests
cargo test --test integration

# Benchmarks
cargo bench
```

---

## 📞 Contato e Suporte

- **Issues**: GitHub Issues
- **Discussões**: GitHub Discussions
- **Documentação**: `docs/`
- **Specs**: `.kiro/specs/`

---

**Última Atualização**: Dezembro 2024  
**Versão**: 0.1.0  
**Status**: Em Desenvolvimento Ativo 🚀
