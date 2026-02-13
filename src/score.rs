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
    pairs.into_iter().take(n).map(|(w, _)| w.to_string()).collect()
}

// Coverage: how much of "small" is present in "big" (0..1).
fn coverage_ratio(small: &[String], big: &[String]) -> f32 {
    if small.is_empty() || big.is_empty() {
        return 0.0;
    }
    let set_small: HashSet<&str> = small.iter().map(|s| s.as_str()).collect();
    let set_big: HashSet<&str> = big.iter().map(|s| s.as_str()).collect();
    let inter = set_small.intersection(&set_big).count() as f32;
    inter / (set_small.len() as f32)
}

fn missing_keywords(needles: &[String], haystack: &[String], k: usize) -> Vec<String> {
    let set_h: HashSet<&str> = haystack.iter().map(|s| s.as_str()).collect();
    let mut out: Vec<String> = needles
        .iter()
        .filter(|t| !set_h.contains(t.as_str()))
        .take(k)
        .cloned()
        .collect();

    let mut seen: HashSet<String> = HashSet::new();
    out.retain(|x| seen.insert(x.clone()));
    out
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
    let title_str = title.unwrap_or("").trim();
    let meta_str = meta.unwrap_or("").trim();
    let h1_joined = h1.join(" ");

    // ---- IMPORTANT FIX ----
    // Some HTML extractors accidentally include <title>/<h1> inside the "body" text.
    // To prevent artificially high overlap, remove title/meta/h1 strings from body before scoring.
    let mut body_sanitized = body.to_lowercase();
    if !title_str.is_empty() {
        body_sanitized = body_sanitized.replace(&title_str.to_lowercase(), " ");
    }
    if !meta_str.is_empty() {
        body_sanitized = body_sanitized.replace(&meta_str.to_lowercase(), " ");
    }
    if !h1_joined.trim().is_empty() {
        body_sanitized = body_sanitized.replace(&h1_joined.to_lowercase(), " ");
    }

    let body_tokens = normalize(&body_sanitized);
    let body_kw = top_keywords(&body_tokens, 10);

    let title_tokens = if title_str.is_empty() { vec![] } else { normalize(title_str) };
    let meta_tokens  = if meta_str.is_empty()  { vec![] } else { normalize(meta_str) };
    let h1_tokens    = if h1.is_empty()        { vec![] } else { normalize(&h1_joined) };

    let t_cov = coverage_ratio(&title_tokens, &body_tokens);
    let m_cov = coverage_ratio(&meta_tokens, &body_tokens);
    let h_cov = coverage_ratio(&h1_tokens, &body_tokens);

    // ---- Warnings (detailed) ----
    let mut warnings: Vec<String> = vec![];

    if title_str.is_empty() {
        warnings.push("Missing <title>. Add a clear page title that matches the main topic.".to_string());
    }
    if meta_str.is_empty() {
        warnings.push("Missing meta description. Add a short summary of the page content (50–160 chars).".to_string());
    }
    if h1.is_empty() {
        warnings.push("Missing <h1>. Add one main heading that matches the page topic.".to_string());
    }
    if h1.len() > 1 {
        warnings.push(format!(
            "Multiple <h1> found ({}). Usually only one H1 is recommended.",
            h1.len()
        ));
    }

    if !title_str.is_empty() {
        let len = title_str.chars().count();
        if len < 15 {
            warnings.push(format!("Title is quite short ({} chars). Consider 15–60 chars.", len));
        } else if len > 60 {
            warnings.push(format!("Title is quite long ({} chars). Consider 15–60 chars.", len));
        }
    }
    if !meta_str.is_empty() {
        let len = meta_str.chars().count();
        if len < 50 {
            warnings.push(format!("Meta description is quite short ({} chars). Consider 50–160 chars.", len));
        } else if len > 160 {
            warnings.push(format!("Meta description is quite long ({} chars). Consider 50–160 chars.", len));
        }
    }

    if body_tokens.len() < min_words {
        warnings.push(format!(
            "Content seems thin ({} words found, recommended at least {}).",
            body_tokens.len(),
            min_words
        ));
    }

    if !title_tokens.is_empty() {
        warnings.push(format!("Title coverage in body: {:.0}%.", t_cov * 100.0));
    }
    if !meta_tokens.is_empty() {
        warnings.push(format!("Meta coverage in body: {:.0}%.", m_cov * 100.0));
    }
    if !h1_tokens.is_empty() {
        warnings.push(format!("H1 coverage in body: {:.0}%.", h_cov * 100.0));
    }

    if !title_tokens.is_empty() && t_cov < 0.30 {
        let miss = missing_keywords(&title_tokens, &body_tokens, 5);
        if !miss.is_empty() {
            warnings.push(format!(
                "Title/body mismatch: body is missing key title terms like: {}.",
                miss.join(", ")
            ));
        } else {
            warnings.push("Title seems weakly reflected in the body (low coverage).".to_string());
        }
    }
    if !meta_tokens.is_empty() && m_cov < 0.25 {
        let miss = missing_keywords(&meta_tokens, &body_tokens, 5);
        if !miss.is_empty() {
            warnings.push(format!(
                "Meta/body mismatch: body is missing key meta terms like: {}.",
                miss.join(", ")
            ));
        } else {
            warnings.push("Meta description seems weakly reflected in the body (low coverage).".to_string());
        }
    }
    if !h1_tokens.is_empty() && h_cov < 0.35 {
        let miss = missing_keywords(&h1_tokens, &body_tokens, 5);
        if !miss.is_empty() {
            warnings.push(format!(
                "H1/body mismatch: body is missing key H1 terms like: {}.",
                miss.join(", ")
            ));
        } else {
            warnings.push("H1 seems weakly reflected in the body (low coverage).".to_string());
        }
    }

// ---- Scoring (rebalanced) ----
// Start from a small baseline for having a minimally structured page.
let mut score = 0.0;
if !title_str.is_empty() { score += 6.0; }
if !meta_str.is_empty()  { score += 5.0; }
if !h1.is_empty()        { score += 5.0; }

// Main score comes from relevance (coverage).
score += t_cov * 44.0; // title matters most
score += m_cov * 28.0; // meta next
score += h_cov * 22.0; // h1

// Depth bonus (0..6)
let depth = (body_tokens.len() as f32) / (min_words.max(1) as f32);
score += depth.clamp(0.0, 1.0) * 6.0;

// Strong mismatch penalties (this is what makes "bad" go low)
if !title_tokens.is_empty() && t_cov < 0.30 { score -= 18.0; }
if !meta_tokens.is_empty()  && m_cov < 0.25 { score -= 14.0; }
if !h1_tokens.is_empty()    && h_cov < 0.35 { score -= 14.0; }

// Extra harsh penalty for very low coverage
if !title_tokens.is_empty() && t_cov < 0.15 { score -= 10.0; }
if !meta_tokens.is_empty()  && m_cov < 0.12 { score -= 8.0; }
if !h1_tokens.is_empty()    && h_cov < 0.18 { score -= 8.0; }


    // Depth bonus
    let depth = (body_tokens.len() as f32) / (min_words.max(1) as f32);
    score += depth.clamp(0.0, 1.0) * 10.0;

    // Penalties
    if title_str.is_empty() { score -= 12.0; }
    if meta_str.is_empty()  { score -= 10.0; }
    if h1.is_empty()        { score -= 10.0; }
    if h1.len() > 1         { score -= 4.0; }

    if !title_tokens.is_empty() && t_cov < 0.15 { score -= 10.0; }
    if !meta_tokens.is_empty()  && m_cov < 0.12 { score -= 8.0; }
    if !h1_tokens.is_empty()    && h_cov < 0.18 { score -= 8.0; }

    if !title_str.is_empty() {
        let len = title_str.chars().count();
        if len < 15 || len > 60 { score -= 2.0; }
    }
    if !meta_str.is_empty() {
        let len = meta_str.chars().count();
        if len < 50 || len > 160 { score -= 2.0; }
    }

    let score_u8 = score.clamp(0.0, 100.0).round() as u8;

    Scored {
        score: score_u8,
        warnings,
        top_keywords: body_kw,
    }
}
