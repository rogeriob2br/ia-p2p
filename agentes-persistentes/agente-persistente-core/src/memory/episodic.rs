use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpisodicLog {
    pub episodes: Vec<String>,
}

impl Default for EpisodicLog {
    fn default() -> Self {
        EpisodicLog { episodes: Vec::new() }
    }
}

impl EpisodicLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, episode: String) {
        self.episodes.push(episode);
    }

    pub fn clear(&mut self) {
        self.episodes.clear();
    }

    pub fn recent(&self, count: usize) -> Vec<String> {
        self.episodes.iter().rev().take(count).cloned().collect()
    }
}
