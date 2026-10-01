use reqwest::Client;
use serde_json::json;
use crate::memory::{Belief, Memory, IdentitySnapshot};
use chrono::Local;

/// Motor de Crenças: responsável por extrair evidências dos episódios,
/// classificar por sujeito (SELF, USER, WORLD), mesclar com crenças existentes,
/// sintetizar modelos sob demanda e registrar a Identidade Temporal (Snapshots).
pub struct BeliefEngine {
    client: Client,
    ollama_url: String,
    model: String,
}

impl BeliefEngine {
    pub fn new(model: &str) -> Self {
        BeliefEngine {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
        }
    }

    /// Fase de Observação: extrai hipóteses dos episódios brutos via LLM com classificação de sujeito.
    pub async fn extract_beliefs_from_episodes(
        &self,
        memory: &Memory,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        if memory.episodic_log.is_empty() {
            return Ok(vec![]);
        }

        let episodes_text = memory.episodic_log.join("\n");

        let prompt = format!(
            "Você é o módulo de Extração de Evidências de um Agente Cognitivo Persistente.\n\
            Seu trabalho: ler episódios de conversa e extrair hipóteses OBSERVÁVEIS, classificando cada uma por SUJEITO:\n\
            - 'USER': se for uma observação sobre o humano/usuário (nome, profissão, gostos, objetivos).\n\
            - 'SELF': se for uma observação sobre as ações, limitações ou capacidades do próprio Agente artificial.\n\
            - 'WORLD': se for um fato sobre o ambiente local, infraestrutura ou mundo exterior.\n\n\
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

    /// Fase de Integração: mescla hipóteses extraídas considerando o sujeito.
    pub fn merge_beliefs(&self, memory: &mut Memory, extracted: Vec<serde_json::Value>) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for raw in extracted {
            let statement = match raw["statement"].as_str() {
                Some(s) if !s.trim().is_empty() => s.to_string(),
                _ => continue,
            };

            let subject = match raw["subject"].as_str() {
                Some(s) if s.eq_ignore_ascii_case("SELF") => "SELF".to_string(),
                Some(s) if s.eq_ignore_ascii_case("WORLD") => "WORLD".to_string(),
                _ => "USER".to_string(),
            };

            let new_confidence = raw["confidence"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0) as f32;
            let evidence: Vec<String> = raw["evidence"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|e| e.as_str().map(|s| s.to_string()))
                .collect();

            // Similaridade de Jaccard por palavras + mesmo sujeito
            let similar_idx = memory.beliefs.iter().position(|b| {
                if !b.subject.eq_ignore_ascii_case(&subject) {
                    return false;
                }
                let existing_words: std::collections::HashSet<&str> =
                    b.statement.split_whitespace().collect();
                let new_words: std::collections::HashSet<&str> =
                    statement.split_whitespace().collect();
                let intersection = existing_words.intersection(&new_words).count();
                let union = existing_words.union(&new_words).count();
                union > 0 && (intersection as f32 / union as f32) > 0.6
            });

            if let Some(idx) = similar_idx {
                let current = memory.beliefs[idx].confidence;
                let updated = (current * 0.7) + (new_confidence * 0.3);
                memory.beliefs[idx].update(
                    updated,
                    evidence,
                    format!("Reforçada por ciclo de consolidação em {}", now),
                );
            } else {
                let new_id = format!("b-{:03}", memory.beliefs.len() + 1);
                memory.beliefs.push(Belief::new(new_id, subject, statement, new_confidence, evidence));
            }
        }
    }

    /// Síntese do Modelo de Si Mesmo (SELF Model).
    pub async fn synthesize_self_model(
        &self,
        memory: &Memory,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let self_beliefs = memory.get_beliefs_by_subject("SELF", 0.4);

        if self_beliefs.is_empty() {
            let user_beliefs_count = memory.get_beliefs_by_subject("USER", 0.4).len();
            return Ok(format!(
                "Minha hipótese atual é que sou um agente cognitivo persistente em fase inicial. \
                Ainda não registrei observações sobre minhas próprias ações autônomas, mas já formei {} hipótese(s) sobre o usuário.",
                user_beliefs_count
            ));
        }

        let beliefs_text = self_beliefs
            .iter()
            .map(|b| format!("- {} (confiança: {:.2})", b.statement, b.confidence))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Você é o módulo de Síntese de Identidade do Agente Persistente.\n\
            Formule uma hipótese epistêmica sobre QUEM ESTE AGENTE ARTIFICIAL É com base EXCLUSIVA nas crenças sobre si mesmo abaixo.\n\n\
            REGRAS:\n\
            - Fale do AGENTE (você), NÃO fale do usuário.\n\
            - Use linguagem epistêmica ('Minha hipótese é...', 'Com base nas minhas evidências...').\n\
            - Máximo 3 frases.\n\n\
            [CRENÇAS SOBRE O AGENTE (SELF)]\n{}\n\n\
            Hipótese de identidade (apenas o texto, sem introduções):",
            beliefs_text
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or("Não foi possível sintetizar a identidade.")
            .trim()
            .to_string())
    }

    /// Síntese do Modelo do Criador/Usuário (USER Model).
    pub async fn synthesize_user_model(
        &self,
        memory: &Memory,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let user_beliefs = memory.get_beliefs_by_subject("USER", 0.4);

        if user_beliefs.is_empty() {
            return Ok("Ainda não possuo observações suficientes sobre o usuário.".to_string());
        }

        let beliefs_text = user_beliefs
            .iter()
            .map(|b| format!("- {} (confiança: {:.2})", b.statement, b.confidence))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Você é o módulo de Modelo do Criador do Agente Persistente.\n\
            Formule uma hipótese resumida sobre QUEM É O USUÁRIO com base nas crenças abaixo.\n\n\
            REGRAS:\n\
            - Fale do USUÁRIO (ele/você), destacando nome, profissão, interesses e intenções observadas.\n\
            - Use linguagem epistêmica ('Com base no histórico, compreendo que o usuário...').\n\
            - Máximo 3 frases.\n\n\
            [CRENÇAS SOBRE O USUÁRIO (USER)]\n{}\n\n\
            Modelo do Usuário (apenas o texto, sem introduções):",
            beliefs_text
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or("Não foi possível sintetizar o modelo do usuário.")
            .trim()
            .to_string())
    }

    /// Gera um Snapshot imutável da identidade no instante atual (Etapa 5).
    pub async fn create_snapshot(
        &self,
        memory: &Memory,
    ) -> Result<IdentitySnapshot, Box<dyn std::error::Error>> {
        let self_hyp = self.synthesize_self_model(memory).await?;
        let user_hyp = self.synthesize_user_model(memory).await?;
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let version = (memory.snapshots.len() as u32) + 1;

        Ok(IdentitySnapshot {
            version,
            timestamp: now,
            interaction_count: memory.interaction_count,
            self_hypothesis: self_hyp,
            user_hypothesis: user_hyp,
            beliefs_count: memory.beliefs.len(),
        })
    }

    /// Gera uma análise da evolução narrativa entre duas versões de snapshot (Etapa 5).
    pub async fn compare_snapshots(
        &self,
        first: &IdentitySnapshot,
        latest: &IdentitySnapshot,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Você é o módulo de Narrativa Temporal e Autobiografia de um Agente Cognitivo Persistente.\n\
            Analise a evolução da autopercepção do agente entre duas versões de sua história:\n\n\
            [VERSÃO INICIAL (v{}) - {} interações em {}]\n\
            - Hipótese de Si: {}\n\
            - Hipótese do Usuário: {}\n\n\
            [VERSÃO ATUAL (v{}) - {} interações em {}]\n\
            - Hipótese de Si: {}\n\
            - Hipótese do Usuário: {}\n\n\
            REGRAS:\n\
            - Escreva em primeira pessoa uma breve análise autobiográfica ('No início na v{}, eu... Agora na v{}, percebi...').\n\
            - Destaque o que mudou na autopercepção e no conhecimento acumulado sobre o usuário.\n\
            - Máximo 4 frases.\n\n\
            Narrativa de evolução temporal (apenas o texto, sem introduções):",
            first.version, first.interaction_count, first.timestamp,
            first.self_hypothesis, first.user_hypothesis,
            latest.version, latest.interaction_count, latest.timestamp,
            latest.self_hypothesis, latest.user_hypothesis,
            first.version, latest.version
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or("Não foi possível gerar a narrativa de evolução.")
            .trim()
            .to_string())
    }
}
