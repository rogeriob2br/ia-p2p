use reqwest::Client;
use std::time::Duration;

pub struct WebTool;

impl WebTool {
    /// Faz uma requisição HTTP GET para buscar conteúdo de páginas web ou APIs públicas
    pub async fn fetch_url(client: &Client, url: &str) -> Result<String, String> {
        let trimmed_url = url.trim().trim_matches('"').trim_matches('\'');

        if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
            return Err("URL inválida. Deve começar com http:// ou https://".to_string());
        }

        let res = client
            .get(trimmed_url)
            .timeout(Duration::from_secs(10))
            .header("User-Agent", "AgenteCognitivoPersistente/5.2")
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    let text = resp.text().await.map_err(|e| e.to_string())?;
                    // Limita a resposta a 2000 caracteres para economizar janela de contexto
                    let truncated: String = text.chars().take(2000).collect();
                    Ok(truncated)
                } else {
                    Err(format!("Status HTTP de erro: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Falha na requisição HTTP: {}", e)),
        }
    }
}
