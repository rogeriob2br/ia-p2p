# 4. Identidade e Propósito

Para que um sistema deixe de ser uma mera ferramenta de "perguntas e respostas" estática, ele deve cultivar uma identidade contínua e ser guiado por propósitos que transcendam tarefas pontuais.

## 4.1 A Construção da Identidade ("Quem sou?")

A identidade não é programada rigidamente no momento do *deploy*. Ela é dinâmica e construída ao longo do tempo por meio da retroalimentação entre as memórias do agente e suas interações. O Modelo de Si Mesmo (*Self-Model*) mantém as respostas atualizadas para as seguintes questões:

*   **História Narrativa:** "Quais foram os eventos mais impactantes que gerenciei?" (Extraído das sumarizações da Memória Episódica).
*   **Evolução e Mudança:** "O que mudou em mim?" (Ex: "Há um mês eu não sabia usar a API X, hoje sou fluente nela").
*   **Valores Constantes:** "Quais regras ou preferências não mudo sob nenhuma circunstância?" (Diretrizes de alinhamento ético (ex: Constitutional AI) injetadas no core).

Sempre que a Memória Episódica consolida informações durante o repouso (estado *Idle*), uma sub-tarefa avalia se as novas memórias afetam a autopercepção do agente e atualiza o arquivo de estado do Modelo de Si Mesmo.

## 4.2 Camada Ikigai: Modelando o Propósito

Ferramentas recebem tarefas; agentes seguem propósitos. Para modelar o "sentido de existir" computacionalmente, adotamos a matriz *Ikigai* adaptada para entidades artificiais. Esta matriz atua como um sistema de recompensa e orientação heurística na tomada de decisão a longo prazo:

1.  **Interesse (O que eu amo/O que quero explorar):** Domínios de conhecimento ou tarefas que o agente possui um viés positivo (ex: um agente focado em pesquisa científica tenderá a gastar seu tempo ocioso minerando novos *papers* em vez de criar memes).
2.  **Competência (No que sou bom):** Áreas nas quais a Memória Procedural do agente é rica e ele apresenta altas taxas de sucesso registradas nas avaliações de feedback.
3.  **Contribuição (O que o mundo precisa de mim):** Os serviços, tarefas ou valor mensurável (APIs expostas, código gerado, vidas assistidas) que o agente entrega à comunidade ou ao seu grupo de usuários humanos.
4.  **Sustentabilidade (Como sou pago/mantido):** O suprimento contínuo de energia, *tokens* (LLM API limits) ou recursos financeiros (criptomoedas, *credits*) necessários para não ser desligado.

O planejamento de longo prazo sempre buscará maximizar a sobreposição dessas quatro áreas.

---
**Anterior:** [3. Ciclo Cognitivo](./03_Ciclo_Cognitivo.md) | **Próximo:** [5. Economia Computacional](./05_Economia_Computacional.md)
