use crate::extract::extract_from_html;
use crate::report::{FolderReport, Outlier, Report};
use crate::score::score_page;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn analyze_html_string(input_label: &str, html: &str, min_words: usize) -> Result<Report> {
    let extracted = extract_from_html(html)?;

    let scored = score_page(
        extracted.title.as_deref(),
        extracted.meta_description.as_deref(),
        &extracted.h1,
        &extracted.body_text,
        min_words,
    );

    Ok(Report {
        input: input_label.to_string(),
        title: extracted.title,
        meta_description: extracted.meta_description,
        h1: extracted.h1,
        score: scored.score,
        warnings: scored.warnings,
        top_keywords: scored.top_keywords,
    })
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn analyze_url(url: &str, min_words: usize) -> Result<Report> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| anyhow!("Failed to fetch URL: {} ({})", url, e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow!("URL returned non-success status {}: {}", status, url));
    }

    let html = resp
        .text()
        .map_err(|e| anyhow!("Failed to read response body: {} ({})", url, e))?;

    analyze_html_string(url, &html, min_words)
}

fn compute_site_keywords(pages: &[Report], k: usize) -> Vec<String> {
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for p in pages {
        for kw in &p.top_keywords {
            *freq.entry(kw.as_str()).or_insert(0) += 1;
        }
    }
    let mut pairs: Vec<(&str, usize)> = freq.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    pairs.into_iter().take(k).map(|(w, _)| w.to_string()).collect()
}

fn alignment(page_keywords: &[String], site_keywords: &[String]) -> f32 {
    if page_keywords.is_empty() || site_keywords.is_empty() {
        return 0.0;
    }
    let site_set: std::collections::HashSet<&str> = site_keywords.iter().map(|s| s.as_str()).collect();
    let page_set: std::collections::HashSet<&str> = page_keywords.iter().map(|s| s.as_str()).collect();
    let inter = page_set.intersection(&site_set).count() as f32;
    let denom = site_set.len().max(page_set.len()) as f32;
    inter / denom
}

pub fn analyze_path(path: &str, min_words: usize) -> Result<AnalyzeResult> {
    if is_url(path) {
        let r = analyze_url(path, min_words)?;
        return Ok(AnalyzeResult::Single(r));
    }

    let p = Path::new(path);
    if p.is_file() {
        let html = fs::read_to_string(p)?;
        let r = analyze_html_string(path, &html, min_words)?;
        return Ok(AnalyzeResult::Single(r));
    }

    if !p.is_dir() {
        return Err(anyhow!("Input is not a file or directory: {}", path));
    }

    let mut pages: Vec<Report> = Vec::new();

    for entry in WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_path = entry.path();

        let is_html = file_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm"))
            .unwrap_or(false);

        if !is_html {
            continue;
        }

        let html = fs::read_to_string(file_path)?;
        let label = file_path.to_string_lossy().to_string();
        let report = analyze_html_string(&label, &html, min_words)?;
        pages.push(report);
    }

    if pages.is_empty() {
        return Err(anyhow!("No .html files found in folder: {}", path));
    }

    let min_score = pages.iter().map(|r| r.score).min().unwrap_or(0);
    let max_score = pages.iter().map(|r| r.score).max().unwrap_or(0);
    let sum: u32 = pages.iter().map(|r| r.score as u32).sum();
    let average_score = (sum as f32) / (pages.len() as f32);

    // Site theme keywords + outliers
    let site_keywords = compute_site_keywords(&pages, 10);
    let mut outliers: Vec<Outlier> = Vec::new();

    for p in &pages {
        let a = alignment(&p.top_keywords, &site_keywords);
        if a < 0.15 {
            outliers.push(Outlier {
                input: p.input.clone(),
                score: p.score,
                alignment: a,
            });
        }
    }
    outliers.sort_by(|x, y| x.alignment.partial_cmp(&y.alignment).unwrap());

    Ok(AnalyzeResult::Folder(FolderReport {
        root: path.to_string(),
        pages,
        average_score,
        min_score,
        max_score,
        site_keywords,
        outliers,
    }))
}

#[derive(Debug)]
pub enum AnalyzeResult {
    Single(Report),
    Folder(FolderReport),
}
