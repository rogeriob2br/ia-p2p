use serde::{Deserialize, Serialize};
use chrono::Local;

/// Relógio Interno e Métricas do Ciclo de Vida (Etapa 6)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LifeCycleMetrics {
    pub birth_timestamp: String,
    pub sleep_cycles_count: u32,
    pub total_conversations: u32,
    pub successful_actions: u32,
    pub failed_actions: u32,
    pub last_interaction_timestamp: String,
}

impl Default for LifeCycleMetrics {
    fn default() -> Self {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        LifeCycleMetrics {
            birth_timestamp: now.clone(),
            sleep_cycles_count: 0,
            total_conversations: 0,
            successful_actions: 0,
            failed_actions: 0,
            last_interaction_timestamp: now,
        }
    }
}

impl LifeCycleMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra uma nova interação
    pub fn register_interaction(&mut self, success: bool) {
        self.total_conversations += 1;
        if success {
            self.successful_actions += 1;
        } else {
            self.failed_actions += 1;
        }
        self.last_interaction_timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// Registra a conclusão de um ciclo de sono
    pub fn register_sleep(&mut self) {
        self.sleep_cycles_count += 1;
    }

    /// Calcula o tempo decorrido desde a última conversa
    pub fn elapsed_since_last_interaction(&self) -> String {
        let fmt = "%Y-%m-%d %H:%M:%S";
        if let Ok(last) = chrono::NaiveDateTime::parse_from_str(&self.last_interaction_timestamp, fmt) {
            let now = Local::now().naive_local();
            let duration = now.signed_duration_since(last);
            let mins = duration.num_minutes();
            let hours = duration.num_hours();
            let days = duration.num_days();
            if days > 0 {
                format!("{} dia(s) e {} hora(s)", days, hours % 24)
            } else if hours > 0 {
                format!("{} hora(s) e {} minuto(s)", hours, mins % 60)
            } else if mins > 0 {
                format!("{} minuto(s)", mins)
            } else {
                format!("{} segundo(s)", duration.num_seconds().max(0))
            }
        } else {
            "pouco tempo".to_string()
        }
    }

    /// Calcula a idade total do agente desde o nascimento
    pub fn total_age(&self) -> String {
        let fmt = "%Y-%m-%d %H:%M:%S";
        if let Ok(birth) = chrono::NaiveDateTime::parse_from_str(&self.birth_timestamp, fmt) {
            let now = Local::now().naive_local();
            let duration = now.signed_duration_since(birth);
            let mins = duration.num_minutes();
            let hours = duration.num_hours();
            let days = duration.num_days();
            if days > 0 {
                format!("{} dia(s), {} hora(s) e {} minuto(s)", days, hours % 24, mins % 60)
            } else if hours > 0 {
                format!("{} hora(s) e {} minuto(s)", hours, mins % 60)
            } else if mins > 0 {
                format!("{} minuto(s)", mins)
            } else {
                format!("{} segundo(s)", duration.num_seconds().max(0))
            }
        } else {
            "pouco tempo".to_string()
        }
    }
}
