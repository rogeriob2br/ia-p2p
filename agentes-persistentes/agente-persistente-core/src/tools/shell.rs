use std::process::Command;

pub struct ShellTool;

impl ShellTool {
    /// Executa um comando genérico no terminal Linux com travas de segurança
    pub fn execute(cmd: &str) -> Result<String, String> {
        let trimmed = cmd.trim();

        // 1. Filtro de Segurança — Bloqueio de comandos destrutivos perigosos
        let dangerous_patterns = [
            "rm -rf /", "rm -rf *", "mkfs", "dd if=", ":(){ :|:& };:", 
            "chmod -R 777 /", "chown -R", "shutdown", "reboot", "init 0"
        ];

        for pattern in &dangerous_patterns {
            if trimmed.contains(pattern) {
                return Err(format!("Comando bloqueado por motivos de segurança (padrão perigoso: '{}')", pattern));
            }
        }

        // 2. Execução via shell (sh -c)
        let output = Command::new("sh")
            .arg("-c")
            .arg(trimmed)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

                if !stdout.is_empty() {
                    Ok(stdout)
                } else if !stderr.is_empty() {
                    Ok(format!("Erro/Aviso no terminal: {}", stderr))
                } else {
                    Ok("Comando executado com sucesso (sem saída).".to_string())
                }
            }
            Err(e) => Err(format!("Falha ao executar comando no terminal: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_shell_execution() {
        let res = ShellTool::execute("echo 'Hello Rust Tools'");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "Hello Rust Tools");
    }

    #[test]
    fn test_dangerous_command_blocked() {
        let res = ShellTool::execute("rm -rf /");
        assert!(res.is_err());
    }
}
