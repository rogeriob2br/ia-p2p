# LLM-Mediated Consensus para Rede P2P de IA

## Visão Geral

Proposta de arquitetura onde **LLMs dos nodes conversam entre si** para:
1. Eleger um orquestrador (líder)
2. Rotear jobs para o node mais adequado

**Diferencial:** Não usa algoritmos de consenso tradicionais (Raft, PBFT). Os LLMs negociam via troca de mensagens em linguagem natural/estruturada.

---

## Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   ┌──────────┐        ┌──────────┐        ┌──────────┐         │
│   │  Node A  │◀──────▶│  Node B  │◀──────▶│  Node C  │         │
│   │  ┌────┐  │        │  ┌────┐  │        │  ┌────┐  │         │
│   │  │LLM │  │        │  │LLM │  │        │  │LLM │  │         │
│   │  └────┘  │        │  └────┘  │        │  └────┘  │         │
│   └──────────┘        └──────────┘        └──────────┘         │
│        │                   │                   │                │
│        └───────────────────┼───────────────────┘                │
│                            │                                    │
│                    Diálogo P2P                                  │
│              (argumentação + votação)                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Fluxo 1: Eleição de Orquestrador

```
Node A: "Proponho ser orquestrador. GPU RTX 4090, 8 peers conectados."

Node B: "Discordo. Você está com 78% de carga. Eu tenho 23% e 
         Mistral-7B carregado para decisões rápidas."

Node C: "Apoio Node B. Menor carga = respostas mais rápidas."

CONSENSO → Node B eleito orquestrador
```

**Critérios considerados pelos LLMs:**
- Carga atual do node
- Hardware disponível (GPU, memória)
- Modelo carregado
- Uptime e estabilidade
- Número de peers conectados

---

## Fluxo 2: Roteamento de Job

```
CLI envia: "Traduza documento de 50 páginas EN→PT"
                    │
                    ▼
         ┌──────────────────┐
         │  Orquestrador    │
         │  (Node B)        │
         └────────┬─────────┘
                  │
      ┌───────────┼───────────┐
      ▼           ▼           ▼
   Node A      Node B      Node C
   "Llama-7B   "Ocupado    "NLLB-200
    45s/pág"   routing"    8s/pág"
      │           │           │
      └───────────┼───────────┘
                  ▼
         Decisão: Node C
         Razão: "Modelo especializado, 5x mais rápido"
```

---

## Protocolo de Mensagens

```rust
enum LLMMessage {
    // Eleição
    ProposeLeader { candidate, reasoning },
    RespondToProposal { agree, counter_argument },
    
    // Estado
    StateAnnouncement { capabilities, load, specializations },
    
    // Roteamento
    RouteJobRequest { job_context, requirements },
    RouteJobProposal { suggested_node, reasoning, confidence },
    
    // Finalização
    ConsensusReached { decision, participants, reasoning_summary },
}
```

---

## Diferenças de Consenso Tradicional

| Aspecto | Raft/PBFT | LLM Consensus |
|---------|-----------|---------------|
| Votação | Binária (sim/não) | Argumentação contextual |
| Decisão | Determinística | Semântica + Heurística |
| Critérios | Timeout, termo | Capacidades, especialização |
| Explicabilidade | Log de votos | Reasoning em linguagem natural |
| Adaptabilidade | Regras fixas | Aprende com contexto |

---

## Questões em Aberto

1. **Rodadas de diálogo** - Quantas trocas antes de decidir?
2. **Timeout** - E se um node não responder?
3. **Critério de consenso** - Maioria simples? Unanimidade? Threshold de confiança?
4. **Fallback** - O que fazer sem consenso? Heurística determinística?
5. **Segurança** - Como evitar node malicioso manipulando argumentos?
6. **Custo computacional** - Cada rodada = inferência LLM. Otimizar?

---

## Stack Técnica Existente

- **Rede P2P:** libp2p (Rust)
- **Protocolos:** Request-Response, GossipSub, mDNS
- **LLM Local:** Candle (Rust) - suporta Llama, Mistral, Phi
- **Comunicação:** JSON sobre TCP/QUIC

---

## Próximos Passos (se aprovado)

1. Definir protocolo de mensagens LLM-to-LLM
2. Implementar prompts de negociação
3. Criar lógica de rodadas e timeout
4. Integrar com sistema de routing existente
5. Testes com múltiplos nodes

---

## Referências

- Multi-Agent LLM Systems
- Debate-based AI alignment
- Constitutional AI (Anthropic)
- AutoGen (Microsoft)
- CrewAI


