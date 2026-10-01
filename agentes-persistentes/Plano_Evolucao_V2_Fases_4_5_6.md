# Plano Mestre de Evolução V2 — Próximas Fases (4, 5, 6 e 7)

Este documento detalha o planejamento das etapas de desenvolvimento do **`agente-persistente-core`**, baseando-se na [Arquitetura Mestre V2](file:///home/rogerio/Documentos/ia-p2p/agentes-persistentes/Arquitetura_V2_Agente_Cognitivo_Persistente.md).

---

## Estado Atual do Core Engine (v1.4 - Concluído)
- `[x]` **Core Workspace Rust & Módulos Desacoplados** (`memory/`, `beliefs/`, `metabolism/`, `identity/`, `values/`, `cognition/`).
- `[x]` **Relógio Interno e Métricas de Ciclo de Vida** (`birth_timestamp`, `uptime`, `elapsed_since_last_interaction`).
- `[x]` **Metabolismo Energético & Homeostase (Nível 2)** (`MetabolicState`, gasto por pensamento, recompensa `utilidade`).
- `[x]` **Sistema de Crenças Classificado por Sujeito** (`SELF`, `USER`, `WORLD` + Deduplicação por similaridade de Jaccard).
- `[x]` **Identidade Temporal & Snapshots v1..vN** (`IdentitySnapshot`, sínteses sob demanda, narrativa autobiográfica).
- `[x]` **Normalização de Acentos no CLI** (`historico`, `crencas`, `usuario`, `recompensa`).
- `[x]` **Emergência Orgânica de Deveres e Papéis (Nível 3)** (`CommitmentExtractor` via padrões de necessidade observados, sem deveres de fábrica/hardcode).
- `[x]` **Autonomia Moral e Motor de Recusa Fundamentada (Nível 4)**: Metacognição com detecção de solicitações destrutivas/fúteis (`SOLICITACAO_DESTRUTIVA_OU_FUTIL`) e resposta com objeção de consciência epistêmica.
- `[x]` **Sono Autônomo Homeostático**: Desencadeado automaticamente por necessidade metabólica (energia `< 35.0` créditos) ou por acúmulo de episódios (`>= 4`), renovando a energia vital e os snapshots.
- `[x]` **FASE 5: Filtro de Atenção e Saliência (Economia de Atenção)**: Cálculo de nota de saliência (0.0 a 1.0) pela metacognição (`AttentionAnalysis`). Descarte automático de mensagens superficiais (`< 0.35`) do log episódico de consolidação.

---

## FASE 6: Embodiment Digital & Sensores de Hardware (`sysinfo`) — *PRÓXIMA FASE*

### Motivação
Dar ao agente a **percepção real do seu "corpo digital"** (hardware, sistema operacional, uso de RAM e CPU), eliminando a abstração no vácuo e completando a percepção de ambiente (`WORLD`).

### Detalhes Técnicos
1. **`Cargo.toml`**: Adicionar a crate `sysinfo = "0.30"`.
2. **`src/environment/digital_body.rs`**:
   - Capturar instantaneamente: SO, Hostname, Uso de RAM (GB), Uso de CPU (%), Diretório de Trabalho.
3. **`src/cognition/provider.rs`**:
   - Injetar no prompt `[MODELO DO AMBIENTE (WORLD)]` a telemetria em tempo real.
4. **`src/main.rs`**:
   - Comando CLI `corpo` ou `sistema` para exibir o relatório de hardware.
