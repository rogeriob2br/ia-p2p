# Requirements Document: Pool Production Readiness

## Introduction

Este documento especifica requisitos para tornar a Inference Pool pronta para uso em produção. Estes requisitos complementam a spec `api-integration` adicionando features essenciais de operação, observabilidade e robustez.

**Pré-requisito**: A spec `api-integration` deve estar implementada (inference-pool funcional com API OpenAI-compatible, registro de nodes, billing básico).

**Objetivo**: Transformar o MVP em um sistema que pode ser operado de forma confiável em produção, servindo como modelo de implementação básico mas robusto para outros operadores de pool.

## Glossary

- **Persistence_Layer**: Camada de persistência de dados (SQLite ou PostgreSQL)
- **Health_Check**: Endpoint que reporta saúde do sistema
- **Structured_Logging**: Logs em formato estruturado (JSON) para análise
- **Graceful_Shutdown**: Desligamento ordenado sem perda de dados ou conexões
- **Configuration_Management**: Sistema de configuração via arquivo + variáveis de ambiente
- **Migration**: Script de alteração de schema de banco de dados
- **Observability**: Capacidade de entender o estado interno do sistema

## Requirements

### Requirement 1: Basic Data Persistence

**User Story:** Como pool operator, eu quero que dados críticos sejam persistidos em disco, para que eu não perca informações ao reiniciar a pool.

**Por quê**: In-memory storage perde tudo ao reiniciar. Dados como registry de nodes, API keys, billing tickets e invoices precisam persistir.

#### Acceptance Criteria

1. WHEN a pool inicia, THE Persistence_Layer SHALL conectar ao banco de dados configurado (SQLite ou PostgreSQL)
2. WHEN a pool registra um node, THE Pool_Registry SHALL persistir os dados do node no banco
3. WHEN a pool cria um billing ticket, THE Billing_System SHALL persistir o ticket no banco
4. WHEN a pool reinicia, THE Pool_Registry SHALL carregar nodes registrados do banco
5. THE Persistence_Layer SHALL suportar SQLite (desenvolvimento) e PostgreSQL (produção)
6. THE Persistence_Layer SHALL executar migrations automaticamente ao iniciar

### Requirement 2: Database Migrations

**User Story:** Como pool operator, eu quero que o schema do banco seja gerenciado via migrations, para que eu possa atualizar a pool sem perder dados.

**Por quê**: Schema evolui com o tempo. Migrations permitem atualizar estrutura sem perda de dados.

#### Acceptance Criteria

1. WHEN a pool inicia, THE Persistence_Layer SHALL verificar versão do schema atual
2. IF schema está desatualizado, THEN THE Persistence_Layer SHALL executar migrations pendentes em ordem
3. WHEN migration falha, THE Persistence_Layer SHALL fazer rollback e reportar erro
4. THE Persistence_Layer SHALL manter histórico de migrations executadas
5. THE Persistence_Layer SHALL suportar migrations SQL para criar/alterar tabelas

### Requirement 3: Health Check Endpoint

**User Story:** Como operador de infraestrutura, eu quero um endpoint de health check, para que load balancers e monitoring possam verificar se a pool está saudável.

**Por quê**: Load balancers precisam saber se a pool está funcionando. Monitoring precisa detectar problemas.

#### Acceptance Criteria

1. WHEN uma requisição GET é feita para `/health`, THE API_Server SHALL retornar status da pool
2. WHEN todos os componentes estão funcionando, THE endpoint SHALL retornar HTTP 200 com status "healthy"
3. IF algum componente crítico falha (database, lightning), THEN THE endpoint SHALL retornar HTTP 503 com status "unhealthy"
4. THE endpoint SHALL incluir: status, version, uptime_seconds, active_members, database_status, lightning_status
5. THE endpoint SHALL responder em menos de 100ms
6. THE endpoint SHALL NOT requerer autenticação (público para load balancers)

### Requirement 4: Structured Logging

**User Story:** Como pool operator, eu quero logs estruturados em formato JSON, para que eu possa analisar logs facilmente e integrar com ferramentas de logging.

**Por quê**: Logs em texto livre são difíceis de analisar. JSON permite queries, alertas e dashboards.

#### Acceptance Criteria

1. WHEN a pool loga eventos, THE sistema SHALL usar structured logging (tracing crate)
2. WHEN evento é logado, THE log SHALL incluir: timestamp, level, message, context fields
3. THE pool SHALL logar eventos importantes: node_registered, job_routed, payment_sent, error_occurred
4. WHEN erro ocorre, THE log SHALL incluir: error_type, error_message, stack_trace, context
5. THE pool SHALL suportar níveis de log configuráveis: ERROR, WARN, INFO, DEBUG, TRACE
6. THE pool SHALL permitir configurar formato de output: JSON (produção) ou pretty (desenvolvimento)

### Requirement 5: Enhanced Graceful Shutdown

**User Story:** Como pool operator, eu quero que a pool desligue de forma ordenada, para que não haja perda de dados ou conexões abruptas.

**Por quê**: Shutdown abrupto pode corromper dados, perder billing tickets, ou fechar canais Lightning de forma não-cooperativa.

#### Acceptance Criteria

1. WHEN pool recebe sinal de shutdown (SIGTERM, SIGINT), THE pool SHALL iniciar graceful shutdown
2. WHEN shutdown inicia, THE API_Server SHALL parar de aceitar novas requisições
3. WHILE shutdown está em progresso, THE pool SHALL aguardar requisições em andamento completarem (timeout: 30s)
4. WHEN requisições completam, THE pool SHALL fechar canais Lightning cooperativamente
5. WHEN canais fecham, THE pool SHALL fazer flush de dados pendentes para banco
6. WHEN flush completa, THE pool SHALL fechar conexão com banco de dados
7. THE pool SHALL logar cada etapa do shutdown para auditoria

### Requirement 6: Configuration Management

**User Story:** Como pool operator, eu quero configurar a pool via arquivo TOML e variáveis de ambiente, para que eu possa adaptar configuração para diferentes ambientes (dev, staging, prod).

**Por quê**: Hardcoded configs são inflexíveis. Arquivo + env vars permite configuração por ambiente.

#### Acceptance Criteria

1. WHEN a pool inicia, THE pool SHALL carregar configuração de arquivo `config.toml`
2. WHEN variável de ambiente está definida, THE pool SHALL sobrescrever valor do arquivo
3. THE pool SHALL suportar sintaxe `${VAR_NAME}` no config.toml para referenciar env vars
4. WHEN configuração obrigatória está faltando, THE pool SHALL falhar ao iniciar com erro claro
5. THE pool SHALL validar configuração ao carregar (tipos, ranges, formatos)
6. THE pool SHALL expor configuração atual via endpoint `/v1/admin/config` (sem secrets)

### Requirement 7: Startup Validation

**User Story:** Como pool operator, eu quero que a pool valide configuração e dependências ao iniciar, para que eu detecte problemas antes de aceitar tráfego.

**Por quê**: Falhar rápido ao iniciar é melhor que falhar durante operação.

#### Acceptance Criteria

1. WHEN a pool inicia, THE pool SHALL validar configuração completa
2. WHEN a pool inicia, THE pool SHALL testar conexão com banco de dados
3. WHEN a pool inicia, THE pool SHALL testar conexão com Lightning daemon
4. IF alguma validação falha, THEN THE pool SHALL logar erro detalhado e sair com código não-zero
5. WHEN todas validações passam, THE pool SHALL logar "Pool ready to accept traffic"

### Requirement 8: Admin API - API Key Management

**User Story:** Como pool operator, eu quero gerenciar API keys via API administrativa, para que eu possa criar e revogar keys sem acessar banco diretamente.

**Por quê**: Operação manual de banco é propensa a erros. API administrativa é mais segura e auditável.

#### Acceptance Criteria

1. WHEN pool expõe `POST /v1/admin/keys/create`, THE endpoint SHALL criar nova API key
2. WHEN criando key, THE endpoint SHALL aceitar: client_id, tier, rate_limit, expires_at
3. WHEN key é criada, THE endpoint SHALL retornar key gerada (mostrar apenas uma vez)
4. WHEN pool expõe `POST /v1/admin/keys/revoke`, THE endpoint SHALL revogar key especificada
5. WHEN pool expõe `GET /v1/admin/keys`, THE endpoint SHALL listar todas as keys (sem mostrar valor)
6. THE admin endpoints SHALL requerer autenticação via admin token configurado

### Requirement 9: Admin API - Member Management

**User Story:** Como pool operator, eu quero visualizar e gerenciar nodes membros via API administrativa, para que eu possa monitorar e operar a pool.

**Por quê**: Visibilidade e controle sobre nodes registrados é essencial para operação.

#### Acceptance Criteria

1. WHEN pool expõe `GET /v1/admin/members`, THE endpoint SHALL listar todos os nodes com detalhes
2. WHEN listando members, THE endpoint SHALL incluir: node_id, status, models, metrics, last_heartbeat
3. WHEN pool expõe `GET /v1/admin/members/{node_id}`, THE endpoint SHALL retornar detalhes completos do node
4. WHEN pool expõe `POST /v1/admin/members/{node_id}/deregister`, THE endpoint SHALL desregistrar node com razão
5. WHEN pool expõe `GET /v1/admin/stats`, THE endpoint SHALL retornar estatísticas agregadas da pool

### Requirement 10: Environment-Specific Defaults

**User Story:** Como pool operator, eu quero defaults sensatos para diferentes ambientes, para que configuração mínima seja suficiente para começar.

**Por quê**: Configuração complexa é barreira de entrada. Defaults bons facilitam adoção.

#### Acceptance Criteria

1. WHEN ambiente é "development", THE pool SHALL usar: SQLite, log level DEBUG, mock Lightning
2. WHEN ambiente é "production", THE pool SHALL requerer: PostgreSQL URL, Lightning config, TLS certs
3. THE pool SHALL detectar ambiente via variável `POOL_ENV` (default: development)
4. WHEN configuração mínima é fornecida, THE pool SHALL usar defaults para valores opcionais
5. THE pool SHALL documentar todos os defaults no README

