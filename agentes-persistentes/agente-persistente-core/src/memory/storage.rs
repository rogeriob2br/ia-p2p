use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::beliefs::Belief;
use crate::identity::snapshots::IdentitySnapshot;
use crate::metabolism::budget::MetabolicState;
use crate::metabolism::clock::LifeCycleMetrics;
use crate::values::commitments::Commitment;
use crate::memory::episodic::EpisodicLog;
use crate::memory::semantic::SemanticMemory;
use crate::memory::vector_store::VectorMemoryStore;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory {
    pub world_model: String,
    pub semantic: SemanticMemory,
    pub episodic: EpisodicLog,
    #[serde(default)]
    pub interaction_count: u32,
    #[serde(default)]
    pub beliefs: Vec<Belief>,
    #[serde(default)]
    pub snapshots: Vec<IdentitySnapshot>,
    #[serde(default)]
    pub lifecycle: LifeCycleMetrics,
    #[serde(default)]
    pub metabolism: MetabolicState,
    #[serde(default)]
    pub commitments: Vec<Commitment>,
    #[serde(default)]
    pub vector_store: VectorMemoryStore,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            world_model: "Terminal local. Engine oficial em Rust conectado ao Ollama.".to_string(),
            semantic: SemanticMemory::new(),
            episodic: EpisodicLog::new(),
            interaction_count: 0,
            beliefs: Vec::new(),
            snapshots: Vec::new(),
            lifecycle: LifeCycleMetrics::new(),
            metabolism: MetabolicState::new(),
            commitments: Vec::new(),
            vector_store: VectorMemoryStore::new(),
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

            // Auto-migração de crenças 'UNKNOWN'
            let mut migrated = false;
            for b in &mut mem.beliefs {
                if b.subject == "UNKNOWN" || b.subject.is_empty() {
                    b.subject = "SELF".to_string();
                    migrated = true;
                }
            }
            if migrated {
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
        let data = serde_json::to_string_pretty(self).expect("Erro ao serializar memória");
        fs::write(path, data).expect("Erro ao salvar arquivo de memória");
    }

    pub fn add_episode(&mut self, text: String, is_success: bool, is_salient: bool) {
        if is_salient {
            self.episodic.add(text);
        }
        self.lifecycle.register_interaction(is_success);
        self.interaction_count += 1;
    }

    pub fn clear_episodes_and_update_semantic(&mut self, new_semantic_content: String) {
        self.episodic.clear();
        self.semantic.update(new_semantic_content);
        self.metabolism.recover_from_sleep();
        self.lifecycle.register_sleep();
    }

    pub fn format_beliefs_for_subject(&self, subject: &str) -> String {
        let filtered: Vec<&Belief> = self.beliefs.iter().filter(|b| b.subject == subject).collect();
        if filtered.is_empty() {
            format!("Nenhuma crença registrada sobre {}.", subject)
        } else {
            filtered
                .iter()
                .map(|b| format!("- {} (confiança: {:.2})", b.statement, b.confidence))
                .collect::<Vec<String>>()
                .join("\n")
        }
    }

    pub fn format_commitments(&self) -> String {
        if self.commitments.is_empty() {
            "Nenhum dever ou papel social emergido ainda.".to_string()
        } else {
            self.commitments
                .iter()
                .map(|c| format!("• [{}] {} (prioridade: {})", c.status, c.description, c.priority))
                .collect::<Vec<String>>()
                .join("\n")
        }
    }
}
