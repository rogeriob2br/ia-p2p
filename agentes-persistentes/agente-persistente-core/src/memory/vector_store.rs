use serde::{Deserialize, Serialize};
use reqwest::Client;
use serde_json::json;

/// Entrada em Tensor Vetorial da Memória Episódica/Semântica
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VectorEntry {
    pub id: String,
    pub text: String,
    pub vector: Vec<f32>,
    pub category: String,
    pub timestamp: String,
}

/// Banco de Dados Vetorial Embarcado Nativo em Rust (Tensor Memory Store)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VectorMemoryStore {
    pub entries: Vec<VectorEntry>,
}

impl VectorMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adiciona uma nova memória convertida em Tensor Vetorial
    pub fn add_entry(&mut self, entry: VectorEntry) {
        self.entries.push(entry);
    }

    /// Calcula a Similaridade de Cosseno entre dois tensores vetoriais: cos(θ) = (A · B) / (||A|| * ||B||)
    pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
        if v1.len() != v2.len() || v1.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
        let norm_b: f32 = v2.iter().map(|b| b * b).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Pesquisa os Top-K episódios/fatos mais semanticamente semelhantes ao vetor query
    pub fn search_top_k(&self, query_vector: &[f32], top_k: usize) -> Vec<(f32, &VectorEntry)> {
        if query_vector.is_empty() || self.entries.is_empty() {
            return Vec::new();
        }

        let mut scored_entries: Vec<(f32, &VectorEntry)> = self
            .entries
            .iter()
            .map(|entry| {
                let sim = Self::cosine_similarity(query_vector, &entry.vector);
                (sim, entry)
            })
            .collect();

        // Ordena do mais semelhante (maior pontuação de cosseno) para o menos semelhante
        scored_entries.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored_entries.truncate(top_k);
        scored_entries
    }
}

/// Helper para gerar embeddings vetoriais de texto usando o endpoint local do Ollama (`/api/embeddings`)
pub async fn generate_embedding(
    client: &Client,
    ollama_model: &str,
    text: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let url = "http://localhost:11434/api/embeddings";
    let payload = json!({
        "model": ollama_model,
        "prompt": text
    });

    let resp = client.post(url).json(&payload).send().await?;
    let json_resp: serde_json::Value = resp.json().await?;

    if let Some(embedding_arr) = json_resp["embedding"].as_array() {
        let vec_f32: Vec<f32> = embedding_arr
            .iter()
            .filter_map(|val| val.as_f64().map(|v| v as f32))
            .collect();
        Ok(vec_f32)
    } else {
        Err("Não foi possível extrair o vetor de embedding do Ollama.".into())
    }
}
