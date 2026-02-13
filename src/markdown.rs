use crate::report::{FolderReport, Report};

pub fn single_to_markdown(r: &Report) -> String {
    let mut out = String::new();
    out.push_str("# IntentCheck Report\n\n");
    out.push_str(&format!("**Input:** `{}`\n\n", r.input));
    out.push_str(&format!("**Score:** **{}/100**\n\n", r.score));

    out.push_str(&format!(
        "**Title:** {}\n\n",
        r.title.clone().unwrap_or_else(|| "_(missing)_".to_string())
    ));
    out.push_str(&format!(
        "**Meta description:** {}\n\n",
        r.meta_description
            .clone()
            .unwrap_or_else(|| "_(missing)_".to_string())
    ));

    out.push_str(&format!(
        "**H1:** {}\n\n",
        if r.h1.is_empty() {
            "_(missing)_".to_string()
        } else {
            r.h1.join(" | ")
        }
    ));

    if !r.top_keywords.is_empty() {
        out.push_str("## Top keywords\n\n");
        for k in &r.top_keywords {
            out.push_str(&format!("- {}\n", k));
        }
        out.push('\n');
    }

    out.push_str("## Warnings\n\n");
    if r.warnings.is_empty() {
        out.push_str("- None ✅\n");
    } else {
        for w in &r.warnings {
            out.push_str(&format!("- {}\n", w));
        }
    }

    out
}

pub fn folder_to_markdown(fr: &FolderReport, top: usize) -> String {
    let mut out = String::new();
    out.push_str("# IntentCheck Folder Report\n\n");
    out.push_str(&format!("**Root:** `{}`\n\n", fr.root));
    out.push_str(&format!(
        "**Pages:** {}  \n**Average score:** {:.1}  \n**Min:** {}  \n**Max:** {}\n\n",
        fr.pages.len(),
        fr.average_score,
        fr.min_score,
        fr.max_score
    ));

    if !fr.site_keywords.is_empty() {
        out.push_str("## Site keywords (overall theme)\n\n");
        out.push_str(&fr.site_keywords.join(", "));
        out.push_str("\n\n");
    }

    if !fr.outliers.is_empty() {
        out.push_str("## Outliers (pages with low alignment to site theme)\n\n");
        for o in &fr.outliers {
            out.push_str(&format!(
                "- `{}` — score **{}**, alignment **{:.2}**\n",
                o.input, o.score, o.alignment
            ));
        }
        out.push('\n');
    }

    let mut pages = fr.pages.clone();
    pages.sort_by_key(|p| p.score);

    out.push_str("## Worst pages\n\n");
    for p in pages.iter().take(top) {
        out.push_str(&format!("- `{}` — **{} / 100**\n", p.input, p.score));
    }
    out.push('\n');

    out.push_str("## Pages\n\n");
    for p in &fr.pages {
        out.push_str(&format!("### `{}`\n\n", p.input));
        out.push_str(&format!("Score: **{} / 100**\n\n", p.score));
        if !p.warnings.is_empty() {
            out.push_str("Warnings:\n");
            for w in &p.warnings {
                out.push_str(&format!("- {}\n", w));
            }
            out.push('\n');
        } else {
            out.push_str("Warnings: None ✅\n\n");
        }
    }

    out
}
