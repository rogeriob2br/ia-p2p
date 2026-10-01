use std::process::Command;
use std::path::Path;

/// Módulo de Escuta de Microfone e Transcrição de Voz Ambiente (Speech-to-Text / STT)
pub struct VoiceActivityListener {
    pub is_listening: bool,
}

impl Default for VoiceActivityListener {
    fn default() -> Self {
        VoiceActivityListener { is_listening: true }
    }
}

impl VoiceActivityListener {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves the path to a Python script in the embodiment directory.
    fn resolve_script_path(script_name: &str) -> String {
        let relative = format!("src/embodiment/{}", script_name);
        if Path::new(&relative).exists() {
            return relative;
        }
        // Fallback if run from parent directory
        let alt = format!("agente-persistente-core/src/embodiment/{}", script_name);
        if Path::new(&alt).exists() {
            return alt;
        }
        relative
    }

    /// Escuta contínua com Voice Activity Detection (VAD).
    /// Bloqueia até o usuário falar e parar (~1.5s de silêncio), sem limite de tempo.
    /// Retorna o texto transcrito da fala detectada.
    pub fn listen_continuous(&self) -> Result<String, Box<dyn std::error::Error>> {
        let script_path = Self::resolve_script_path("listen_daemon.py");

        let py_out = Command::new("/usr/bin/python3")
            .arg(&script_path)
            .output()?;

        let transcribed = String::from_utf8_lossy(&py_out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&py_out.stderr).trim().to_string();

        // Mostra mensagens de status do VAD no console (são informativas, vão para stderr)
        if !stderr.is_empty() {
            for line in stderr.lines() {
                println!("{}", line);
            }
        }

        if !transcribed.is_empty() {
            Ok(transcribed)
        } else {
            Err("Nenhuma fala detectada no microfone.".into())
        }
    }

    /// Captura fixa de 4 segundos (fallback legado).
    pub fn listen_ambient_speech(&self) -> Result<String, Box<dyn std::error::Error>> {
        let audio_path = "/tmp/mic_input.wav";

        let status_ffmpeg = Command::new("ffmpeg")
            .arg("-y")
            .arg("-loglevel")
            .arg("quiet")
            .arg("-f")
            .arg("pulse")
            .arg("-i")
            .arg("default")
            .arg("-t")
            .arg("4")
            .arg("-ac")
            .arg("1")
            .arg("-ar")
            .arg("16000")
            .arg(audio_path)
            .status();

        let rec_success = match status_ffmpeg {
            Ok(st) => st.success(),
            Err(_) => false,
        };

        if !rec_success {
            let _ = Command::new("arecord")
                .arg("-q")
                .arg("-d")
                .arg("4")
                .arg("-f")
                .arg("S16_LE")
                .arg("-r")
                .arg("16000")
                .arg(audio_path)
                .status();
        }

        let script_path = Self::resolve_script_path("transcribe.py");

        let py_out = Command::new("/usr/bin/python3")
            .arg(&script_path)
            .arg(audio_path)
            .output()?;

        let transcribed = String::from_utf8_lossy(&py_out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&py_out.stderr).trim().to_string();

        if !transcribed.is_empty() {
            Ok(transcribed)
        } else if !stderr.is_empty() {
            Err(format!("Erro na transcrição: {}", stderr).into())
        } else {
            Err("Nenhuma fala detectada no microfone.".into())
        }
    }
}
