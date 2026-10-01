use serde::{Deserialize, Serialize};
use chrono::Local;

pub mod extractor;
pub mod merge;

/// Registro histórico de alteração na confiança ou razão de uma crença.
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

/// Hipótese sustentada por evidências com classificação por Sujeito (SELF, USER, WORLD).
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
