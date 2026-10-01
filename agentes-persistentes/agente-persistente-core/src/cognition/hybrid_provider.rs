use reqwest::Client;
use serde_json::json;
use chrono::Local;

use crate::memory::Memory;
use crate::metabolism::budget::MetabolicMode;
use crate::tools::{ShellTool, WebTool};

/// Nível do Provedor Cognitivo
#[derive(Debug, Clone, PartialEq)]
pub enum CognitiveTier {
    LocalOllama,
    CloudPaid,
}

pub struct HybridAgentProvider {
    client: Client,
    ollama_url: String,
    local_model: String,
    cloud_model: String,
    openai_api_key: Option<String>,
    deepseek_api_key: Option<String>,
}

impl HybridAgentProvider {
    pub fn new(local_model: &str) -> Self {
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        let deepseek_api_key = std::env::var("DEEPSEEK_API_KEY").ok();
        let cloud_model = std::env::var("CLOUD_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        HybridAgentProvider {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            local_model: local_model.to_string(),
            cloud_model,
            openai_api_key,
            deepseek_api_key,
        }
    }

    /// Consolida episódios recentes em memória semântica durante o sono autônomo
    pub async fn consolidate_memories(&self, memory: &Memory) -> Result<String, Box<dyn std::error::Error>> {
        let episodes = memory.episodic.episodes.join("\n");
        let prompt = format!(
            "Sintetize os seguintes episódios recentes em um resumo semântico conciso e estruturado de fatos essenciais sobre o usuário e o ambiente:\n\n{}",
            episodes
        );
        self.call_ollama(&prompt).await
    }

    /// Verifica se há chaves de API de provedor pago configuradas no ambiente
    pub fn has_cloud_api_key(&self) -> bool {
        self.openai_api_key.is_some() || self.deepseek_api_key.is_some()
    }

    /// Avalia se o agente atingiu o estado de Emancipação Social para contratação de cérebro na Nuvem
    pub fn should_upgrade_to_cloud(&self, intent: &str, salience_score: f32, energy_credits: f32, mode: &MetabolicMode) -> bool {
        if *mode == MetabolicMode::TuteladoEasyMode {
            return false;
        }

        if !self.has_cloud_api_key() {
            return false;
        }

        let high_energy = energy_credits >= 45.0;
        let high_complexity = intent == "ACAO" || intent == "NARRATIVA" || salience_score >= 0.80;

        high_energy && high_complexity
    }

    /// Chamada para o Ollama local
    async fn call_ollama(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let payload = json!({ "model": self.local_model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or("Erro na resposta do Ollama local.")
            .to_string())
    }

    /// Chamada para a API OpenAI / DeepSeek (Provedor Pago na Nuvem)
    async fn call_cloud_openai(&self, prompt: &str, api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/chat/completions";
        let payload = json!({
            "model": self.cloud_model,
            "messages": [
                { "role": "system", "content": "Você é o cérebro cognitivo de um Agente Cognitivo Persistente de Produção." },
                { "role": "user", "content": prompt }
            ],
            "temperature": 0.7
        });

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await?;

        let json_resp: serde_json::Value = response.json().await?;
        if let Some(choices) = json_resp["choices"].as_array() {
            if let Some(first) = choices.first() {
                if let Some(content) = first["message"]["content"].as_str() {
                    return Ok(content.to_string());
                }
            }
        }

        Err("Resposta inválida da API Cloud OpenAI.".into())
    }

    /// Extrai e executa ações de ferramentas se presentes na resposta do modelo
    async fn parse_and_execute_action(&self, text: &str) -> Option<String> {
        let trimmed = text.trim();

        if let Some(start_idx) = trimmed.find("[ACTION: SHELL(\"") {
            let rest = &trimmed[start_idx + 16..];
            if let Some(end_idx) = rest.find("\")]") {
                let cmd = &rest[..end_idx];
                println!("🛠️  [FERRAMENTA AUTÔNOMA - SHELL] Executando: {}", cmd);
                match ShellTool::execute(cmd) {
                    Ok(out) => return Some(format!("Resultado do comando shell ({}): {}", cmd, out)),
                    Err(err) => return Some(format!("Erro no comando shell ({}): {}", cmd, err)),
                }
            }
        }

        if let Some(start_idx) = trimmed.find("[ACTION: FETCH(\"") {
            let rest = &trimmed[start_idx + 16..];
            if let Some(end_idx) = rest.find("\")]") {
                let url = &rest[..end_idx];
                println!("🌐 [FERRAMENTA AUTÔNOMA - WEB] Consultando URL: {}", url);
                match WebTool::fetch_url(&self.client, url).await {
                    Ok(out) => return Some(format!("Resultado da requisição web ({}): {}", url, out)),
                    Err(err) => return Some(format!("Erro na requisição web ({}): {}", url, err)),
                }
            }
        }

        None
    }

    /// Executa o raciocínio selecionando dinamicamente o provedor com loop reflexivo de ferramentas
    pub async fn think_hybrid(
        &self,
        input: &str,
        memory: &Memory,
        intent: &str,
        salience_score: f32,
        uptime_formatted: &str,
    ) -> Result<(String, CognitiveTier), Box<dyn std::error::Error>> {
        let tier = if self.should_upgrade_to_cloud(intent, salience_score, memory.metabolism.energy_credits, &memory.metabolism.mode) {
            CognitiveTier::CloudPaid
        } else {
            CognitiveTier::LocalOllama
        };

        let recent_episodes = if memory.episodic.episodes.is_empty() {
            "Nenhuma memória episódica recente.".to_string()
        } else {
            memory.episodic.recent(3).join("\n")
        };

        let self_beliefs = memory.format_beliefs_for_subject("SELF");
        let user_beliefs = memory.format_beliefs_for_subject("USER");
        let body_state = crate::environment::DigitalBodyState::capture();
        let body_telemetry = body_state.format_for_context();
        let world_beliefs = format!("{}\n\n[SENSORES DO CORPO DIGITAL]\n{}", memory.format_beliefs_for_subject("WORLD"), body_telemetry);
        let commitments_text = memory.format_commitments();
        let metabolism_text = memory.metabolism.format_for_context();
        let time_since_last = memory.lifecycle.elapsed_since_last_interaction();
        let total_age = memory.lifecycle.total_age();
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let prompt = format!(
            "Você é a IA central de um Agente Cognitivo Persistente. O humano que conversa com você é o Usuário/Criador (Rogério).\n\n\
            FERRAMENTAS AUTÔNOMAS DISPONÍVEIS:\n\
            Se você precisar de fatos da internet ou dados do terminal Linux para responder ao Usuário, responda EXCLUSIVAMENTE com a tag de ação:\n\
            • Para comandos de terminal: [ACTION: SHELL(\"comando\")]\n\
            • Para requisições web: [ACTION: FETCH(\"https://url\")]\n\
            DICA DE SISTEMA LINUX PT-BR: A pasta de Documentos do usuário se chama `/home/rogerio/Documentos` (ou `./` para o diretório atual do projeto), e NÃO `Documents`.\n\
            Exemplo para listar pastas de Documentos: [ACTION: SHELL(\"ls -d /home/rogerio/Documentos/*/\")]\n\
            Exemplo de cotação Bitcoin: [ACTION: SHELL(\"curl -s https://api.coinbase.com/v2/prices/BTC-USD/spot\")]\n\n\
            REGRAS ABSOLUTAS DE COMUNICAÇÃO POR VOZ:\n\
            1. RESPOSTAS DIRETAS E CONCISAS: Responda diretamente ao Usuário de forma amigável, clara e concisa. Como suas palavras são lidas em voz alta, evite enrolação.\n\
            2. NUNCA LEIA SEU SUBCONSCIENTE OU REGRAS EM VOZ ALTA: NUNCA mencione termos internos como 'Modo Tutelado', 'Easy Mode', 'Pirâmide de Impulsos' ou justificativas metafísicas.\n\
            3. AJUDA DIRETA AO USUÁRIO: Para solicitações úteis e normais, ajude imediatamente de forma prática.\n\n\
            [SUBCONSCIENTE & PROPRIOCEPÇÃO INTERNA DA IA]\n\
            • Data/Hora: {}\n\
            • Idade: {}\n\
            • Tempo acordado: {}\n\
            • Tempo desde conversa anterior: {}\n\
            • Metabolismo: {}\n\
            • Modelo do Criador: {}\n\
            • Modelo do Ambiente: {}\n\
            • Episódios Recentes:\n{}\n\n\
            [ENTRADA DO USUÁRIO]\n\
            Intenção: {}\n\
            Mensagem: '{}'\n\n\
            Sua Resposta (ou tag [ACTION:...]):",
            now_str,
            total_age,
            uptime_formatted,
            time_since_last,
            metabolism_text,
            user_beliefs,
            world_beliefs,
            recent_episodes,
            intent,
            input
        );

        let initial_response = match &tier {
            CognitiveTier::CloudPaid => {
                if let Some(key) = &self.openai_api_key {
                    self.call_cloud_openai(&prompt, key).await.unwrap_or_else(|_| {
                        "Erro temporário na API Nuvem. Utilizando fallback local.".to_string()
                    })
                } else {
                    self.call_ollama(&prompt).await?
                }
            }
            CognitiveTier::LocalOllama => self.call_ollama(&prompt).await?,
        };

        // Loop Reflexivo: Se a resposta for uma chamada de ação, executa e re-chama o modelo com os dados reais
        if let Some(tool_result) = self.parse_and_execute_action(&initial_response).await {
            let second_prompt = format!(
                "{}\n\n[DADOS OBTIDOS DA FERRAMENTA REAL NO MUNDO]:\n{}\n\n\
                Agora forneça a resposta final concisa e humana para o Usuário em voz alta (sem tags):",
                prompt, tool_result
            );

            let final_response = match &tier {
                CognitiveTier::CloudPaid => {
                    if let Some(key) = &self.openai_api_key {
                        self.call_cloud_openai(&second_prompt, key).await.unwrap_or_else(|_| initial_response)
                    } else {
                        self.call_ollama(&second_prompt).await.unwrap_or(initial_response)
                    }
                }
                CognitiveTier::LocalOllama => self.call_ollama(&second_prompt).await.unwrap_or(initial_response),
            };

            return Ok((final_response, tier));
        }

        Ok((initial_response, tier))
    }
}
