use std::io::{self, Write};
use chrono::Local;
use colored::*;
use reqwest::Client;

use agente_persistente_core::{
    Memory, HybridAgentProvider, CognitiveTier, MetacognitionEngine, IdentityNarrativeEngine, CommitmentExtractor, SpeechSynthesizer, VoiceActivityListener
};
use agente_persistente_core::beliefs::extractor::BeliefExtractor;
use agente_persistente_core::beliefs::merge::merge_extracted_beliefs;
use agente_persistente_core::memory::vector_store::{generate_embedding, VectorEntry};

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

/// Executa o Ciclo de Sono Autônomo em 3 Fases (Subconsciente)
async fn run_sleep_cycle(
    mem: &mut Memory,
    extractor: &BeliefExtractor,
    commitment_extractor: &CommitmentExtractor,
    provider: &HybridAgentProvider,
    narrative: &IdentityNarrativeEngine,
    memory_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "🌙 [SONO AUTÔNOMO METABÓLICO] Restauração homeostática em andamento...".magenta().bold());

    // Fase 1: Extrair crenças e deveres
    if !mem.episodic.episodes.is_empty() {
        if let Ok(extracted) = extractor.extract_from_episodes(&mem.episodic.episodes).await {
            merge_extracted_beliefs(&mut mem.beliefs, extracted);
        }
        if let Ok(extracted_commitments) = commitment_extractor.extract_commitments_from_episodes(&mem.episodic.episodes).await {
            CommitmentExtractor::merge_commitments(&mut mem.commitments, extracted_commitments);
        }
    }

    // Fase 2: Consolidação Semântica
    if let Ok(new_semantic) = provider.consolidate_memories(&mem).await {
        mem.clear_episodes_and_update_semantic(new_semantic);
    }

    // Fase 3: Snapshot de Identidade
    if let Ok(snapshot) = narrative.create_snapshot(&mem.beliefs, mem.interaction_count, mem.snapshots.len()).await {
        mem.snapshots.push(snapshot);
    }

    // Restauração Metabólica
    mem.metabolism.recover_from_sleep();
    mem.save(memory_file);
    println!("  {}", "★ [RESTAURAÇÃO METABÓLICA CONCLUÍDA] Agente renovado.".yellow().bold());

    Ok(())
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    let lower = text.to_lowercase();
    keywords.iter().any(|k| lower.contains(k))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_start_time = Local::now().naive_local();
    let http_client = Client::new();
    let speech = SpeechSynthesizer::new();
    let listener = VoiceActivityListener::new();

    let is_interactive = atty::is(atty::Stream::Stdin);
    let mut is_standby = false;

    println!("{}", "╔════════════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║   AGENTE COGNITIVO PERSISTENTE — CORE ENGINE v5.1 (WAKE WORD / VAD)║".cyan().bold());
    println!("{}", "║  Modo Descaso | Palavra de Ativação | Voz Neural HD | Daemon      ║".cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════════════╝".cyan());
    println!("Iniciando Entidade Cognitiva...\n");

    let memory_file = "agent_memory.json";
    let mut mem = Memory::load_or_create(memory_file);

    let ollama_model = "llama3";
    let provider = HybridAgentProvider::new(ollama_model);
    let metacognition = MetacognitionEngine::new(ollama_model);
    let extractor = BeliefExtractor::new(ollama_model);
    let commitment_extractor = CommitmentExtractor::new(ollama_model);
    let narrative = IdentityNarrativeEngine::new(ollama_model);

    println!(
        "{}",
        format!(
            "Memórias Vetoriais: {} registros. Modo: {}.",
            mem.vector_store.entries.len(),
            if is_interactive { "Interativo (terminal + voz)" } else { "Daemon (voz contínua + wake word)" }
        )
        .green()
    );

    if is_interactive {
        println!("{}", "Modo Interativo: digite texto OU pressione ENTER para escuta contínua por voz.".yellow());
        println!("{}", "Comandos de Voz: diga 'descanse' para pausar, e 'acorde' para reativar.\n".yellow());
    } else {
        println!("{}", "Modo Daemon: Escuta contínua por voz ativa. Diga 'descanse' para pausar e 'acorde' para reativar.\n".yellow());
    }

    loop {
        // --- Verificação de Sono Autônomo Homeostático Subconsciente ---
        if mem.metabolism.needs_autonomous_sleep(mem.episodic.episodes.len()) {
            if let Err(e) = run_sleep_cycle(&mut mem, &extractor, &commitment_extractor, &provider, &narrative, memory_file).await {
                println!("{}", format!("Erro ao executar sono autônomo: {}", e).red());
            }
        }

        // --- Captura de Entrada (Teclado ou Voz Contínua) ---
        let user_text = if is_interactive {
            print!("{}", "Você: ".green().bold());
            io::stdout().flush()?;

            let mut raw_input = String::new();
            io::stdin().read_line(&mut raw_input)?;
            let raw_input = raw_input.trim().to_string();

            if raw_input.eq_ignore_ascii_case("sair") {
                println!("{}", "Agente: Até logo. Salvando estado...".cyan());
                speech.speak("Até logo. Salvando meu estado completo.");
                mem.save(memory_file);
                break;
            }

            if raw_input.is_empty() || raw_input.eq_ignore_ascii_case("voz") || raw_input.eq_ignore_ascii_case("falar") {
                match listener.listen_continuous() {
                    Ok(text) => {
                        println!("{} \"{}\"", "Você (voz):".green().bold(), text);
                        text
                    }
                    Err(e) => {
                        println!("{}", format!("⚠ {}", e).yellow());
                        continue;
                    }
                }
            } else {
                raw_input
            }
        } else {
            // Modo Daemon: escuta contínua VAD
            match listener.listen_continuous() {
                Ok(text) => {
                    println!("{} \"{}\"", "Você (voz):".green().bold(), text);
                    text
                }
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            }
        };

        // --- LÓGICA DE MODO DESCASO & PALAVRA DE ATIVAÇÃO (WAKE WORD) ---
        let sleep_keywords = &["descanse", "silêncio", "silencio", "pause", "descaso", "pare de ouvir", "dormir", "modo descaso", "fica quieto", "não fale", "nao fale"];
        let wake_keywords = &["acorde", "acorda", "agente", "ei agente", "está aí", "esta ai", "voltar", "fale comigo", "olá", "ola"];

        if is_standby {
            // No modo descaso, ignora tudo que não seja uma palavra de ativação
            if contains_any(&user_text, wake_keywords) {
                is_standby = false;
                println!("{}", "🌙 -> ☀️ [MODO ATIVO REATIVADO] O agente foi acordado!".yellow().bold());
                speech.speak("Estou ouvindo. Pode falar.");
            } else {
                println!("{}", "💤 [MODO DESCASO ATIVO] Áudio ignorado. Aguardando 'acorde' ou 'ei agente'...".dimmed());
                continue;
            }
        } else {
            // Se o usuário pedir para o agente descansar/pausar
            if contains_any(&user_text, sleep_keywords) {
                is_standby = true;
                println!("{}", "☀️ -> 🌙 [MODO DESCASO ENTRANDO] Agente em pausa.".magenta().bold());
                speech.speak("Entrando em modo descaso. Ficarei em silêncio até você dizer acorde ou ei agente.");
                continue;
            }
        }

        // --- Ciclo Cognitivo Fluido (Propriocepção & Memória Vetorial) ---

        let attention = metacognition
            .analyze_attention(&user_text)
            .await
            .unwrap_or_else(|_| agente_persistente_core::AttentionAnalysis {
                intent: "OUTRO".to_string(),
                salience_score: 0.5,
                is_salient: true,
            });

        let is_cloud_upgrade = provider.should_upgrade_to_cloud(&attention.intent, attention.salience_score, mem.metabolism.energy_credits, &mem.metabolism.mode);
        if is_cloud_upgrade {
            if !mem.metabolism.consume_cloud_think() {
                if !mem.metabolism.consume_think() {
                    println!("{}", "⚠ [EXAUSTÃO METABÓLICA] Sem energia. O agente precisa descansar.".red());
                    continue;
                }
            }
        } else {
            if !mem.metabolism.consume_think() {
                println!("{}", "⚠ [EXAUSTÃO METABÓLICA] Sem energia. O agente precisa descansar.".red());
                continue;
            }
        }

        let uptime = get_session_uptime(session_start_time);
        match provider.think_hybrid(&user_text, &mem, &attention.intent, attention.salience_score, &uptime).await {
            Ok((resposta, tier_used)) => {
                let tier_tag = if tier_used == CognitiveTier::CloudPaid { "[Nuvem]".yellow() } else { "[Local]".blue() };
                println!("\n{} {}: {}\n", "Agente".cyan().bold(), tier_tag, resposta);

                speech.speak(&resposta);

                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let resposta_resumida: String = resposta.chars().take(300).collect();
                let episode = format!(
                    "[{}] [intent:{} | tier:{:?}] Usuário: '{}' → Agente: '{}'",
                    timestamp, attention.intent, tier_used, user_text, resposta_resumida
                );
                mem.add_episode(episode.clone(), true, attention.is_salient);

                if attention.is_salient {
                    if let Ok(vec_emb) = generate_embedding(&http_client, ollama_model, &episode).await {
                        let entry = VectorEntry {
                            id: format!("ep-{}", mem.interaction_count),
                            text: episode,
                            vector: vec_emb,
                            category: attention.intent.clone(),
                            timestamp: timestamp.clone(),
                        };
                        mem.vector_store.add_entry(entry);
                    }
                }

                mem.save(memory_file);
            }
            Err(e) => {
                mem.lifecycle.register_interaction(false);
                println!("{}", format!("Erro na resposta: {}", e).red());
            }
        }
    }

    Ok(())
}
