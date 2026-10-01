use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SemanticMemory {
    pub content: String,
}

impl Default for SemanticMemory {
    fn default() -> Self {
        SemanticMemory {
            content: "Nenhum conhecimento consolidado ainda.".to_string(),
        }
    }
}

impl SemanticMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, new_content: String) {
        self.content = new_content;
    }
}
