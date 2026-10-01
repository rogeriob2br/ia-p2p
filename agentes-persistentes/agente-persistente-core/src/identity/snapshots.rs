use serde::{Deserialize, Serialize};

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
