use reqwest::Client;
use serde_json::json;
use crate::beliefs::Belief;
use crate::identity::snapshots::IdentitySnapshot;
use chrono::Local;

pub struct IdentityNarrativeEngine {
    client: Client,
    ollama_url: String,
    model: String,
}

impl IdentityNarrativeEngine {
    pub fn new(model: &str) -> Self {
        IdentityNarrativeEngine {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
        }
    }

    /// Síntese do Modelo de Si Mesmo (SELF Model)
    pub async fn synthesize_self_model(
        &self,
        beliefs: &[Belief],
        interaction_count: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let self_beliefs: Vec<&Belief> = beliefs
            .iter()
            .filter(|b| b.subject.eq_ignore_ascii_case("SELF") && b.confidence >= 0.4)
            .collect();

        if self_beliefs.is_empty() {
            let user_beliefs_count = beliefs.iter().filter(|b| b.subject.eq_ignore_ascii_case("USER")).count();
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
            [CRENÇAS SOBRE O AGENTE (SELF)]\n{}\n\
            [INTERAÇÕES REALIZADAS]\n{}\n\n\
            Hipótese de identidade (apenas o texto, sem introduções):",
            beliefs_text, interaction_count
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

    /// Síntese do Modelo do Criador/Usuário (USER Model)
    pub async fn synthesize_user_model(
        &self,
        beliefs: &[Belief],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let user_beliefs: Vec<&Belief> = beliefs
            .iter()
            .filter(|b| b.subject.eq_ignore_ascii_case("USER") && b.confidence >= 0.4)
            .collect();

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

    /// Cria um Snapshot de Identidade imutável registrado no momento do sono.
    pub async fn create_snapshot(
        &self,
        beliefs: &[Belief],
        interaction_count: u32,
        snapshots_len: usize,
    ) -> Result<IdentitySnapshot, Box<dyn std::error::Error>> {
        let self_hyp = self.synthesize_self_model(beliefs, interaction_count).await?;
        let user_hyp = self.synthesize_user_model(beliefs).await?;
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let version = (snapshots_len as u32) + 1;

        Ok(IdentitySnapshot {
            version,
            timestamp: now,
            interaction_count,
            self_hypothesis: self_hyp,
            user_hypothesis: user_hyp,
            beliefs_count: beliefs.len(),
        })
    }

    /// Gera uma comparação da evolução narrativa entre dois snapshots.
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
