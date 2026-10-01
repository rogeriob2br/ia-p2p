# Implementation Plan: Pool Production Readiness

## Overview

Este plano implementa features essenciais para tornar a Inference Pool pronta para produção. A implementação adiciona persistência, observabilidade, configuração robusta e APIs administrativas ao MVP funcional da spec `api-integration`.

**Pré-requisito**: Spec `api-integration` deve estar implementada (inference-pool funcional).

**Objetivo**: Transformar MVP em sistema operável em produção, servindo como modelo de implementação básico mas robusto.

## Tasks

- [ ] 1. Setup Database Layer
  - [ ] 1.1 Adicionar dependências de database
    - Adicionar `sqlx` com features sqlite e postgres ao Cargo.toml
    - Adicionar `sqlx-cli` para migrations
    - _Requirements: 1.5_
    - _Localização: `inference-pool/Cargo.toml`_
  - [ ] 1.2 Criar estruturas de configuração
    - `DatabaseConfig`, `DatabaseType`, `DatabasePool`
    - _Requirements: 1.1_
    - _Localização: `inference-pool/src/database/mod.rs`_
  - [ ] 1.3 Implementar DatabasePool
    - `new()` - criar pool de conexões
    - `acquire()` - obter conexão
    - `health_check()` - verificar saúde
    - `close()` - fechar conexões
    - _Requirements: 1.1, 1.2_
    - _Localização: `inference-pool/src/database/pool.rs`_
  - [ ] 1.4 Write property test for database round-trip
    - **Property 35: Database Persistence Round-Trip**
    - **Validates: Requirements 1.2, 1.4**
    - _Localização: `inference-pool/tests/property/database.rs`_

- [ ] 2. Implementar Database Migrations
  - [ ] 2.1 Criar migration inicial (001_initial_schema.sql)
    - Tabelas: pool_members, api_keys, billing_tickets, invoices, lightning_channels, migrations
    - _Requirements: 2.5_
    - _Localização: `inference-pool/migrations/001_initial_schema.sql`_
  - [ ] 2.2 Implementar MigrationManager
    - `current_version()` - versão atual do schema
    - `pending_migrations()` - migrations pendentes
    - `migrate_up()` - executar migrations
    - `migrate_down()` - rollback
    - `migration_history()` - histórico
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
    - _Localização: `inference-pool/src/database/migrations.rs`_
  - [ ] 2.3 Integrar migrations no startup
    - Executar migrations automaticamente se `auto_migrate = true`
    - Logar cada migration executada
    - _Requirements: 1.6, 2.2_
    - _Localização: `inference-pool/src/main.rs`_
  - [ ] 2.4 Write property test for migration idempotency
    - **Property 36: Migration Idempotency**
    - **Validates: Requirements 2.2**
    - _Localização: `inference-pool/tests/property/migrations.rs`_

- [ ] 3. Checkpoint - Database Layer Complete
  - Ensure database tests pass
  - Verify migrations run successfully
  - Test with both SQLite and PostgreSQL
  - Ask the user if questions arise.

- [ ] 4. Integrar Database com Componentes Existentes
  - [ ] 4.1 Atualizar Pool Registry para usar database
    - Modificar `register_member()` para persistir no banco
    - Modificar `get_active_members()` para carregar do banco
    - Adicionar `load_from_database()` no startup
    - _Requirements: 1.2, 1.4_
    - _Localização: `inference-pool/src/registry/mod.rs`_
  - [ ] 4.2 Atualizar Auth Middleware para usar database
    - Modificar `validate_key()` para consultar banco
    - Modificar `create_key()` para persistir no banco
    - Modificar `revoke_key()` para atualizar banco
    - _Requirements: 1.2_
    - _Localização: `inference-pool/src/auth/mod.rs`_
  - [ ] 4.3 Atualizar Billing System para usar database
    - Modificar `create_ticket()` para persistir no banco
    - Modificar `generate_invoice()` para persistir no banco
    - Adicionar queries para consultar tickets e invoices
    - _Requirements: 1.3_
    - _Localização: `inference-pool/src/billing/mod.rs`_
  - [ ] 4.4 Atualizar Channel Manager para usar database
    - Modificar `open_channel()` para persistir no banco
    - Modificar `send_payment()` para atualizar banco
    - Adicionar `load_channels()` no startup
    - _Requirements: 1.2_
    - _Localização: `inference-pool/src/channel_manager/mod.rs`_

- [ ] 5. Implementar Configuration Manager
  - [ ] 5.1 Criar estruturas de configuração
    - `PoolConfig`, `PoolInfo`, `ServerConfig`, `AdminConfig`
    - _Requirements: 6.1_
    - _Localização: `inference-pool/src/config/mod.rs`_
  - [ ] 5.2 Implementar carregamento de configuração
    - `load()` - carregar de arquivo + env vars
    - `load_from_file()` - ler config.toml
    - `apply_env_overrides()` - aplicar env vars
    - `validate()` - validar configuração
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
    - _Localização: `inference-pool/src/config/loader.rs`_
  - [ ] 5.3 Criar config.toml de exemplo
    - Documentar todos os campos
    - Incluir exemplos de env vars
    - Defaults para development
    - _Requirements: 10.1, 10.4_
    - _Localização: `inference-pool/config.example.toml`_
  - [ ] 5.4 Implementar environment-specific defaults
    - Detectar ambiente via `POOL_ENV`
    - Aplicar defaults para development/production
    - _Requirements: 10.1, 10.2, 10.3_
    - _Localização: `inference-pool/src/config/defaults.rs`_
  - [ ] 5.5 Write property test for config override
    - **Property 38: Configuration Override Precedence**
    - **Validates: Requirements 6.2**
    - _Localização: `inference-pool/tests/property/config.rs`_

- [ ] 6. Implementar Structured Logging
  - [ ] 6.1 Adicionar dependências de logging
    - Adicionar `tracing`, `tracing-subscriber` ao Cargo.toml
    - _Requirements: 4.1_
    - _Localização: `inference-pool/Cargo.toml`_
  - [ ] 6.2 Criar LoggingConfig e init_logging
    - Suportar níveis: ERROR, WARN, INFO, DEBUG, TRACE
    - Suportar formatos: JSON, Pretty
    - _Requirements: 4.1, 4.5, 4.6_
    - _Localização: `inference-pool/src/logging/mod.rs`_
  - [ ] 6.3 Adicionar logs estruturados em componentes
    - Registry: node_registered, node_deregistered
    - Router: job_routed, job_failed
    - Billing: ticket_created, payment_sent
    - Todos os erros com contexto
    - _Requirements: 4.2, 4.3, 4.4_
    - _Localização: Modificar componentes existentes_
  - [ ] 6.4 Integrar logging no startup
    - Inicializar logging antes de qualquer outra coisa
    - Logar configuração carregada (sem secrets)
    - _Requirements: 4.1_
    - _Localização: `inference-pool/src/main.rs`_

- [ ] 7. Checkpoint - Configuration and Logging Complete
  - Ensure configuration loads correctly
  - Verify env var overrides work
  - Test JSON and Pretty log formats
  - Ask the user if questions arise.

- [ ] 8. Implementar Health Check Service
  - [ ] 8.1 Criar estruturas de health check
    - `HealthStatus`, `HealthState`, `ComponentHealth`, `ComponentStatus`
    - _Requirements: 3.4_
    - _Localização: `inference-pool/src/health/mod.rs`_
  - [ ] 8.2 Implementar HealthCheckService
    - `check_health()` - verificar todos os componentes
    - `check_component()` - verificar componente específico
    - `register_check()` - registrar health check
    - _Requirements: 3.1, 3.2, 3.3_
    - _Localização: `inference-pool/src/health/service.rs`_
  - [ ] 8.3 Implementar health checks para componentes
    - Database: testar query simples
    - Lightning: verificar conexão com daemon
    - Registry: verificar se tem nodes ativos
    - _Requirements: 3.3_
    - _Localização: `inference-pool/src/health/checks.rs`_
  - [ ] 8.4 Adicionar endpoint GET /health
    - Retornar HTTP 200 se healthy, 503 se unhealthy
    - Incluir detalhes de todos os componentes
    - Responder em < 100ms
    - Não requerer autenticação
    - _Requirements: 3.1, 3.2, 3.5, 3.6_
    - _Localização: `inference-pool/src/api/health.rs`_
  - [ ] 8.5 Write property test for health check
    - **Property 37: Health Check Reflects Reality**
    - **Validates: Requirements 3.2, 3.3**
    - _Localização: `inference-pool/tests/property/health.rs`_

- [ ] 9. Implementar Startup Validation
  - [ ] 9.1 Criar StartupValidator
    - Validar configuração completa
    - Testar conexão com database
    - Testar conexão com Lightning
    - _Requirements: 7.1, 7.2, 7.3_
    - _Localização: `inference-pool/src/startup/validator.rs`_
  - [ ] 9.2 Integrar validação no startup
    - Executar todas as validações antes de iniciar API
    - Logar cada validação executada
    - Falhar rápido com erro claro se validação falha
    - Logar "Pool ready to accept traffic" quando pronto
    - _Requirements: 7.1, 7.4, 7.5_
    - _Localização: `inference-pool/src/main.rs`_

- [ ] 10. Implementar Shutdown Manager
  - [ ] 10.1 Criar ShutdownManager
    - `new()` - criar manager
    - `register()` - registrar componente
    - `listen_for_signals()` - escutar SIGTERM/SIGINT
    - `shutdown()` - executar shutdown ordenado
    - _Requirements: 5.1_
    - _Localização: `inference-pool/src/shutdown/mod.rs`_
  - [ ] 10.2 Implementar ShutdownHook trait
    - Implementar para API Server
    - Implementar para Channel Manager
    - Implementar para Billing System
    - Implementar para Database Pool
    - _Requirements: 5.2, 5.4, 5.5, 5.6_
    - _Localização: Modificar componentes existentes_
  - [ ] 10.3 Integrar shutdown no main
    - Registrar todos os componentes
    - Iniciar listener de sinais
    - Executar shutdown ao receber sinal
    - Logar cada etapa do shutdown
    - _Requirements: 5.1, 5.7_
    - _Localização: `inference-pool/src/main.rs`_
  - [ ] 10.4 Write property test for graceful shutdown
    - **Property 39: Graceful Shutdown Completes**
    - **Validates: Requirements 5.3, 5.5**
    - _Localização: `inference-pool/tests/property/shutdown.rs`_

- [ ] 11. Checkpoint - Observability and Lifecycle Complete
  - Ensure health checks work correctly
  - Verify startup validation catches errors
  - Test graceful shutdown sequence
  - Ask the user if questions arise.

- [ ] 12. Implementar Admin API
  - [ ] 12.1 Criar AdminApi struct e endpoints
    - Estruturas: `CreateKeyRequest`, `CreateKeyResponse`, `MemberDetail`, `PoolStats`
    - _Requirements: 8.2, 9.2_
    - _Localização: `inference-pool/src/admin/mod.rs`_
  - [ ] 12.2 Implementar API Key Management
    - `POST /v1/admin/keys/create` - criar API key
    - `POST /v1/admin/keys/revoke` - revogar key
    - `GET /v1/admin/keys` - listar keys
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
    - _Localização: `inference-pool/src/admin/keys.rs`_
  - [ ] 12.3 Implementar Member Management
    - `GET /v1/admin/members` - listar nodes
    - `GET /v1/admin/members/{node_id}` - detalhes do node
    - `POST /v1/admin/members/{node_id}/deregister` - desregistrar
    - `GET /v1/admin/stats` - estatísticas da pool
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
    - _Localização: `inference-pool/src/admin/members.rs`_
  - [ ] 12.4 Implementar autenticação de admin
    - Middleware que verifica admin token
    - Retornar 401 se token inválido
    - _Requirements: 8.6_
    - _Localização: `inference-pool/src/admin/auth.rs`_
  - [ ] 12.5 Adicionar endpoint GET /v1/admin/config
    - Retornar configuração atual (sem secrets)
    - _Requirements: 6.6_
    - _Localização: `inference-pool/src/admin/config.rs`_
  - [ ] 12.6 Write property test for admin auth
    - **Property 40: Admin API Requires Auth**
    - **Validates: Requirements 8.6**
    - _Localização: `inference-pool/tests/property/admin.rs`_

- [ ] 13. Checkpoint - Admin API Complete
  - Ensure admin endpoints work correctly
  - Verify authentication is enforced
  - Test key creation and revocation
  - Ask the user if questions arise.

- [ ] 14. Integração e Documentação
  - [ ] 14.1 Atualizar main.rs com todos os componentes
    - Carregar configuração
    - Inicializar logging
    - Conectar database e executar migrations
    - Validar startup
    - Iniciar health checks
    - Iniciar API server
    - Registrar shutdown hooks
    - _Requirements: All_
    - _Localização: `inference-pool/src/main.rs`_
  - [ ] 14.2 Criar README.md completo
    - Quick start
    - Configuração (arquivo + env vars)
    - Como nodes se registram
    - Como clientes usam
    - Admin API
    - Health checks
    - Troubleshooting
    - _Requirements: 10.5_
    - _Localização: `inference-pool/README.md`_
  - [ ] 14.3 Criar docker-compose.yml
    - Serviço pool
    - Serviço PostgreSQL (opcional)
    - Serviço LND (mock para testes)
    - Volumes para persistência
    - _Localização: `inference-pool/docker-compose.yml`_
  - [ ] 14.4 Write integration tests
    - Full startup com database
    - Health check com falhas simuladas
    - Graceful shutdown com requests em andamento
    - Admin API end-to-end
    - Configuration loading
    - _Localização: `inference-pool/tests/integration/`_

- [ ] 15. Checkpoint Final
  - Ensure all tests pass
  - Verify pool starts and shuts down cleanly
  - Test with SQLite and PostgreSQL
  - Verify README is complete and accurate
  - Ask the user if questions arise.

## Notes

- Todos os testes são obrigatórios para garantir qualidade
- Cada task referencia requisitos específicos para rastreabilidade
- Checkpoints garantem validação incremental
- Property tests validam propriedades universais de corretude
- A implementação usa Rust com sqlx para database
- Suporta SQLite (desenvolvimento) e PostgreSQL (produção)
- Structured logging usa tracing crate
- Configuration suporta arquivo TOML + env vars
- Admin API requer token de autenticação
- **Pré-requisito**: Spec `api-integration` deve estar implementada
- **Objetivo**: Tornar pool operável em produção
- **Total de Tasks:** 15
- **Total de Properties:** 6 (35-40)
- **Total de Requirements:** 10
- **Ordem de Implementação:**
  1. Tasks 1-3: Database Layer (persistência básica)
  2. Task 4: Integrar database com componentes existentes
  3. Tasks 5-7: Configuration e Logging
  4. Tasks 8-11: Observability e Lifecycle (health, startup, shutdown)
  5. Tasks 12-13: Admin API
  6. Tasks 14-15: Integration e Documentation

