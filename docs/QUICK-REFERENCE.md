# ⚡ Guia Rápido de Referência

> **Acesso rápido a informações essenciais do projeto**

---

## 🎯 Links Rápidos

| O que você precisa | Onde encontrar |
|-------------------|----------------|
| **Status do projeto** | [PROJECT-STATUS.md](PROJECT-STATUS.md) |
| **Roadmap completo** | [ROADMAP.md](ROADMAP.md) |
| **Índice de docs** | [INDEX.md](INDEX.md) |
| **Specs do CLI** | [.kiro/specs/mvp-cli/](../.kiro/specs/mvp-cli/) |
| **Specs do Orchestrator** | [.kiro/specs/llm-orchestrator/](../.kiro/specs/llm-orchestrator/) |
| **Código MVP Node** | [mvp-node/](../mvp-node/) |
| **Código MVP CLI** | [mvp-cli/](../mvp-cli/) |

---

## 📊 Status Atual (Dezembro 2024)

```
MVP Node:          ████████████████░░  80% ✅
MVP CLI:           ████████████████░░  80% ✅
LLM Orchestrator:  ██░░░░░░░░░░░░░░  10% 🚧
Fine-tuning:       ░░░░░░░░░░░░░░░░   0% 📋
```

---

## 🏗️ Arquitetura em 3 Camadas

```
┌─────────────────────────────────────┐
│  MVP CLI (Interface)                │  ← Comandos do usuário
├─────────────────────────────────────┤
│  LLM Orchestrator (Cognição)        │  ← LLM decide O QUE
├─────────────────────────────────────┤
│  MVP Node (Infraestrutura)          │  ← Infra decide COMO
└─────────────────────────────────────┘
```

---

## 🚀 Comandos Essenciais

### Build e Execução

```bash
# Build MVP Node
cd mvp-node && cargo build --release

# Build MVP CLI
cd mvp-cli && cargo build --release

# Executar Node
./mvp-node/target/release/mvp-node --config config.toml

# Executar CLI
./mvp-cli/target/release/mvp-cli node list
```

### Testes

```bash
# Unit tests
cargo test

# Property tests
cargo test --features proptest

# Testes específicos
cargo test test_name

# Com output
cargo test -- --nocapture
```

---

## 📋 Specs Disponíveis

### MVP CLI
- **Requirements**: 10 user stories, 50 critérios
- **Design**: 50 propriedades de corretude
- **Tasks**: 19 tarefas, 100+ subtarefas
- **Status**: ✅ 80% implementado

### LLM Orchestrator
- **Requirements**: 10 user stories, 50 critérios
- **Design**: 44 propriedades de corretude
- **Tasks**: 18 tarefas, 70+ subtarefas
- **Status**: 🚧 10% implementado

---

## 🎯 Próximas Tasks (Prioridade)

### Esta Semana
1. ✅ Task 1: Setup core infrastructure (50%)
2. 📋 Task 2: Local LLM Backend
3. 📋 Task 3: Metrics Aggregator

### Próximas 2 Semanas
1. Task 4: Decision Validator
2. Task 5: Training Data Collector
3. Task 6: Core Orchestrator

---

## 🔧 Configuração Rápida

### MVP Node (config.toml)
```toml
[node]
node_id = "node-1"
listen_address = "/ip4/0.0.0.0/tcp/4001"

[inference]
model_path = "./models/tinyllama"
max_batch_size = 4

[network]
max_peers = 50
```

### Orchestrator (config.toml)
```toml
[orchestrator]
enabled = true
model_path = "./models/llama3-2-3b-instruct"
model_type = "llama3_2_3b"
device = "auto"

[orchestrator.generation]
max_tokens = 512
temperature = 0.7
```

---

## 📊 Métricas Chave

### Targets
- Inference latency: < 500ms
- Validation success: > 95%
- Decision success: > 80%
- Cache hit rate: > 60%

### Atuais (MVP Node)
- Uptime: ~98%
- Job completion: ~92%
- Avg latency: ~1.8s
- Network connectivity: ~95%

---

## 🧩 Componentes do LLM Orchestrator

| Componente | Responsabilidade | Status |
|------------|------------------|--------|
| **Local LLM Backend** | Gerenciar modelos locais | 📋 |
| **Metrics Aggregator** | Coletar estado do sistema | 📋 |
| **Decision Validator** | Validar decisões | 📋 |
| **Training Collector** | Coletar dados de treino | 📋 |
| **Natural Language** | Interpretar comandos | 📋 |
| **Decision Cache** | Cache inteligente | 📋 |

---

## 🎨 Tipos de Decisão

### 🏗️ Infrastructure (Conservador)
- **Temperatura**: 0.3
- **Foco**: Estabilidade
- **Uso**: Sobrecarga, falhas, migração

### 🎭 Context (Moderado)
- **Temperatura**: 0.7
- **Foco**: Semântica
- **Uso**: Interpretação, correlação

### ⚖️ Mediation (Criativo)
- **Temperatura**: 0.9
- **Foco**: Resolução de conflitos
- **Uso**: Conflitos, negociação

---

## 📚 Documentação por Tipo

### Para Começar
1. [PROJECT-STATUS.md](PROJECT-STATUS.md) - Status executivo
2. [ROADMAP.md](ROADMAP.md) - Roadmap completo
3. [INDEX.md](INDEX.md) - Índice de navegação

### Para Implementar
1. Spec: requirements.md - O que fazer
2. Spec: design.md - Como fazer
3. Spec: tasks.md - Passo a passo

### Para Entender Conceitos
1. [llm-orquestrationtx](../llm-orquestrationtx) - Orquestração
2. [distributed-models.md](distributed-models.md) - Arquitetura
3. [whitepaper.md](../whitepaper.md) - Visão geral

---

## 🔍 Busca Rápida de Código

### MVP Node
```
mvp-node/src/
├── lib.rs              # Core traits
├── main.rs             # Entry point
├── monitoring/         # Health & metrics
└── orchestration/      # LLM orchestrator (novo)
```

### MVP CLI
```
mvp-cli/src/
├── main.rs             # Entry point
├── commands/           # Command handlers
│   ├── job.rs
│   ├── node.rs
│   └── network.rs
├── config.rs           # Configuration
└── output.rs           # Output formatting
```

### Orchestrator
```
mvp-node/src/orchestration/
├── mod.rs              # Module root
├── orchestrator.rs     # Core orchestrator
├── llm_backend.rs      # LLM management
├── metrics_aggregator.rs
├── decision_validator.rs
├── training_collector.rs
├── natural_language.rs
├── cache.rs
├── prompts.rs
├── types.rs
└── error.rs
```

---

## 🧪 Testing Cheat Sheet

### Property Tests
```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn property_name(input in strategy()) {
            // Test property
        }
    }
}
```

### Unit Tests
```rust
#[tokio::test]
async fn test_name() {
    // Setup
    // Execute
    // Assert
}
```

---

## 💡 Dicas Rápidas

### Desenvolvimento
- Sempre ler spec antes de implementar
- Escrever property tests junto com código
- Documentar decisões de design
- Fazer commits pequenos e frequentes

### Debugging
- Usar `RUST_LOG=debug` para logs detalhados
- Property tests ajudam a encontrar edge cases
- Verificar métricas no health monitor

### Performance
- Usar cache quando possível
- Quantizar modelos para produção
- Monitorar latência de inferência
- Otimizar apenas após medir

---

## 🚨 Troubleshooting Comum

### Problema: Modelo não carrega
```bash
# Verificar path do modelo
ls -la ./models/

# Verificar logs
RUST_LOG=debug ./mvp-node
```

### Problema: Testes falhando
```bash
# Executar teste específico
cargo test test_name -- --nocapture

# Limpar e rebuildar
cargo clean && cargo build
```

### Problema: Latência alta
```bash
# Verificar métricas
./mvp-cli node metrics node-1

# Verificar cache
# Ver configuração de cache em config.toml
```

---

## 📞 Onde Pedir Ajuda

| Tipo de Dúvida | Onde Ir |
|----------------|---------|
| **Conceitos** | [llm-orquestrationtx](../llm-orquestrationtx), [whitepaper.md](../whitepaper.md) |
| **Arquitetura** | [distributed-models.md](distributed-models.md), specs/design.md |
| **Implementação** | specs/tasks.md, código existente |
| **Bugs** | GitHub Issues |
| **Discussões** | GitHub Discussions |

---

## 🎯 Checklist de Implementação

### Antes de Começar
- [ ] Ler spec completa (requirements + design)
- [ ] Entender task específica
- [ ] Verificar dependências
- [ ] Setup ambiente de desenvolvimento

### Durante Implementação
- [ ] Seguir design da spec
- [ ] Escrever código limpo e documentado
- [ ] Implementar property tests
- [ ] Testar localmente
- [ ] Verificar métricas

### Antes de Commit
- [ ] Todos os testes passando
- [ ] Código formatado (`cargo fmt`)
- [ ] Sem warnings (`cargo clippy`)
- [ ] Documentação atualizada
- [ ] Commit message descritivo

---

## 📈 Métricas de Qualidade

### Código
- Test coverage: > 80%
- Property tests: Todos passando
- Clippy warnings: 0
- Documentation: Completa

### Performance
- Build time: < 5 min
- Test time: < 2 min
- Inference latency: < 500ms
- Memory usage: < 8GB

---

## 🔗 Links Externos Úteis

### Rust
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Docs](https://tokio.rs/)

### LLM e ML
- [Candle Docs](https://github.com/huggingface/candle)
- [HuggingFace Hub](https://huggingface.co/)
- [Llama Models](https://ai.meta.com/llama/)

### P2P
- [libp2p Docs](https://docs.libp2p.io/)
- [libp2p Rust](https://github.com/libp2p/rust-libp2p)

---

**Última Atualização**: Dezembro 2024  
**Versão**: 1.0  
**Mantido por**: Equipe de Desenvolvimento
