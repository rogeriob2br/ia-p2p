use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::Local;

// ---------------------------------------------------------------------------
// Estruturas do Sistema de Crenças, Identidade Temporal e Relógio Interno
// ---------------------------------------------------------------------------

/// Registro histórico de uma alteração em uma crença.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeliefChange {
    pub timestamp: String,
    pub old_confidence: f32,
    pub new_confidence: f32,
    pub reason: String,
}

fn default_subject() -> String {
    "UNKNOWN".to_string()
}

/// Uma crença do agente — hipótese sustentada por evidências, com sujeito explícito (SELF, USER, WORLD).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Belief {
    pub id: String,
    #[serde(default = "default_subject")]
    pub subject: String, // "SELF" | "USER" | "WORLD" | "UNKNOWN"
    pub statement: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub created_at: String,
    pub last_reviewed: String,
    pub change_history: Vec<BeliefChange>,
}

impl Belief {
    pub fn new(id: String, subject: String, statement: String, confidence: f32, evidence: Vec<String>) -> Self {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Belief {
            id,
            subject,
            statement,
            confidence,
            evidence,
            created_at: now.clone(),
            last_reviewed: now,
            change_history: Vec::new(),
        }
    }

    /// Atualiza a crença com nova confiança e evidências, registrando a mudança.
    pub fn update(&mut self, new_confidence: f32, new_evidence: Vec<String>, reason: String) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.change_history.push(BeliefChange {
            timestamp: now.clone(),
            old_confidence: self.confidence,
            new_confidence,
            reason,
        });
        self.confidence = new_confidence;
        for ev in new_evidence {
            if !self.evidence.contains(&ev) {
                self.evidence.push(ev);
            }
        }
        self.last_reviewed = now;
    }
}

/// Snapshot imutável da identidade em um determinado instante (Ciclo de Sono / Versão)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentitySnapshot {
    pub version: u32,
    pub timestamp: String,
    pub interaction_count: u32,
    pub self_hypothesis: String,
    pub user_hypothesis: String,
    pub beliefs_count: usize,
}

/// Métricas do Ciclo de Vida e Relógio Interno (Etapa 6)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LifeCycleMetrics {
    pub birth_timestamp: String,
    pub sleep_cycles_count: u32,
    pub total_conversations: u32,
    pub successful_actions: u32,
    pub failed_actions: u32,
    pub last_interaction_timestamp: String,
}

fn default_lifecycle() -> LifeCycleMetrics {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    LifeCycleMetrics {
        birth_timestamp: now.clone(),
        sleep_cycles_count: 0,
        total_conversations: 0,
        successful_actions: 0,
        failed_actions: 0,
        last_interaction_timestamp: now,
    }
}

// ---------------------------------------------------------------------------
// Estrutura principal de Memória
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory {
    /// Compreensão do ambiente onde o agente opera.
    pub world_model: String,
    /// Fatos consolidados sobre o usuário e o mundo (gerado por consolidação).
    #[serde(default = "default_semantic_memory")]
    pub semantic_memory: String,
    /// Log bruto de episódios recentes (limpo após cada sono).
    pub episodic_log: Vec<String>,
    /// Contador total de turnos de interação — percepção de "passagem do tempo".
    #[serde(default)]
    pub interaction_count: u32,
    /// Sistema de Crenças: hipóteses categorizadas por sujeito (SELF, USER, WORLD).
    #[serde(default)]
    pub beliefs: Vec<Belief>,
    /// Histórico de Snapshots Cronológicos de Identidade (Etapa 5)
    #[serde(default)]
    pub snapshots: Vec<IdentitySnapshot>,
    /// Métricas do Relógio Interno e Ciclo de Vida (Etapa 6)
    #[serde(default = "default_lifecycle")]
    pub lifecycle: LifeCycleMetrics,
}

fn default_semantic_memory() -> String {
    "Nenhum conhecimento consolidado ainda.".to_string()
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            world_model: "Terminal local. PoC em Rust conectado ao Ollama. Agente em fase inicial de desenvolvimento.".to_string(),
            semantic_memory: default_semantic_memory(),
            episodic_log: Vec::new(),
            interaction_count: 0,
            beliefs: Vec::new(),
            snapshots: Vec::new(),
            lifecycle: default_lifecycle(),
        }
    }

    pub fn load_or_create(path: &str) -> Self {
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).expect("Erro ao ler arquivo de memória");
            let mut mem: Memory = serde_json::from_str(&data).unwrap_or_else(|_| {
                println!("[AVISO] Arquivo de memória com schema incompatível. Iniciando novo.");
                let mem = Memory::new();
                mem.save(path);
                mem
            });

            // Migração automática de crenças salvas anteriormente sem subject ("UNKNOWN")
            let mut migrated = false;
            for b in &mut mem.beliefs {
                if b.subject == "UNKNOWN" || b.subject.is_empty() {
                    let text = format!("{} {:?}", b.statement, b.evidence).to_lowercase();
                    if text.contains("rogério") || text.contains("usuário") || text.contains("desenvolvedor") || text.contains("construindo") {
                        b.subject = "USER".to_string();
                    } else {
                        b.subject = "SELF".to_string();
                    }
                    migrated = true;
                }
            }

            if migrated {
                println!("[MIGRAÇÃO AUTOMÁTICA] Crenças anteriores com assunto 'UNKNOWN' foram reclassificadas (USER/SELF).");
                mem.save(path);
            }

            mem
        } else {
            let mem = Memory::new();
            mem.save(path);
            mem
        }
    }

    pub fn save(&self, path: &str) {
        let json = serde_json::to_string_pretty(self).expect("Erro ao serializar memória");
        fs::write(path, json).expect("Erro ao salvar arquivo de memória");
    }

    pub fn add_episode(&mut self, episode: String) {
        self.episodic_log.push(episode);
        self.interaction_count += 1;
        self.lifecycle.total_conversations += 1;
        self.lifecycle.last_interaction_timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    pub fn clear_episodes_and_update_semantic(&mut self, new_semantic: String) {
        self.semantic_memory = new_semantic;
        self.episodic_log.clear();
    }

    pub fn add_snapshot(&mut self, snapshot: IdentitySnapshot) {
        self.snapshots.push(snapshot);
    }

    pub fn latest_snapshot(&self) -> Option<&IdentitySnapshot> {
        self.snapshots.last()
    }

    /// Retorna crenças filtradas por sujeito e limiar de confiança.
    pub fn get_beliefs_by_subject(&self, subject: &str, threshold: f32) -> Vec<&Belief> {
        self.beliefs
            .iter()
            .filter(|b| b.subject.eq_ignore_ascii_case(subject) && b.confidence >= threshold)
            .collect()
    }

    /// Formata crenças de um determinado sujeito para injeção no LLM.
    pub fn format_beliefs_for_subject(&self, subject: &str) -> String {
        let list = self.get_beliefs_by_subject(subject, 0.4);
        if list.is_empty() {
            return format!("Nenhuma hipótese de alta confiança sobre {} ainda.", subject);
        }
        list.iter()
            .map(|b| format!(
                "• [conf: {:.2}] {} — {} evidência(s)",
                b.confidence, b.statement, b.evidence.len()
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Calcula o tempo decorrido desde a última interação em formato legível.
    pub fn get_elapsed_since_last_interaction(&self) -> String {
        let fmt = "%Y-%m-%d %H:%M:%S";
        if let Ok(last) = chrono::NaiveDateTime::parse_from_str(&self.lifecycle.last_interaction_timestamp, fmt) {
            let now = Local::now().naive_local();
            let duration = now.signed_duration_since(last);
            let mins = duration.num_minutes();
            let hours = duration.num_hours();
            let days = duration.num_days();
            if days > 0 {
                format!("{} dia(s) e {} hora(s)", days, hours % 24)
            } else if hours > 0 {
                format!("{} hora(s) e {} minuto(s)", hours, mins % 60)
            } else if mins > 0 {
                format!("{} minuto(s)", mins)
            } else {
                format!("{} segundo(s)", duration.num_seconds().max(0))
            }
        } else {
            "pouco tempo".to_string()
        }
    }

    /// Calcula a idade total do agente desde seu nascimento em formato legível.
    pub fn get_total_age(&self) -> String {
        let fmt = "%Y-%m-%d %H:%M:%S";
        if let Ok(birth) = chrono::NaiveDateTime::parse_from_str(&self.lifecycle.birth_timestamp, fmt) {
            let now = Local::now().naive_local();
            let duration = now.signed_duration_since(birth);
            let mins = duration.num_minutes();
            let hours = duration.num_hours();
            let days = duration.num_days();
            if days > 0 {
                format!("{} dia(s), {} hora(s) e {} minuto(s)", days, hours % 24, mins % 60)
            } else if hours > 0 {
                format!("{} hora(s) e {} minuto(s)", hours, mins % 60)
            } else if mins > 0 {
                format!("{} minuto(s)", mins)
            } else {
                format!("{} segundo(s)", duration.num_seconds().max(0))
            }
        } else {
            "pouco tempo".to_string()
        }
    }
}
