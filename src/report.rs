use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Report {
    pub input: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub h1: Vec<String>,
    pub score: u8,
    pub warnings: Vec<String>,
    pub top_keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Outlier {
    pub input: String,
    pub score: u8,
    pub alignment: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderReport {
    pub root: String,
    pub pages: Vec<Report>,
    pub average_score: f32,
    pub min_score: u8,
    pub max_score: u8,
    pub site_keywords: Vec<String>,
    pub outliers: Vec<Outlier>,
}
