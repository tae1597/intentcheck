use regex::Regex;
use std::collections::{HashMap, HashSet};

const STOPWORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "to", "of", "in", "on", "for", "with", "as", "is", "are",
    "was", "were", "be", "this", "that", "it", "you", "your", "we", "our", "at", "by", "from",
];

fn normalize(text: &str) -> Vec<String> {
    let re = Regex::new(r"[^a-z0-9\s]+").unwrap();
    let lower = text.to_lowercase();
    let cleaned = re.replace_all(&lower, " ");

    cleaned
        .split_whitespace()
        .map(|w| w.trim().to_string())
        .filter(|w| w.len() >= 3)
        .filter(|w| !STOPWORDS.contains(&w.as_str()))
        .collect()
}

fn top_keywords(tokens: &[String], n: usize) -> Vec<String> {
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for t in tokens {
        *freq.entry(t.as_str()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(&str, usize)> = freq.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    pairs
        .into_iter()
        .take(n)
        .map(|(w, _)| w.to_string())
        .collect()
}

fn overlap_ratio(a: &[String], b: &[String]) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let set_a: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let inter = set_a.intersection(&set_b).count() as f32;
    let denom = set_a.len().max(set_b.len()) as f32;
    inter / denom
}

pub struct Scored {
    pub score: u8,
    pub warnings: Vec<String>,
    pub top_keywords: Vec<String>,
}

pub fn score_page(
    title: Option<&str>,
    meta: Option<&str>,
    h1: &[String],
    body: &str,
    min_words: usize,
) -> Scored {
    let body_tokens = normalize(body);
    let body_kw = top_keywords(&body_tokens, 10);

    let mut warnings: Vec<String> = vec![];

    // Basic presence checks
    if title.map(|s| s.trim().is_empty()).unwrap_or(true) {
        warnings.push("Missing or empty <title>.".to_string());
    }
    if meta.map(|s| s.trim().is_empty()).unwrap_or(true) {
        warnings.push("Missing meta description.".to_string());
    }
    if h1.is_empty() {
        warnings.push("Missing <h1>.".to_string());
    }
    if h1.len() > 1 {
        warnings.push("Multiple <h1> found (usually only one is recommended).".to_string());
    }

    // Thin content
    if body_tokens.len() < min_words {
        warnings.push(format!(
            "Content seems thin ({} words found, recommended at least {}).",
            body_tokens.len(),
            min_words
        ));
    }

    // Title length checks
    if let Some(t) = title {
        let len = t.chars().count();
        if len < 15 {
            warnings.push(format!("Title is quite short ({} characters).", len));
        } else if len > 60 {
            warnings.push(format!("Title is quite long ({} characters).", len));
        }
    }

    // Meta length checks
    if let Some(m) = meta {
        let len = m.chars().count();
        if len < 50 {
            warnings.push(format!("Meta description is quite short ({} characters).", len));
        } else if len > 160 {
            warnings.push(format!("Meta description is quite long ({} characters).", len));
        }
    }

    // Overlap scoring
    let title_tokens = title.map(normalize).unwrap_or_default();
    let meta_tokens = meta.map(normalize).unwrap_or_default();
    let h1_tokens = normalize(&h1.join(" "));

    let t_body = overlap_ratio(&title_tokens, &body_tokens);
    let m_body = overlap_ratio(&meta_tokens, &body_tokens);
    let h_body = overlap_ratio(&h1_tokens, &body_tokens);

    // Base score from overlap
    let mut score = 0.0;
    score += t_body * 45.0;
    score += m_body * 35.0;
    score += h_body * 20.0;

    // Completeness bonus
    if title.is_some() {
        score += 5.0;
    }
    if meta.is_some() {
        score += 5.0;
    }
    if !h1.is_empty() {
        score += 5.0;
    }

    // Mismatch warnings (added after computing overlaps)
    if title.is_some() && t_body < 0.08 {
        warnings.push("Low overlap between title and body (possible topic mismatch).".to_string());
    }
    if meta.is_some() && m_body < 0.06 {
        warnings.push("Meta description does not match body content well.".to_string());
    }

    // Apply penalties based on warnings
    let mut final_score = score.clamp(0.0, 100.0);

    for w in &warnings {
        if w.starts_with("Missing or empty <title>") {
            final_score -= 15.0;
        }
        if w.starts_with("Missing meta description") {
            final_score -= 12.0;
        }
        if w.starts_with("Missing <h1>") {
            final_score -= 12.0;
        }
        if w.starts_with("Multiple <h1>") {
            final_score -= 6.0;
        }
        if w.starts_with("Content seems thin") {
            final_score -= 10.0;
        }
        if w.starts_with("Title is quite short") || w.starts_with("Title is quite long") {
            final_score -= 3.0;
        }
        if w.starts_with("Meta description is quite short")
            || w.starts_with("Meta description is quite long")
        {
            final_score -= 3.0;
        }
        if w.starts_with("Low overlap between title and body") {
            final_score -= 8.0;
        }
        if w.starts_with("Meta description does not match body content well") {
            final_score -= 8.0;
        }
    }

    let score_u8 = final_score.clamp(0.0, 100.0).round() as u8;

    Scored {
        score: score_u8,
        warnings,
        top_keywords: body_kw,
    }
}
