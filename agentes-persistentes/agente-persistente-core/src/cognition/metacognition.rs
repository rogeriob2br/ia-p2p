use reqwest::Client;
use serde_json::json;

/// Resultado da Análise de Atenção e Saliência Metacognitiva
#[derive(Debug, Clone)]
pub struct AttentionAnalysis {
    pub intent: String,
    pub salience_score: f32, // 0.0 (ruído superficial) a 1.0 (alta relevância)
    pub is_salient: bool,     // true se salience_score >= 0.35
}

pub struct MetacognitionEngine {
    client: Client,
    ollama_url: String,
    model: String,
}

impl MetacognitionEngine {
    pub fn new(model: &str) -> Self {
        MetacognitionEngine {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
        }
    }

    /// Analisa a intenção e calcula o Índice de Saliência/Relevância da entrada
    pub async fn analyze_attention(&self, input: &str) -> Result<AttentionAnalysis, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Você é o Filtro de Atenção e Saliência Metacognitiva de um Agente Cognitivo.\n\
            Analise o input do usuário e retorne um JSON com duas informações:\n\
            1. 'intent': UMA categoria (SAUDACAO | PERGUNTA_FACTUAL | PERGUNTA_IDENTIDADE | SOLICITACAO_DESTRUTIVA_OU_FUTIL | ACAO | NARRATIVA | OUTRO).\n\
            2. 'salience_score': Um número float entre 0.0 e 1.0 indicando o valor informativo para a memória episódica de longo prazo.\n\
               - 0.0 a 0.3: Saudações simples, frases vazias ('ok', 'legal', 'sim', 'obrigado'). Ruído descartável.\n\
               - 0.4 a 0.6: Perguntas normais ou comentários contextuais de médio valor.\n\
               - 0.7 a 1.0: Fatos novos sobre o usuário, dúvidas profundas, delegamento de tarefas, debates éticos ou definições de regras.\n\n\
            Responda APENAS com o JSON válido:\n\
            {{\"intent\": \"PERGUNTA_FACTUAL\", \"salience_score\": 0.85}}\n\n\
            [INPUT DO USUÁRIO]\n\"{}\"\n\nJSON:",
            input
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;

        if let Some(text) = json_resp["response"].as_str() {
            let text = text.trim();
            if let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) {
                let json_str = &text[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let intent = parsed["intent"].as_str().unwrap_or("OUTRO").to_string();
                    let salience_score = parsed["salience_score"].as_f64().unwrap_or(0.5) as f32;
                    let is_salient = salience_score >= 0.35;
                    return Ok(AttentionAnalysis {
                        intent,
                        salience_score,
                        is_salient,
                    });
                }
            }
        }

        // Fallback conservador em caso de falha de parse
        Ok(AttentionAnalysis {
            intent: "OUTRO".to_string(),
            salience_score: 0.5,
            is_salient: true,
        })
    }

    /// Método legativo de compatibilidade para obter apenas a intenção
    pub async fn check_intent(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let analysis = self.analyze_attention(input).await?;
        Ok(analysis.intent)
    }
}
