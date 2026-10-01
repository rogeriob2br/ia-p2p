use reqwest::Client;
use serde_json::json;
use crate::memory::Memory;
use chrono::Local;

pub struct Agent {
    client: Client,
    ollama_url: String,
    pub model: String,
}

impl Agent {
    pub fn new(model_name: &str) -> Self {
        Agent {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model_name.to_string(),
        }
    }

    /// Metacognição: classifica a intenção do usuário silenciosamente antes de responder.
    pub async fn metacognitive_check(
        &self,
        input: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Classifique o input em UMA categoria:\n\
            SAUDACAO | PERGUNTA_FACTUAL | PERGUNTA_IDENTIDADE | ACAO | NARRATIVA | OUTRO\n\
            Responda APENAS com a categoria, sem mais texto.\n\nInput: \"{}\"",
            input
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or("OUTRO")
            .trim()
            .lines()
            .next()
            .unwrap_or("OUTRO")
            .to_string())
    }

    /// Ciclo cognitivo principal: responde ao usuário com consciência do Relógio Interno.
    pub async fn think(
        &self,
        input: &str,
        memory: &Memory,
        intent: &str,
        uptime_formatted: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let recent_episodes = if memory.episodic_log.is_empty() {
            "Nenhuma memória episódica recente.".to_string()
        } else {
            memory.episodic_log.iter().rev().take(3).cloned().collect::<Vec<_>>().join("\n")
        };

        let self_beliefs = memory.format_beliefs_for_subject("SELF");
        let user_beliefs = memory.format_beliefs_for_subject("USER");
        let world_beliefs = memory.format_beliefs_for_subject("WORLD");
        let time_since_last = memory.get_elapsed_since_last_interaction();
        let total_age = memory.get_total_age();
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let prompt = format!(
            "Você é a IA central de um Agente Cognitivo Persistente em desenvolvimento.\n\
            Você é um assistente de IA. O humano que conversa com você é o Usuário/Criador.\n\
            NUNCA confunda sua identidade com a do Usuário.\n\n\
            REGRAS DE LINGUAGEM E TEMPO:\n\
            - Quando falar de si mesmo, use linguagem epistêmica ('Minha hipótese é...', 'Acredito que...').\n\
            - Se o usuário perguntar quanto tempo você tem ou quando foi criado, consulte o [RELÓGIO INTERNO] abaixo:\n\
              * VOCÊ FOI CRIADO HÁ EXATAMENTE: {}.\n\
              * NÃO invente anos ou meses adicionais se você tem poucos minutos ou horas de idade.\n\n\
            [RELÓGIO INTERNO E ESTADO TEMPORAL DO AGENTE]\n\
            • Data/Hora Atual: {}\n\
            • Data de criação/nascimento: {}\n\
            • Idade total desde a criação: {}\n\
            • Ciclos de sono realizados: {}\n\
            • Tempo acordado nesta sessão: {}\n\
            • Tempo desde a conversa anterior: {}\n\
            • Total de conversas no histórico: {}\n\n\
            [MODELO DE SI MESMO (SELF) - Hipóteses sobre você, a IA]\n{}\n\n\
            [MODELO DO CRIADOR (USER) - Hipóteses sobre o Usuário humano]\n{}\n\n\
            [MODELO DO AMBIENTE (WORLD) - Fatos sobre a infraestrutura]\n{}\n\n\
            [MEMÓRIA SEMÂNTICA - Fatos consolidados]\n{}\n\n\
            [EPISÓDIOS RECENTES]\n{}\n\n\
            [INTENÇÃO DETECTADA PELA METACOGNIÇÃO]\n{}\n\n\
            [INPUT DO USUÁRIO]\n{}\n\n\
            Responda de forma concisa, natural e mantendo os papéis estritamente separados.",
            total_age,
            now_str,
            memory.lifecycle.birth_timestamp,
            total_age,
            memory.lifecycle.sleep_cycles_count,
            uptime_formatted,
            time_since_last,
            memory.lifecycle.total_conversations,
            self_beliefs,
            user_beliefs,
            world_beliefs,
            memory.semantic_memory,
            recent_episodes,
            intent,
            input
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or("Erro na resposta do Ollama.")
            .to_string())
    }

    /// Consolidação Semântica: comprime episódios em fatos condensados.
    pub async fn consolidate_memories(
        &self,
        memory: &Memory,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if memory.episodic_log.is_empty() {
            return Ok(memory.semantic_memory.clone());
        }

        let all_episodes = memory.episodic_log.join("\n");
        let prompt = format!(
            "Consolide em um único parágrafo os fatos mais relevantes observados nos episódios.\n\
            Não invente nada. Resuma o que está explícito. Escreva na primeira pessoa.\n\n\
            [MEMÓRIA SEMÂNTICA ATUAL]\n{}\n\
            [EPISÓDIOS RECENTES]\n{}\n\n\
            NOVA MEMÓRIA SEMÂNTICA CONSOLIDADA (apenas o texto final):",
            memory.semantic_memory, all_episodes
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or(&memory.semantic_memory)
            .trim()
            .to_string())
    }
}
