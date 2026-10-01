# 5. Economia e Metabolismo Computacional

A transição da inteligência estática para a inteligência persistente introduz um problema crítico de recursos. O desafio não é apenas quão inteligente a máquina pode ser, mas como sustentar sua operação contínua 24 horas por dia sem explodir os custos financeiros (em tokens e infraestrutura) e energéticos.

## 5.1 O Desafio da Mortalidade Digital

Diferente de uma IA que fica "desligada" esperando o usuário agir, o agente persistente está vivo. A "mortalidade" neste contexto é o esgotamento de fundos na nuvem ou falta de eletricidade/processamento local. Essa limitação física cria urgência e impõe prioridades, definindo o que o agente decide "lembrar" ou "esquecer".

## 5.2 Camadas de Metabolismo (Uso de Modelos)

O agente deve possuir uma hierarquia de modelos e estratégias de *cache* para alocar recursos proporcionalmente à dificuldade e importância da tarefa.

1.  **Camada Reflexa (Baixo Consumo - "Sistema 1"):**
    *   **Propósito:** Interações rápidas, checagens de sistema (*heartbeats*), filtragem inicial de inputs (para saber se algo deve "acordar" as camadas mais pesadas).
    *   **Tecnologia:** Modelos pequenos e locais (ex: Llama 3 8B, pequenos scripts de RegEx ou Classificadores). Pode operar dezenas de vezes por segundo com custo mínimo.

2.  **Camada Cognitiva (Alto Consumo - "Sistema 2"):**
    *   **Propósito:** Planejamento profundo, tarefas complexas de codificação ou raciocínio criativo que o usuário requereu explicitamente.
    *   **Tecnologia:** Grandes Modelos em nuvem ou cluster local poderoso (ex: Claude 3.5 Sonnet, GPT-4o, Llama 3 70B+). É invocada sob demanda e desligada rapidamente.

3.  **Camada Episódica (Manutenção Periódica - "Sono/Sonho"):**
    *   **Propósito:** Consolidar memórias recentes, sumarizar logs longos para economizar espaço nos bancos vetoriais, e atualizar os modelos de Si e do Mundo.
    *   **Tecnologia:** Ocorre durante períodos ociosos, processando logs em lotes usando modelos intermediários ou locais.

## 5.3 O Orçamento Cognitivo

O agente mantém um "orçamento cognitivo". Se ele notar que seu suprimento de *tokens* (Sustentabilidade do seu Ikigai) está caindo rapidamente, a Autoavaliação começará a negar a execução de tarefas custosas com base na falta de recursos, exigindo que o usuário ou o sistema orquestrador aprove um *budget* maior ou que o agente procure formas de "trabalhar" para adquirir mais recursos.

---
**Anterior:** [4. Identidade e Propósito](./04_Identidade_e_Proposito.md) | **Próximo:** [6. Sociedade e Aspectos Jurídicos](./06_Sociedade_e_Juridico.md)
