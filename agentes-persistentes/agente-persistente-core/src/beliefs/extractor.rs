use reqwest::Client;
use serde_json::json;

pub struct BeliefExtractor {
    client: Client,
    ollama_url: String,
    model: String,
}

impl BeliefExtractor {
    pub fn new(model: &str) -> Self {
        BeliefExtractor {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
        }
    }

    /// Extrai hipóteses observáveis dos episódios brutos via LLM, classificando o sujeito
    pub async fn extract_from_episodes(
        &self,
        episodes: &[String],
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        if episodes.is_empty() {
            return Ok(vec![]);
        }

        let episodes_text = episodes.join("\n");

        let prompt = format!(
            "Você é o módulo de Extração de Evidências de um Agente Cognitivo Persistente.\n\
            Leia episódios de conversa e extraia hipóteses OBSERVÁVEIS, classificando cada uma por SUJEITO:\n\
            - 'USER': observações sobre o humano/usuário (nome, profissão, preferências, objetivos).\n\
            - 'SELF': observações sobre o comportamento, limitações ou capacidades da própria IA artificial.\n\
            - 'WORLD': fatos sobre o ambiente local, infraestrutura ou mundo exterior.\n\n\
            REGRAS ESTRITAS:\n\
            - Não invente nada. Só extraia o que está explícito no texto.\n\
            - NUNCA confunda o Usuário com o Agente. O humano (ex: Rogério) é 'USER'. O assistente de IA é 'SELF'.\n\
            - Confidence: 0.9 se mencionado explicitamente, 0.6 se uma vez, 0.3 se inferido.\n\
            - Evidence: copie o trecho exato do episódio que sustenta a hipótese.\n\n\
            Responda APENAS com um array JSON válido, sem texto adicional:\n\
            [{{\n\
              \"subject\": \"USER\",\n\
              \"statement\": \"O usuário se chama Rogério e é desenvolvedor.\",\n\
              \"confidence\": 0.9,\n\
              \"evidence\": [\"exemplo de fala\"]\n\
            }}]\n\n\
            [EPISÓDIOS]\n{}\n\nJSON:",
            episodes_text
        );

        let payload = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false
        });

        let response = self.client.post(&self.ollama_url)
            .json(&payload)
            .send()
            .await?;

        let json_resp: serde_json::Value = response.json().await?;

        if let Some(text) = json_resp["response"].as_str() {
            let text = text.trim();
            if let (Some(start), Some(end)) = (text.find('['), text.rfind(']')) {
                let json_str = &text[start..=end];
                if let Ok(beliefs) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    return Ok(beliefs);
                }
            }
        }

        Ok(vec![])
    }
}
