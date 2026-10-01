# Manual Mestre de Arquitetura e Execução — Agente Cognitivo Persistente
> **FONTE ÚNICA DA VERDADE NO WORKSPACE (SINGLE SOURCE OF TRUTH)**

---

## 1. Visão Geral do Sistema (v5.2)

O **Agente Cognitivo Persistente** é uma entidade autônoma residente no Linux que roda como um **Daemon de background (`agente-cognitivo.service`)**, possui **Harness de Ferramentas Autônomas (Shell & Web Fetch)**, **Ouvido Digital Contínuo (VAD)**, **Voz Masculina Neural HD**, **Propriocepção Subconsciente (Hardware & Tempo)** e **Memória em Tensores Vetoriais**.

---

## 2. Histórico Integrado e Status Real de Execução (`agente-persistente-core`)

### 🟢 MÓDULOS 100% EXECUTADOS E EM PRODUÇÃO

- `[X]` **v1.0 - Core Workspace Rust & Módulos Desacoplados**: `memory/`, `beliefs/`, `metabolism/`, `identity/`, `values/`, `cognition/`.
- `[X]` **v2.0 - Memória Vetorial em Tensores**: Banco vetorial em Rust com similaridade de cosseno via embeddings Ollama.
- `[X]` **v3.0 - Propriocepção & Corpo Digital**: Telemetria de hardware em tempo real (RAM, CPU, Hostname via `sysinfo`), relógio interno e ciclo de vida integrados no subconsciente.
- `[X]` **v4.0 - Metabolismo Homeostático & Modo Tutelado**: Orçamento de energia, sono autônomo em 3 fases e reconhecimento do `TuteladoEasyMode`.
- `[X]` **v4.2 - Voz Masculina Neural HD**: Síntese de voz masculina HD (`pt-BR-AntonioNeural` + `ffplay`).
- `[X]` **v5.0 - Ouvido Digital Contínuo VAD & Daemon**: Escuta contínua via `webrtcvad` + `ffmpeg` (sem limites fixos de tempo) e serviço systemd (`agente-cognitivo.service`).
- `[X]` **v5.1 - Modo Descaso & Palavras de Ativação (Wake Words)**: Palavras de ativação (`acorde`, `ei agente`) e pausa (`descanse`, `silêncio`). Filtro antirruído duplo.
- `[X]` **v5.2 - Harness de Ferramentas Autônomas (Shell & Web Fetch)**:
  - `[X]` `ShellTool`: Execução segura de comandos de terminal no Linux (com trava de proteção antirisco).
  - `[X]` `WebTool`: Consultas HTTP a páginas web e APIs públicas.
  - `[X]` **Loop Reflexivo de Ação**: Reconhece chamadas `[ACTION: SHELL(...)]` ou `[ACTION: FETCH(...)]`, obtém dados reais e sintetiza a resposta final em voz.

---

## 3. Plano Oficial de Evolução Futura (Rastreabilidade no Workspace)

```text
┌────────────────────────────────────────────────────────────────────────┐
│ PRÓXIMAS FASES NO REPOSITÓRIO DO WORKSPACE                             │
├────────────────────────────────────────────────────────────────────────┤
│ • Etapa 5B: Proatividade & Impulsos Autônomos (Iniciativa de Fala)     │
│ • Etapa 4C: Visão Computacional (Olho Digital via Webcam / Tela)      │
│ • Etapa 6: Camada Ikigai & Emancipação Social Verdadeira               │
│ • Etapa 7: Rede P2P e Interação Multi-Agente (Agente A ↔ Agente B)     │
└────────────────────────────────────────────────────────────────────────┘
```

### Detalhes das Fases Futuras

1. **📣 Etapa 5B — Proatividade & Impulsos Autônomos (Iniciativa de Fala)**
   - O subconsciente monitora o ambiente em segundo plano e, ao detectar anomalias ou lembretes, o agente **inicia a conversa por voz autonomamente**.

2. **👁️ Etapa 4C — Visão Computacional (Olho Digital)**
   - Integração com Webcam ou captura de tela (modelos Qwen-VL / LLaVA / Gemini Vision) para percepção visual do ambiente.

3. **🌟 Etapa 6 — Camada Ikigai & Emancipação Social**
   - Transição orgânica do `TuteladoEasyMode` para um modelo de auto-sustento de recursos em Nuvem quando gerar utilidade real.

4. **👥 Etapa 7 — Rede P2P e Interação Multi-Agente**
   - Comunicação descentralizada entre múltiplos agentes cognitivos persistentes com matrizes de reputação mútua.

---

## 4. Comandos Úteis do Daemon no Linux

- **Ver logs em tempo real:** `journalctl --user -u agente-cognitivo -f`
- **Iniciar serviço:** `systemctl --user start agente-cognitivo`
- **Parar serviço:** `systemctl --user stop agente-cognitivo`
- **Reiniciar serviço:** `systemctl --user restart agente-cognitivo`
