use sysinfo::System;
use serde::{Deserialize, Serialize};

/// Telemetria do Corpo Digital do Agente (Hardware e SO)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DigitalBodyState {
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub cpu_count: usize,
    pub cpu_usage_pct: f32,
    pub working_directory: String,
}

impl DigitalBodyState {
    /// Lê instantaneamente os sensores do corpo digital (hardware e SO)
    pub fn capture() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(std::time::Duration::from_millis(50));
        sys.refresh_cpu();

        let os_name = System::name().unwrap_or_else(|| "Linux".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());

        let total_memory_mb = sys.total_memory() / (1024 * 1024);
        let used_memory_mb = sys.used_memory() / (1024 * 1024);

        let cpus = sys.cpus();
        let cpu_count = cpus.len();
        let cpu_usage_pct: f32 = if !cpus.is_empty() {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / (cpus.len() as f32)
        } else {
            0.0
        };

        let working_directory = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "Desconhecido".to_string());

        DigitalBodyState {
            os_name,
            os_version,
            hostname,
            total_memory_mb,
            used_memory_mb,
            cpu_count,
            cpu_usage_pct,
            working_directory,
        }
    }

    /// Retorna o percentual de uso de RAM
    pub fn memory_usage_pct(&self) -> f32 {
        if self.total_memory_mb == 0 {
            0.0
        } else {
            (self.used_memory_mb as f32 / self.total_memory_mb as f32) * 100.0
        }
    }

    /// Formata uma barra de progresso em texto
    pub fn render_bar(pct: f32, width: usize) -> String {
        let filled = ((pct / 100.0) * (width as f32)).round() as usize;
        let filled = filled.clamp(0, width);
        let empty = width.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    /// Formata a telemetria do corpo para injeção no prompt do LLM (WORLD MODEL)
    pub fn format_for_context(&self) -> String {
        format!(
            "Host: {} ({}) | OS: {} {}\n\
            RAM: {}MB / {}MB ({:.1}%) | CPUs: {} núcleos ({:.1}% uso)\n\
            Diretório de Trabalho: {}",
            self.hostname,
            self.working_directory,
            self.os_name,
            self.os_version,
            self.used_memory_mb,
            self.total_memory_mb,
            self.memory_usage_pct(),
            self.cpu_count,
            self.cpu_usage_pct,
            self.working_directory
        )
    }
}
