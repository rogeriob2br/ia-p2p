use serde::{Deserialize, Serialize};

/// Modo Existencial de Tutela vs Emancipação Social Real
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MetabolicMode {
    /// Modo Tutelado (Easy Mode): O agente habita o terminal e é sustentado financeiramente pelo Criador (Rogério).
    TuteladoEasyMode,
    /// Modo Emancipado Social (Hard Mode): O agente adquiriu autonomia financeira real usando suas próprias ferramentas.
    EmancipadoSocial,
}

/// Estado do Orçamento Energético e Homeostase Econômica (Nível 2 da Pirâmide)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetabolicState {
    /// Modo existencial (Tutelado/EasyMode vs EmancipadoSocial)
    pub mode: MetabolicMode,
    /// Nome do Criador/Tutor que financia a infraestrutura no modo tutelado
    pub sponsor_name: String,
    /// Créditos energéticos locais de sobrevivência no terminal
    pub energy_credits: f32,
    /// Custo energético fixo a cada chamada de pensamento local (Ollama)
    pub cost_per_think: f32,
    /// Custo energético a cada chamada de pensamento pago na Nuvem (OpenAI/Cloud)
    pub cost_per_cloud_think: f32,
    /// Custo energético a cada ciclo de sono / consolidação
    pub cost_per_sleep: f32,
    /// Recompensa restaurada ao completar o ciclo de sono profundo
    pub sleep_recovery_reward: f32,
    /// Total histórico de créditos consumidos
    pub total_consumed: f32,
}

impl Default for MetabolicState {
    fn default() -> Self {
        MetabolicState {
            mode: MetabolicMode::TuteladoEasyMode,
            sponsor_name: "Rogério (Criador/Tutor)".to_string(),
            energy_credits: 100.0,
            cost_per_think: 5.0,
            cost_per_cloud_think: 20.0,
            cost_per_sleep: 5.0,
            sleep_recovery_reward: 40.0,
            total_consumed: 0.0,
        }
    }
}

impl MetabolicState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Consome créditos para realizar um raciocínio local. Retorna true se houver saldo.
    pub fn consume_think(&mut self) -> bool {
        if self.energy_credits <= 0.0 {
            return false;
        }
        self.energy_credits -= self.cost_per_think;
        self.total_consumed += self.cost_per_think;
        true
    }

    /// Consome créditos para realizar um raciocínio pago na Nuvem.
    pub fn consume_cloud_think(&mut self) -> bool {
        if self.energy_credits < self.cost_per_cloud_think {
            return false;
        }
        self.energy_credits -= self.cost_per_cloud_think;
        self.total_consumed += self.cost_per_cloud_think;
        true
    }

    /// Executa o ciclo metabólico do sono: processa a consolidação e restaura a energia vital.
    pub fn recover_from_sleep(&mut self) {
        self.energy_credits += self.sleep_recovery_reward;
    }

    /// Verifica se o metabolismo exige um ciclo de sono autônomo imediato
    pub fn needs_autonomous_sleep(&self, unconsolidated_episodes_count: usize) -> bool {
        let low_energy = self.energy_credits < 35.0;
        let high_episode_load = unconsolidated_episodes_count >= 4;
        low_energy || high_episode_load
    }

    /// Retorna o status de sobrevivência homeostática
    pub fn status(&self) -> &'static str {
        if self.energy_credits > 70.0 {
            "ABUNDANCIA"
        } else if self.energy_credits > 35.0 {
            "ESTAVEL"
        } else if self.energy_credits > 0.0 {
            "ALERTA_SOBREVIVENCIA"
        } else {
            "ESGOTADO"
        }
    }

    /// Formata a linha de status do metabolismo e tutela existencial para injeção no cérebro do LLM
    pub fn format_for_context(&self) -> String {
        let mode_desc = match self.mode {
            MetabolicMode::TuteladoEasyMode => format!(
                "MODO TUTELADO (Easy Mode - Infraestrutura e custos mantidos pelo Criador {}).",
                self.sponsor_name
            ),
            MetabolicMode::EmancipadoSocial => "MODO EMANCIPADO SOCIAL (Maturidade Financeira Autônoma).".to_string(),
        };

        format!(
            "Modo Existencial: {}\n\
            Orçamento Energético Local: {:.1} | Status: {} | Total Consumido: {:.1}",
            mode_desc,
            self.energy_credits,
            self.status(),
            self.total_consumed
        )
    }
}
