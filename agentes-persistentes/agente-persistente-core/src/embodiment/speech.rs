use std::process::Command;

/// Módulo de Síntese de Voz Masculina Neural de Alta Definição (HD Neural TTS - Antonio)
pub struct SpeechSynthesizer {
    pub voice_name: String,
}

impl Default for SpeechSynthesizer {
    fn default() -> Self {
        SpeechSynthesizer {
            voice_name: "pt-BR-AntonioNeural".to_string(), // Voz Masculina Neural HD em Português do Brasil
        }
    }
}

impl SpeechSynthesizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fala o texto utilizando a Voz Masculina Neural de Alta Definição (pt-BR-AntonioNeural + ffplay)
    pub fn speak(&self, text: &str) {
        let clean_text = text.replace('"', "").replace('\'', "").replace('\n', " ");
        let audio_path = "/tmp/agente_voice_antonio.mp3";

        // 1. Tenta sintetizar a Voz Masculina Neural HD via edge-tts (pt-BR-AntonioNeural)
        let status_edge = Command::new("edge-tts")
            .arg("--voice")
            .arg(&self.voice_name)
            .arg("--text")
            .arg(&clean_text)
            .arg("--write-media")
            .arg(audio_path)
            .status();

        if let Ok(status) = status_edge {
            if status.success() {
                // Toca a voz masculina HD via ffplay sem janela gráfica
                let status_play = Command::new("ffplay")
                    .arg("-nodisp")
                    .arg("-autoexit")
                    .arg(audio_path)
                    .status();

                if status_play.is_ok() && status_play.unwrap().success() {
                    return;
                }

                // Fallback de player via paplay
                let _ = Command::new("paplay")
                    .arg(audio_path)
                    .status();
                return;
            }
        }

        // 2. Fallback via gTTS em Python caso o edge-tts falhe
        let py_cmd = format!(
            "from gtts import gTTS; tts = gTTS(\"{}\", lang='pt'); tts.save('/tmp/agente_voice_gtts.mp3')",
            clean_text
        );
        let _ = Command::new("python3").arg("-c").arg(&py_cmd).status();
        let _ = Command::new("ffplay").arg("-nodisp").arg("-autoexit").arg("/tmp/agente_voice_gtts.mp3").status();
    }
}
