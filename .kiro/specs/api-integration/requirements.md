# Requirements Document

## Introduction

Este documento especifica os requisitos para uma Inference Pool - um componente de Layer 2 que agrega nodes da rede P2P de IA descentralizada e expõe uma API unificada compatível com OpenAI. Pools são operadores independentes que facilitam o acesso à rede para desenvolvedores que preferem não gerenciar conexões P2P diretamente. Múltiplas pools podem coexistir e competir, preservando a natureza descentralizada do sistema.

**Modelo de Operação:**
- Operadores de nodes escolhem manualmente qual pool usar (via contrato/acordo comercial)
- Nodes se registram na pool via configuração manual (não há descoberta automática)
- Pool monitora a orquestração distribuída que acontece entre os nodes
- Pool visualiza papéis, grupos de contexto, e métricas da rede
- Pool pode desregistrar nodes com baixa performance

## Glossary

- **Inference_Pool**: Agregador Layer 2 que gerencia um conjunto de nodes e expõe API unificada
- **Pool_Operator**: Entidade que opera uma Inference_Pool
- **Pool_Member**: Node da rede P2P que se registrou manualmente em uma pool específica
- **Pool_Registry**: Registro de nodes membros e suas capacidades dentro da pool
- **OpenAI_Compatible_API**: Endpoints que seguem a especificação da API OpenAI
- **Job_Router**: Componente da pool que distribui jobs entre Pool_Members
- **Pool_Fee**: Taxa cobrada pela pool por facilitar inferências
- **SLA_Tier**: Nível de serviço oferecido pela pool (latência, disponibilidade)
- **Node**: Instância da rede P2P Layer 1 que processa inferências
- **Streaming**: Modo de resposta onde tokens são enviados incrementalmente via SSE
- **Registration_Key**: Credencial fornecida pela pool para operador de node registrar seu node
- **Network_Map**: Visualização da topologia e papéis dos nodes na orquestração distribuída
- **Orchestration_Observer**: Componente que monitora eventos da orquestração distribuída
- **Context_Group**: Agrupamento dinâmico de nodes especializados em um domínio específico
- **Emergent_Coordinator**: Node que assume temporariamente papel de coordenação na rede
- **Group_Leader**: Node que coordena decisões dentro de um Context_Group específico
- **Billing_System**: Sistema de bilhetagem para cobrança de clientes e pagamento de nodes
- **Billing_Ticket**: Registro individual de consumo e custo de uma inferência
- **Invoice**: Fatura consolidada de um período para um cliente
- **Lightning_Channel**: Canal de pagamento direto Lightning entre Pool e Pool_Member
- **Channel_Manager**: Componente que gerencia abertura, fechamento e rebalanceamento de canais Lightning

## Requirements

### Requirement 1: Pool Membership

**User Story:** Como operador de node, eu quero registrar meu node em uma pool via configuração manual, para que eu possa receber jobs sem gerenciar conexões diretas com clientes.

#### Acceptance Criteria

1. WHEN um Node envia PoolRegistrationRequest com Registration_Key válida, THE Inference_Pool SHALL verificar a conectividade e capacidades do node
2. WHEN um Node é aceito, THE Pool_Registry SHALL armazenar as capacidades do node (modelos, GPU, memória, Lightning pubkey)
3. WHEN um Node é aceito, THE Inference_Pool SHALL abrir canal Lightning direto com o node
4. WHILE um Pool_Member está ativo, THE Inference_Pool SHALL monitorar sua disponibilidade via heartbeat
5. IF um Pool_Member não responde ao heartbeat por período configurável, THEN THE Pool_Registry SHALL marcar o node como inativo

### Requirement 2: Chat Completions API

**User Story:** Como desenvolvedor usando LangChain, eu quero chamar o endpoint `/v1/chat/completions` com o mesmo formato da OpenAI, para que meu código existente funcione sem modificações.

#### Acceptance Criteria

1. WHEN uma requisição POST é feita para `/v1/chat/completions`, THE Inference_Pool SHALL aceitar o payload no formato OpenAI
2. WHEN o payload contém `messages` array, THE Inference_Pool SHALL processar mensagens com roles (system, user, assistant)
3. WHEN `stream: true` é especificado, THE Inference_Pool SHALL retornar resposta via Server-Sent Events
4. WHEN `stream: false` ou omitido, THE Inference_Pool SHALL retornar resposta JSON completa
5. WHEN a inferência completa, THE Inference_Pool SHALL retornar resposta com `choices`, `usage` e `id` no formato OpenAI

### Requirement 3: Embeddings API

**User Story:** Como desenvolvedor, eu quero gerar embeddings através do endpoint `/v1/embeddings`, para que eu possa usar a pool para tarefas de busca semântica e RAG.

#### Acceptance Criteria

1. WHEN uma requisição POST é feita para `/v1/embeddings`, THE Inference_Pool SHALL aceitar input como string ou array de strings
2. WHEN o input é processado, THE Inference_Pool SHALL retornar vetores de embedding no formato OpenAI
3. WHEN múltiplos inputs são fornecidos, THE Inference_Pool SHALL retornar um embedding para cada input na mesma ordem

### Requirement 4: Models Discovery

**User Story:** Como desenvolvedor, eu quero listar os modelos disponíveis na pool, para que eu possa descobrir quais modelos posso usar.

#### Acceptance Criteria

1. WHEN uma requisição GET é feita para `/v1/models`, THE Inference_Pool SHALL retornar lista agregada de modelos disponíveis nos Pool_Members
2. WHEN um modelo é listado, THE Inference_Pool SHALL incluir quantidade de nodes que o suportam e capacidade total disponível

### Requirement 5: Job Routing

**User Story:** Como pool operator, eu quero distribuir jobs entre nodes membros de forma inteligente, para que eu possa otimizar latência e utilização.

#### Acceptance Criteria

1. WHEN um job é recebido, THE Job_Router SHALL selecionar um Pool_Member disponível que suporte o modelo solicitado
2. WHEN múltiplos Pool_Members estão disponíveis, THE Job_Router SHALL considerar carga atual, latência histórica e capacidade
3. IF o Pool_Member selecionado falha, THEN THE Job_Router SHALL fazer retry em outro Pool_Member disponível
4. WHEN nenhum Pool_Member está disponível para o modelo, THE Inference_Pool SHALL retornar erro 503 com mensagem descritiva

### Requirement 6: Pool Authentication

**User Story:** Como pool operator, eu quero autenticar clientes via API Key, para que eu possa controlar acesso e rastrear uso para cobrança.

#### Acceptance Criteria

1. WHEN uma requisição contém header `Authorization: Bearer <api_key>`, THE Inference_Pool SHALL validar a API_Key
2. IF uma requisição não contém API_Key válida, THEN THE Inference_Pool SHALL retornar erro 401 Unauthorized
3. WHEN uma API_Key é validada, THE Inference_Pool SHALL associar a requisição ao cliente para tracking de uso e billing

### Requirement 7: Usage Tracking e Pool Fee

**User Story:** Como pool operator, eu quero rastrear uso por cliente e aplicar taxas, para que eu possa monetizar a operação da pool.

#### Acceptance Criteria

1. WHEN uma inferência é completada, THE Inference_Pool SHALL registrar tokens consumidos associados à API_Key do cliente
2. WHEN uma inferência é completada, THE Inference_Pool SHALL calcular Pool_Fee baseado em tokens e SLA_Tier
3. THE Inference_Pool SHALL expor endpoint `/v1/usage` para clientes consultarem seu consumo

### Requirement 8: Pool Status e Transparency

**User Story:** Como desenvolvedor, eu quero consultar o status da pool, para que eu possa monitorar disponibilidade e tomar decisões informadas.

#### Acceptance Criteria

1. WHEN uma requisição GET é feita para `/v1/pool/status`, THE Inference_Pool SHALL retornar estatísticas incluindo Pool_Members ativos, capacidade total e latência média
2. WHEN uma requisição GET é feita para `/v1/pool/info`, THE Inference_Pool SHALL retornar informações públicas da pool incluindo Pool_Fee, SLA_Tiers oferecidos e políticas

### Requirement 9: Error Handling

**User Story:** Como desenvolvedor, eu quero receber erros no formato padrão da OpenAI, para que meu código de tratamento de erros existente funcione corretamente.

#### Acceptance Criteria

1. IF um erro ocorre durante o processamento, THEN THE Inference_Pool SHALL retornar resposta de erro no formato OpenAI com `error.message`, `error.type` e `error.code`
2. IF o modelo solicitado não está disponível na pool, THEN THE Inference_Pool SHALL retornar erro 404 com type `model_not_found`
3. IF o payload é inválido, THEN THE Inference_Pool SHALL retornar erro 400 com type `invalid_request_error`
4. IF nenhum Pool_Member está disponível, THEN THE Inference_Pool SHALL retornar erro 503 com type `service_unavailable`

### Requirement 10: Registration API

**User Story:** Como pool operator, eu quero expor uma API de registro para nodes, para que operadores possam registrar seus nodes após acordo comercial.

#### Acceptance Criteria

1. WHEN a Pool expõe `POST /v1/pool/register`, THE endpoint SHALL aceitar PoolRegistrationRequest com Registration_Key
2. WHEN Registration_Key é inválida, THE Pool SHALL retornar erro 401 Unauthorized
3. WHEN Node é registrado com sucesso, THE Pool SHALL retornar RegistrationResponse com member_id e channel_id
4. WHEN Pool expõe `GET /v1/pool/info`, THE endpoint SHALL retornar informações públicas (fee, modelos, SLA, capacidade) sem autenticação
5. THE Pool SHALL permitir operador revogar Registration_Key via interface administrativa

### Requirement 11: Orchestration Monitoring

**User Story:** Como pool operator, eu quero monitorar a orquestração distribuída dos nodes, para que eu possa visualizar papéis, grupos e métricas da rede.

#### Acceptance Criteria

1. WHEN nodes formam Context_Group, THE nodes SHALL notificar a Pool via P2P com detalhes do grupo
2. WHEN Emergent_Coordinator muda, THE nodes SHALL notificar a Pool com novo coordenador e affinity score
3. WHEN Group_Leader é eleito, THE nodes SHALL notificar a Pool com líder e membros do grupo
4. WHEN Pool recebe eventos de orquestração, THE Orchestration_Observer SHALL atualizar Network_Map interno
5. THE Pool SHALL expor endpoint `GET /v1/pool/network-map` com topologia atual da rede

### Requirement 12: Node Health Management

**User Story:** Como pool operator, eu quero desregistrar nodes com baixa performance, para que eu possa manter qualidade de serviço.

#### Acceptance Criteria

1. WHEN node tem success rate < 90% por 24h, THE Pool SHALL marcar node como "at risk"
2. WHEN node tem uptime < 95% por 24h, THE Pool SHALL enviar warning ao operador do node
3. WHEN node falha critérios de qualidade por 48h, THE Pool SHALL desregistrar node automaticamente
4. WHEN node é desregistrado, THE Pool SHALL fechar canal Lightning cooperativamente
5. THE Pool SHALL expor endpoint `POST /v1/pool/admin/deregister` para desregistro manual com justificativa

### Requirement 13: Orchestration Integration

**User Story:** Como pool operator, eu quero que a Pool respeite decisões da orquestração distribuída, para que roteamento seja otimizado.

#### Acceptance Criteria

1. WHEN Pool roteia job, THE Job_Router SHALL consultar Network_Map para identificar Group_Leader apropriado
2. WHEN job requer contexto específico, THE Job_Router SHALL rotear para nodes do Context_Group correspondente
3. WHEN Emergent_Coordinator está sobrecarregado, THE Job_Router SHALL distribuir jobs para outros nodes do grupo
4. WHEN consenso adaptativo é necessário, THE Pool SHALL aguardar decisão da rede antes de confirmar roteamento
5. THE Pool SHALL priorizar nodes com alta participação em consenso e boa reputação

### Requirement 14: LLM-Powered Orchestration

**User Story:** Como pool operator, eu quero que a pool use LLM para tomar decisões inteligentes de roteamento complexo, para que eu possa oferecer melhor qualidade de serviço em casos específicos.

#### Acceptance Criteria

1. WHEN decisões complexas de roteamento são necessárias, THE Pool SHALL consultar um LLM local para análise
2. WHEN métricas de performance indicam problemas, THE Pool SHALL usar LLM para sugerir otimizações
3. WHEN um cliente tem requisitos específicos de SLA, THE Pool SHALL usar LLM para planejar alocação de recursos
4. THE Pool SHALL manter histórico de decisões LLM para fine-tuning futuro
5. WHEN orquestração distribuída já tomou decisão, THE Pool SHALL respeitar decisão da rede ao invés de usar LLM

### Requirement 15: Billing System

**User Story:** Como pool operator, eu quero um sistema completo de bilhetagem, para que eu possa cobrar clientes e pagar nodes de forma transparente.

#### Acceptance Criteria

1. WHEN uma inferência é completada, THE Billing_System SHALL criar um ticket com detalhes de consumo e custo
2. WHEN um ticket é criado, THE Billing_System SHALL calcular a divisão entre Pool_Fee e pagamento ao Pool_Member
3. THE Billing_System SHALL expor endpoint `/v1/billing/invoices` para clientes consultarem faturas
4. THE Billing_System SHALL expor endpoint `/v1/billing/earnings` para Pool_Members consultarem ganhos
5. WHEN o período de faturamento fecha, THE Billing_System SHALL gerar invoice consolidada por cliente
6. THE Billing_System SHALL suportar múltiplas moedas incluindo crypto (Lightning Network)

### Requirement 16: Lightning Channel Management

**User Story:** Como pool operator, eu quero gerenciar canais Lightning diretos com Pool_Members, para que eu possa pagar nodes instantaneamente sem fees de routing.

#### Acceptance Criteria

1. WHEN um Pool_Member é registrado, THE Channel_Manager SHALL abrir um canal Lightning direto com o node
2. WHEN uma inferência é completada, THE Channel_Manager SHALL enviar pagamento instantâneo ao Pool_Member via canal direto
3. WHILE um canal está ativo, THE Channel_Manager SHALL monitorar liquidez e rebalancear quando necessário
4. WHEN um Pool_Member sai da pool, THE Channel_Manager SHALL fechar o canal de forma cooperativa
5. IF um canal não pode ser aberto, THEN THE Channel_Manager SHALL usar fallback para pagamentos via Lightning routing
6. THE Channel_Manager SHALL expor métricas de canais incluindo capacidade, liquidez e status

---

## Node-Side Requirements

**IMPORTANTE**: Os requisitos abaixo especificam mudanças necessárias no **mvp-node** (código em `mvp-node/src/`) para suportar integração com Inference Pools. Estes requisitos são parte da mesma feature porque Pool e Node precisam se comunicar bidirecionalmente.

**Localização das Mudanças**: `mvp-node/src/pool/` (novo módulo)

### Requirement 17: Node Pool Registration

**User Story:** Como node da rede P2P, eu quero me registrar em uma pool configurada manualmente, para que eu possa receber jobs agregados e pagamentos via Lightning.

**Por quê**: Nodes precisam se identificar e enviar suas capacidades para a pool aceitar e rotear jobs corretamente.

**Onde**: `mvp-node/src/pool/registration.rs` (novo arquivo)

#### Acceptance Criteria

1. WHEN node inicia com pool configurada em config.toml, THE Node SHALL ler pool_url e registration_key
2. WHEN node tem credenciais válidas, THE Node SHALL enviar PoolRegistrationRequest para `POST /v1/pool/register`
3. WHEN Pool aceita registro, THE Node SHALL armazenar member_id e channel_id recebidos
4. IF registro falha, THEN THE Node SHALL logar erro e tentar novamente após intervalo configurável
5. THE Node SHALL incluir no registro: node_id, peer_id, capabilities, models, lightning_pubkey, multiaddrs

### Requirement 18: Node Heartbeat

**User Story:** Como node registrado em uma pool, eu quero enviar heartbeats periódicos, para que a pool saiba que estou ativo e disponível.

**Por quê**: Pool precisa monitorar disponibilidade dos nodes para roteamento e desregistro automático de nodes inativos.

**Onde**: `mvp-node/src/pool/heartbeat.rs` (novo arquivo)

#### Acceptance Criteria

1. WHEN node está registrado em pool, THE Node SHALL iniciar task assíncrona de heartbeat
2. WHILE node está ativo, THE Node SHALL enviar heartbeat a cada intervalo configurado (padrão: 30s)
3. WHEN enviando heartbeat, THE Node SHALL incluir: node_id, timestamp, current_load, available_capacity
4. IF heartbeat falha por N tentativas consecutivas, THEN THE Node SHALL tentar re-registro
5. THE Node SHALL expor métrica de última resposta de heartbeat bem-sucedida

### Requirement 19: Node Orchestration Event Publishing

**User Story:** Como node participando de orquestração distribuída, eu quero notificar a pool sobre eventos importantes, para que a pool possa visualizar topologia e otimizar roteamento.

**Por quê**: Pool precisa saber sobre coordenadores, grupos de contexto e consenso para tomar decisões inteligentes de roteamento.

**Onde**: `mvp-node/src/pool/event_publisher.rs` (novo arquivo) + modificações em `mvp-node/src/orchestration/`

#### Acceptance Criteria

1. WHEN node se torna Emergent_Coordinator, THE Node SHALL publicar OrchestrationEvent::CoordinatorChanged via P2P para pool
2. WHEN node forma ou entra em Context_Group, THE Node SHALL publicar OrchestrationEvent::ContextGroupFormed
3. WHEN node participa de consenso adaptativo, THE Node SHALL publicar OrchestrationEvent::ConsensusReached após decisão
4. WHEN node muda de papel (regular → coordinator → leader), THE Node SHALL publicar OrchestrationEvent::NodeRoleChanged
5. WHEN node detecta partição de rede, THE Node SHALL publicar OrchestrationEvent::NetworkPartitionDetected
6. THE Node SHALL usar topic P2P dedicado para eventos de orquestração (ex: `/pool/orchestration/1.0.0`)

### Requirement 20: Node Job Reception from Pool

**User Story:** Como node registrado em pool, eu quero receber e processar jobs vindos da pool, para que eu possa servir clientes da pool e receber pagamentos.

**Por quê**: Pool roteia jobs de clientes para nodes - nodes precisam aceitar e processar esses jobs.

**Onde**: `mvp-node/src/inference/handler.rs` (modificar existente) + `mvp-node/src/pool/job_receiver.rs` (novo)

#### Acceptance Criteria

1. WHEN node recebe job via P2P da pool, THE Node SHALL identificar origem como pool (vs P2P direto)
2. WHEN processando job da pool, THE Node SHALL executar inferência normalmente
3. WHEN inferência completa, THE Node SHALL retornar resultado para pool via P2P
4. IF inferência falha, THEN THE Node SHALL retornar erro estruturado para pool
5. THE Node SHALL rastrear métricas separadas para jobs da pool vs jobs P2P diretos

### Requirement 21: Node Lightning Channel Acceptance

**User Story:** Como node registrado em pool, eu quero aceitar abertura de canal Lightning da pool, para que eu possa receber pagamentos instantâneos por inferências.

**Por quê**: Pool paga nodes via canais Lightning diretos - nodes precisam aceitar e gerenciar esses canais.

**Onde**: `mvp-node/src/lightning/channel.rs` (novo ou modificar existente)

#### Acceptance Criteria

1. WHEN pool solicita abertura de canal Lightning, THE Node SHALL aceitar se tiver capacidade
2. WHEN canal é aberto, THE Node SHALL armazenar channel_id e associar com pool
3. WHEN node recebe pagamento via canal, THE Node SHALL registrar pagamento e associar com job_id
4. WHILE canal está ativo, THE Node SHALL monitorar saldo e alertar se liquidez baixa
5. WHEN pool solicita fechamento cooperativo, THE Node SHALL cooperar e fechar canal gracefully

### Requirement 22: Node Metrics Reporting

**User Story:** Como node registrado em pool, eu quero reportar métricas de performance periodicamente, para que a pool possa avaliar minha saúde e tomar decisões de roteamento.

**Por quê**: Pool usa métricas para roteamento inteligente e desregistro de nodes com baixa performance.

**Onde**: `mvp-node/src/pool/metrics.rs` (novo arquivo)

#### Acceptance Criteria

1. WHILE node está registrado, THE Node SHALL coletar métricas locais: success_rate, avg_latency_ms, current_load, jobs_completed, tokens_processed
2. WHEN período de reporte chega (padrão: 5min), THE Node SHALL enviar métricas para pool via P2P
3. WHEN node participa de consenso, THE Node SHALL incluir consensus_participation_rate nas métricas
4. THE Node SHALL calcular success_rate como: jobs_completed / (jobs_completed + jobs_failed)
5. THE Node SHALL expor endpoint local `/metrics` para debugging (opcional)
