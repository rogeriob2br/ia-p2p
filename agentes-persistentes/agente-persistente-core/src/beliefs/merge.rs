use crate::beliefs::Belief;
use chrono::Local;

/// Mescla hipóteses extraídas com as crenças existentes em Rust puro (sem LLM).
pub fn merge_extracted_beliefs(existing_beliefs: &mut Vec<Belief>, extracted: Vec<serde_json::Value>) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for raw in extracted {
        let statement = match raw["statement"].as_str() {
            Some(s) if !s.trim().is_empty() => s.to_string(),
            _ => continue,
        };

        let subject = match raw["subject"].as_str() {
            Some(s) if s.eq_ignore_ascii_case("SELF") => "SELF".to_string(),
            Some(s) if s.eq_ignore_ascii_case("WORLD") => "WORLD".to_string(),
            _ => "USER".to_string(),
        };

        let new_confidence = raw["confidence"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0) as f32;
        let evidence: Vec<String> = raw["evidence"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|e| e.as_str().map(|s| s.to_string()))
            .collect();

        // Similaridade de Jaccard por palavras + mesmo sujeito
        let similar_idx = existing_beliefs.iter().position(|b| {
            if !b.subject.eq_ignore_ascii_case(&subject) {
                return false;
            }
            let words_b: std::collections::HashSet<&str> = b.statement.split_whitespace().collect();
            let words_new: std::collections::HashSet<&str> = statement.split_whitespace().collect();
            let intersection = words_b.intersection(&words_new).count();
            let union = words_b.union(&words_new).count();
            union > 0 && (intersection as f32 / union as f32) > 0.6
        });

        if let Some(idx) = similar_idx {
            let current = existing_beliefs[idx].confidence;
            let updated = (current * 0.7) + (new_confidence * 0.3);
            existing_beliefs[idx].update(
                updated,
                evidence,
                format!("Reforçada por ciclo de consolidação em {}", now),
            );
        } else {
            let new_id = format!("b-{:03}", existing_beliefs.len() + 1);
            existing_beliefs.push(Belief::new(new_id, subject, statement, new_confidence, evidence));
        }
    }
}
