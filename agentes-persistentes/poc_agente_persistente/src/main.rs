mod memory;
mod agent;
mod belief_engine;

use memory::Memory;
use agent::Agent;
use belief_engine::BeliefEngine;
use std::io::{self, Write};
use chrono::Local;

fn get_session_uptime(start_time: chrono::NaiveDateTime) -> String {
    let now = Local::now().naive_local();
    let duration = now.signed_duration_since(start_time);
    let mins = duration.num_minutes();
    let secs = duration.num_seconds() % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs.max(0))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_start_time = Local::now().naive_local();

    println!("╔════════════════════════════════════════════════════╗");
    println!("║   Agente Cognitivo Persistente — PoC v0.7         ║");
    println!("║   Etapa 6: Relógio Interno & Ciclo de Vida         ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!("Carregando Memória...\n");

    let memory_file = "agent_memory.json";
    let mut mem = Memory::load_or_create(memory_file);

    let ollama_model = "llama3";
    let agent = Agent::new(ollama_model);
    let belief_engine = BeliefEngine::new(ollama_model);

    let elapsed_since_last = mem.get_elapsed_since_last_interaction();

    println!(
        "Agente acordou | Modelo: {} | Nascimento: {}\nCiclos de sono: {} | Interações: {} | Desde último papo: {}\n",
        ollama_model, mem.lifecycle.birth_timestamp, mem.lifecycle.sleep_cycles_count, mem.interaction_count, elapsed_since_last
    );
    println!("Comandos especiais:");
    println!("  'relogio'    → Dashboard do Relógio Interno e Métricas do Ciclo de Vida");
    println!("  'dormir'     → Consolidar memórias, extrair crenças e registrar Snapshot (vN)");
    println!("  'historico'  → Linha do tempo de Snapshots e Análise de Evolução Temporal");
    println!("  'identidade' → Ver Modelo de Si Mesmo (Self Model atual)");
    println!("  'criador'    → Ver Modelo do Usuário/Criador (User Model atual)");
    println!("  'crenças'    → Listar crenças detalhadas (SELF, USER, WORLD)");
    println!("  'sair'       → Salvar e encerrar\n");

    loop {
        print!("Você: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // --- Comandos Especiais ---

        if input.eq_ignore_ascii_case("sair") {
            println!("Agente: Até logo. Salvando estado...");
            mem.save(memory_file);
            break;
        }

        // Dashboard do Relógio Interno e Ciclo de Vida (Etapa 6)
        if input.eq_ignore_ascii_case("relogio") || input.eq_ignore_ascii_case("tempo") {
            let uptime = get_session_uptime(session_start_time);
            let time_since_last = mem.get_elapsed_since_last_interaction();
            println!("\n╔════════════════════════════════════════════════════════╗");
            println!("║      DASHBOARD DO RELÓGIO INTERNO & CICLO DE VIDA      ║");
            println!("╠════════════════════════════════════════════════════════╣");
            println!("  • Data de Nascimento:       {}", mem.lifecycle.birth_timestamp);
            println!("  • Tempo Acordado na Sessão: {}", uptime);
            println!("  • Tempo Desde Último Papo:  {}", time_since_last);
            println!("  • Ciclos de Sono (Dormir):  {}", mem.lifecycle.sleep_cycles_count);
            println!("  • Total de Conversas:       {}", mem.lifecycle.total_conversations);
            println!("  • Sucessos / Falhas:        {} / {}", mem.lifecycle.successful_actions, mem.lifecycle.failed_actions);
            println!("  • Snapshots Registrados:    {}", mem.snapshots.len());
            println!("  • Crenças Formadas:         {}", mem.beliefs.len());
            println!("╚════════════════════════════════════════════════════════╝\n");
            continue;
        }

        // Exibe crenças agrupadas e com tags de sujeito
        if input.eq_ignore_ascii_case("crenças") {
            println!("\n[SISTEMA DE CRENÇAS ATUAL]");
            println!("Total: {} crença(s)\n", mem.beliefs.len());
            if mem.beliefs.is_empty() {
                println!("  Nenhuma crença formada ainda.");
                println!("  Converse e depois use 'dormir' para extrair hipóteses.\n");
            } else {
                for b in &mem.beliefs {
                    println!("  [{}] [{}] confiança: {:.2}", b.id, b.subject, b.confidence);
                    println!("  Hipótese: {}", b.statement);
                    println!("  Evidências: {} | Criada: {}", b.evidence.len(), b.created_at);
                    if !b.evidence.is_empty() {
                        println!("  Evidência: \"{}\"",
                            b.evidence.last().unwrap_or(&String::new()).chars().take(80).collect::<String>());
                    }
                    println!();
                }
            }
            continue;
        }

        // Modelo de Si Mesmo (SELF)
        if input.eq_ignore_ascii_case("identidade") {
            println!("\n[MODELO DE SI MESMO (SELF MODEL) — Sintetizado sob demanda]");
            println!("Sintetizando crenças do tipo [SELF]...\n");
            match belief_engine.synthesize_self_model(&mem).await {
                Ok(hypothesis) => println!("{}\n", hypothesis),
                Err(e) => println!("Erro ao sintetizar: {}\n", e),
            }
            println!("[CRENÇAS SOBRE O AGENTE (SELF)]");
            println!("{}\n", mem.format_beliefs_for_subject("SELF"));
            continue;
        }

        // Modelo do Criador (USER)
        if input.eq_ignore_ascii_case("criador") || input.eq_ignore_ascii_case("usuario") {
            println!("\n[MODELO DO CRIADOR/USUÁRIO (USER MODEL) — Sintetizado sob demanda]");
            println!("Sintetizando crenças do tipo [USER]...\n");
            match belief_engine.synthesize_user_model(&mem).await {
                Ok(hypothesis) => println!("{}\n", hypothesis),
                Err(e) => println!("Erro ao sintetizar: {}\n", e),
            }
            println!("[CRENÇAS SOBRE O USUÁRIO (USER)]");
            println!("{}\n", mem.format_beliefs_for_subject("USER"));
            continue;
        }

        // Histórico Cronológico e Comparação de Snapshots de Identidade (Etapa 5)
        if input.eq_ignore_ascii_case("historico") || input.eq_ignore_ascii_case("snapshots") {
            println!("\n[IDENTIDADE TEMPORAL — Linha do Tempo de Snapshots]");
            println!("Total de versões registradas: {}\n", mem.snapshots.len());

            if mem.snapshots.is_empty() {
                println!("  Nenhum snapshot de identidade foi registrado ainda.");
                println!("  Use o comando 'dormir' após conversar para gerar o primeiro snapshot (v1).\n");
            } else {
                for snap in &mem.snapshots {
                    println!("  📸 Snapshot v{} | Interação #{} | {}", snap.version, snap.interaction_count, snap.timestamp);
                    println!("     Crenças ativas: {}", snap.beliefs_count);
                    println!("     Hipótese de Si: \"{}\"", snap.self_hypothesis);
                    println!("     Hipótese do Usuário: \"{}\"", snap.user_hypothesis);
                    println!();
                }

                if mem.snapshots.len() >= 2 {
                    let first = &mem.snapshots[0];
                    let latest = mem.snapshots.last().unwrap();
                    println!("  ⟳ Analisando evolução narrativa entre v{} e v{}...\n", first.version, latest.version);
                    match belief_engine.compare_snapshots(first, latest).await {
                        Ok(narrativa) => {
                            println!("  [NARRATIVA AUTOBIOGRÁFICA DE EVOLUÇÃO]");
                            println!("  {}\n", narrativa);
                        }
                        Err(e) => println!("  Erro ao gerar narrativa: {}\n", e),
                    }
                }
            }
            continue;
        }

        // Ciclo de Sono em 3 Fases (Extração -> Consolidação -> Snapshot Temporal)
        if input.eq_ignore_ascii_case("dormir") {
            println!("\nAgente em MODO DE SONO...\n");
            mem.lifecycle.sleep_cycles_count += 1;

            // Fase 1: Extrair crenças classificadas
            println!("  ⟳ Fase 1/3: Extraindo hipóteses classificadas (SELF / USER / WORLD)...");
            if mem.episodic_log.is_empty() {
                println!("  ⚠ Nenhum episódio novo para processar.");
            } else {
                match belief_engine.extract_beliefs_from_episodes(&mem).await {
                    Ok(extracted) => {
                        let count = extracted.len();
                        belief_engine.merge_beliefs(&mut mem, extracted);
                        println!("  ✓ {} hipótese(s) extraída(s) → {} crença(s) no total.", count, mem.beliefs.len());
                    }
                    Err(e) => println!("  ✗ Erro na extração de crenças: {}", e),
                }
            }

            // Fase 2: Consolidar episódios em memória semântica
            println!("  ⟳ Fase 2/3: Consolidando episódios em memória semântica...");
            match agent.consolidate_memories(&mem).await {
                Ok(new_semantic) => {
                    mem.clear_episodes_and_update_semantic(new_semantic);
                    println!("  ✓ Memória semântica atualizada. Log de episódios limpo.");
                }
                Err(e) => println!("  ✗ Erro na consolidação: {}", e),
            }

            // Fase 3: Registrar Snapshot de Identidade Temporal (vN)
            let version = (mem.snapshots.len() as u32) + 1;
            println!("  ⟳ Fase 3/3: Gerando Snapshot de Identidade Temporal (v{})...", version);
            match belief_engine.create_snapshot(&mem).await {
                Ok(snapshot) => {
                    mem.add_snapshot(snapshot);
                    println!("  ✓ Snapshot v{} registrado com sucesso.", version);
                }
                Err(e) => println!("  ✗ Erro ao gerar snapshot: {}", e),
            }

            mem.save(memory_file);
            println!("\n  Agente acordou (Ciclo de sono #{} concluído). Estado e Snapshot v{} salvos.\n", mem.lifecycle.sleep_cycles_count, version);
            continue;
        }

        if input.is_empty() {
            continue;
        }

        // --- Ciclo Cognitivo Normal ---

        let intent = agent
            .metacognitive_check(input)
            .await
            .unwrap_or_else(|_| "OUTRO".to_string());
        println!("[meta: {}] Agente pensando...", intent);

        let uptime = get_session_uptime(session_start_time);
        match agent.think(input, &mem, &intent, &uptime).await {
            Ok(resposta) => {
                println!("\nAgente: {}\n", resposta);
                mem.lifecycle.successful_actions += 1;

                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let resposta_resumida: String = resposta.chars().take(300).collect();
                let episode = format!(
                    "[{}] [intent:{}] Usuário: '{}' → Agente: '{}'",
                    timestamp, intent, input, resposta_resumida
                );
                mem.add_episode(episode);
                mem.save(memory_file);
            }
            Err(e) => {
                mem.lifecycle.failed_actions += 1;
                println!("Erro ao chamar Ollama: {}", e);
                println!("Verifique se o serviço está rodando com: ollama serve\n");
            }
        }
    }

    Ok(())
}
