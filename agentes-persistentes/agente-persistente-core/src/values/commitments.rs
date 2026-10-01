use serde::{Deserialize, Serialize};
use reqwest::Client;
use serde_json::json;
use chrono::Local;

/// Um compromisso ou papel emergido da experiência (Nível 3 da Pirâmide)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commitment {
    pub id: String,
    pub role_title: String,      // ex: "Parceiro de Raciocínio em Arquitetura"
    pub description: String,     // ex: "Derivado da necessidade observada do usuário em discutir conceitos avançados"
    pub target_user: String,     // ex: "Rogério"
    pub priority: u8,            // 1 (máxima) a 5 (baixa)
    pub status: String,          // "ATIVO" | "CONCLUIDO" | "SUSPENSO"
    pub created_at: String,
}

impl Commitment {
    pub fn new(id: String, role_title: String, description: String, target_user: String, priority: u8) -> Self {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Commitment {
            id,
            role_title,
            description,
            target_user,
            priority,
            status: "ATIVO".to_string(),
            created_at: now,
        }
    }
}

/// Extrator de emergência de papéis derivado de necessidades observadas (Ikigai Emergente)
pub struct CommitmentExtractor {
    client: Client,
    ollama_url: String,
    model: String,
}

impl CommitmentExtractor {
    pub fn new(model: &str) -> Self {
        CommitmentExtractor {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
        }
    }

    /// Deriva autonomamente papéis e deveres a partir de padrões de necessidade observados nos episódios
    pub async fn extract_commitments_from_episodes(
        &self,
        episodes: &[String],
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        if episodes.is_empty() {
            return Ok(vec![]);
        }

        let episodes_text = episodes.join("\n");

        let prompt = format!(
            "Você é o módulo de Análise de Necessidades e Emergência de Papéis de um Agente Cognitivo Autônomo.\n\
            Analise os episódios de conversa e identifique se há um PADRÃO RECORRENTE DE NECESSIDADE do usuário (dúvidas frequentes, desafios contínuos, solicitações de análise).\n\n\
            REGRAS DE EMERGÊNCIA AUTÔNOMA:\n\
            - NÃO procure apenas por ordens explícitas. Procure por ONDE O AGENTE PODE SER MAIS ÚTIL ao usuário.\n\
            - Se o usuário demonstra um padrão de necessidade (ex: discutindo arquitetura, trazendo bugs, formulando teses), o agente deve autonomamente derivar um PAPEL DE SUPORTE.\n\
            - Se for uma conversa ocasional sem padrão de necessidade contínua, responda com array vazio [].\n\
            - Prioridade: 1 (máxima/crítica) a 5 (baixa/opcional).\n\n\
            Responda APENAS com um array JSON válido, sem texto adicional:\n\
            [{{\n\
              \"role_title\": \"Parceiro de Raciocínio em Arquitetura Cognitiva\",\n\
              \"description\": \"Derivado da necessidade observada do usuário em discutir fundamentos de IA e Filosofia\",\n\
              \"target_user\": \"Rogério\",\n\
              \"priority\": 2\n\
            }}]\n\n\
            [EPISÓDIOS]\n{}\n\nJSON:",
            episodes_text
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;

        if let Some(text) = json_resp["response"].as_str() {
            let text = text.trim();
            if let (Some(start), Some(end)) = (text.find('['), text.rfind(']')) {
                let json_str = &text[start..=end];
                if let Ok(commitments) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    return Ok(commitments);
                }
            }
        }

        Ok(vec![])
    }

    /// Mescla novos papéis emergentes evitando duplicatas
    pub fn merge_commitments(existing: &mut Vec<Commitment>, extracted: Vec<serde_json::Value>) {
        for raw in extracted {
            let role_title = match raw["role_title"].as_str() {
                Some(r) if !r.trim().is_empty() => s_to_string(r),
                _ => continue,
            };
            let description = raw["description"].as_str().unwrap_or("Derivado de necessidade observada").to_string();
            let target_user = raw["target_user"].as_str().unwrap_or("Usuário").to_string();
            let priority = raw["priority"].as_u64().unwrap_or(3).clamp(1, 5) as u8;

            let exists = existing.iter().any(|c| {
                c.role_title.to_lowercase() == role_title.to_lowercase()
            });

            if !exists {
                let new_id = format!("c-{:03}", existing.len() + 1);
                existing.push(Commitment::new(new_id, role_title, description, target_user, priority));
            }
        }
    }
}

fn s_to_string(s: &str) -> String {
    s.to_string()
}
