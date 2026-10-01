# IA-P2P: Decentralized AI Network

Rede descentralizada de IA com orquestração distribuída inspirada em Swarm Intelligence.

## Visão Geral

Este projeto implementa uma rede P2P para execução distribuída de modelos de IA, com:

- **Nós de execução** que hospedam micromodelos (≤7B parâmetros)
- **Comunicação P2P** via libp2p (gossipsub, mDNS, request-response)
- **Orquestração distribuída** com consenso adaptativo
- **CLI interativa** para gerenciamento da rede

## Componentes

```
ia-p2p/
├── mvp-node/       # Nó de execução e orquestração
├── mvp-cli/        # Interface de linha de comando
├── docs/           # Documentação detalhada
└── .kiro/specs/    # Especificações técnicas
```

### MVP Node

Nó completo da rede com capacidades de:

- Execução de jobs de inferência
- Bootstrap de rede
- Orquestração local (LLM)
- Orquestração distribuída (Swarm Intelligence)
- Health monitoring e métricas

[Documentação completa →](mvp-node/README.md)

### MVP CLI

Interface de linha de comando para:

- Descoberta de nós via mDNS
- Submissão e gerenciamento de jobs
- Consulta de status e métricas
- Modo REPL interativo

[Documentação completa →](mvp-cli/README.md)

## Quick Start

### Requisitos

- Rust 1.75+
- Linux, macOS ou Windows (WSL2)

### Instalação

```bash
# Clone o repositório
git clone <repository>
cd ia-p2p

# Build de todos os componentes
cargo build --release
```

### Iniciando um Nó

```bash
# Nó bootstrap (primeiro nó da rede)
./target/release/mvp-node --port 9000

# Nó adicional conectando ao bootstrap
./target/release/mvp-node --port 9001 --bootstrap /ip4/127.0.0.1/tcp/9000
```

### Usando a CLI

```bash
# Descobrir nós na rede local
./target/release/mvp-cli node list

# Submeter um job
./target/release/mvp-cli job submit --prompt "Hello, world!"

# Modo interativo
./target/release/mvp-cli repl
```

## Arquitetura

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              MVP CLI                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │   Node      │  │    Job      │  │   Config    │  │      REPL       │ │
│  │  Commands   │  │  Commands   │  │  Commands   │  │    Interface    │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
│         └────────────────┴────────────────┴──────────────────┘          │
│                                   │                                      │
│                          NodeClient (P2P)                                │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │ libp2p (mDNS, Request-Response)
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              MVP NODE                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────┐  ┌─────────────────────────────────────────┐   │
│  │   Network Layer     │  │       Distributed Orchestration         │   │
│  │  ┌───────────────┐  │  │  ┌─────────────┐  ┌─────────────────┐   │   │
│  │  │   Gossipsub   │  │  │  │  Affinity   │  │    Adaptive     │   │   │
│  │  │     mDNS      │◄─┼──┼─▶│ Calculator  │  │    Consensus    │   │   │
│  │  │   Identify    │  │  │  └─────────────┘  └─────────────────┘   │   │
│  │  └───────────────┘  │  │  ┌─────────────┐  ┌─────────────────┐   │   │
│  └─────────────────────┘  │  │  Decision   │  │    Context      │   │   │
│  ┌─────────────────────┐  │  │   Router    │  │     Groups      │   │   │
│  │   Job Executor      │  │  └─────────────┘  └─────────────────┘   │   │
│  │  ┌───────────────┐  │  │  ┌─────────────┐  ┌─────────────────┐   │   │
│  │  │  Job Queue    │  │  │  │  Partition  │  │   P2P Adapter   │   │   │
│  │  │  Scheduler    │  │  │  │   Handler   │  │  Network Bridge │   │   │
│  │  └───────────────┘  │  │  └─────────────┘  └─────────────────┘   │   │
│  └─────────────────────┘  └─────────────────────────────────────────┘   │
│  ┌─────────────────────┐  ┌─────────────────────────────────────────┐   │
│  │  LLM Orchestrator   │  │          Health Monitor                 │   │
│  │  (Local Decisions)  │  │         (Metrics & Status)              │   │
│  └─────────────────────┘  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Orquestração Distribuída

O sistema implementa coordenação multi-nó inspirada em Swarm Intelligence:

### Características

- **Coordenação Emergente**: Coordenadores surgem baseados em afinidade
- **Consenso Adaptativo**: Response Threshold Model (bio-inspirado)
- **Decision Routing**: Seleção automática de estratégia (heurística/LLM/híbrido)
- **Context Groups**: Grupos dinâmicos de especialização
- **Partition Handling**: Detecção e reconciliação automática

[Documentação detalhada →](docs/DISTRIBUTED-ORCHESTRATION.md)

## Testes

### Cobertura Total

| Componente | Testes |
|------------|--------|
| mvp-node (unit) | 250 |
| mvp-node (integration) | 47 |
| mvp-node (property) | 235 |
| mvp-cli | 63 |
| **Total** | **595+** |

### Executando Testes

```bash
# Todos os testes
cargo test --workspace

# Apenas mvp-node
cd mvp-node && cargo test

# Apenas mvp-cli
cd mvp-cli && cargo test

# Testes de orquestração distribuída
cd mvp-node && cargo test distributed_orchestration
```

## Documentação

- [MVP Node README](mvp-node/README.md)
- [MVP CLI README](mvp-cli/README.md)
- [Distributed Orchestration](docs/DISTRIBUTED-ORCHESTRATION.md)
- [CLI P2P Integration](docs/CLI-P2P-INTEGRATION-VALIDATION-UPDATED.md)

## Especificações

- [MVP Node Specs](.kiro/specs/mvp-node/)
- [Distributed Orchestration Specs](.kiro/specs/distributed-orchestration/)

## Licença

MIT OR Apache-2.0

