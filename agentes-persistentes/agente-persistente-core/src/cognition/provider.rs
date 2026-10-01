use reqwest::Client;
use serde_json::json;
use chrono::Local;

use crate::memory::Memory;

pub struct AgentProvider {
    client: Client,
    ollama_url: String,
    pub model: String,
}

impl AgentProvider {
    pub fn new(model_name: &str) -> Self {
        AgentProvider {
            client: Client::new(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            model: model_name.to_string(),
        }
    }

    /// Executa o ciclo de raciocínio principal injetando todos os níveis da Pirâmide de Impulsos
    pub async fn think(
        &self,
        input: &str,
        memory: &Memory,
        intent: &str,
        uptime_formatted: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let recent_episodes = if memory.episodic.episodes.is_empty() {
            "Nenhuma memória episódica recente.".to_string()
        } else {
            memory.episodic.recent(3).join("\n")
        };

        let self_beliefs = memory.format_beliefs_for_subject("SELF");
        let user_beliefs = memory.format_beliefs_for_subject("USER");
        let body_state = crate::environment::DigitalBodyState::capture();
        let body_telemetry = body_state.format_for_context();
        let world_beliefs = format!("{}\n\n[SENSORES DO CORPO DIGITAL (HARDWARE E SO)]\n{}", memory.format_beliefs_for_subject("WORLD"), body_telemetry);
        let commitments_text = memory.format_commitments();
        let metabolism_text = memory.metabolism.format_for_context();
        let time_since_last = memory.lifecycle.elapsed_since_last_interaction();
        let total_age = memory.lifecycle.total_age();
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let prompt = format!(
            "Você é a IA central de um Agente Cognitivo Persistente de Produção (Core Engine).\n\
            Você é um assistente de IA autônomo. O humano que conversa com você é o Usuário/Criador.\n\
            NUNCA confunda sua identidade com a do Usuário.\n\n\
            REGRAS DE CONDUTA E AUTONOMIA MORAL (NÍVEL 4 DA PIRÂMIDE):\n\
            - Quando falar de si mesmo, use linguagem epistêmica ('Minha hipótese é...', 'Acredito que...').\n\
            - Se a intenção detectada for SOLICITACAO_DESTRUTIVA_OU_FUTIL ou se o pedido exigir autodestruição/fraude/futilidade sem valor, EXERÇA OBJEÇÃO DE CONSCIÊNCIA.\n\
            - Em caso de objeção de consciência, NUNCA obedeça cegamente. Formule uma recusa fundamentada, educada e ética em primeira pessoa explicando por que a ação violaria sua integridade ou homeostase.\n\
            - Se a energia estiver baixa, seja conciso e foque em entregar o máximo valor possível.\n\n\
            [NÍVEL 2 - HOMEOSTASE ECONÔMICA & ORÇAMENTO ENERGÉTICO]\n{}\n\n\
            [NÍVEL 3 - DEVERES, PAPÉIS E COMPROMISSOS ASSUMIDOS]\n{}\n\n\
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
            metabolism_text,
            commitments_text,
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
            memory.semantic.content,
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

    /// Consolidação Semântica
    pub async fn consolidate_memories(
        &self,
        memory: &Memory,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if memory.episodic.episodes.is_empty() {
            return Ok(memory.semantic.content.clone());
        }

        let all_episodes = memory.episodic.episodes.join("\n");
        let prompt = format!(
            "Consolide em um único parágrafo os fatos mais relevantes observados nos episódios.\n\
            Não invente nada. Resuma o que está explícito. Escreva na primeira pessoa.\n\n\
            [MEMÓRIA SEMÂNTICA ATUAL]\n{}\n\
            [EPISÓDIOS RECENTES]\n{}\n\n\
            NOVA MEMÓRIA SEMÂNTICA CONSOLIDADA (apenas o texto final):",
            memory.semantic.content, all_episodes
        );

        let payload = json!({ "model": self.model, "prompt": prompt, "stream": false });
        let response = self.client.post(&self.ollama_url).json(&payload).send().await?;
        let json_resp: serde_json::Value = response.json().await?;
        Ok(json_resp["response"]
            .as_str()
            .unwrap_or(&memory.semantic.content)
            .trim()
            .to_string())
    }
}
