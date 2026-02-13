use anyhow::Result;
use scraper::{Html, Selector};

pub struct Extracted {
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub h1: Vec<String>,
    pub body_text: String,
}

pub fn extract_from_html(html: &str) -> Result<Extracted> {
    let doc = Html::parse_document(html);

    let title_sel = Selector::parse("title").unwrap();
    let meta_sel = Selector::parse(r#"meta[name="description"]"#).unwrap();
    let h1_sel = Selector::parse("h1").unwrap();
    let body_sel = Selector::parse("body").unwrap();
    let main_sel = Selector::parse("main").unwrap();
    let article_sel = Selector::parse("article").unwrap();
    let nav_sel = Selector::parse("nav").unwrap();
    let footer_sel = Selector::parse("footer").unwrap();

    let title = doc
        .select(&title_sel)
        .next()
        .map(|n| n.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty());

    let meta_description = doc
        .select(&meta_sel)
        .next()
        .and_then(|n| n.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let h1 = doc
        .select(&h1_sel)
        .map(|n| n.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

fn node_text(el: scraper::element_ref::ElementRef) -> String {
    el.text().collect::<Vec<_>>().join(" ")
}

let body_text = if let Some(m) = doc.select(&main_sel).next() {
    node_text(m)
} else if let Some(a) = doc.select(&article_sel).next() {
    node_text(a)
} else {
    // fallback: take body text, but we will try to reduce noise a bit
    let mut txt = doc
        .select(&body_sel)
        .next()
        .map(node_text)
        .unwrap_or_default();

    // remove obvious navigation/footer text if present (cheap heuristic)
    if let Some(nav) = doc.select(&nav_sel).next() {
        let nav_txt = node_text(nav);
        if !nav_txt.is_empty() {
            txt = txt.replace(&nav_txt, " ");
        }
    }
    if let Some(footer) = doc.select(&footer_sel).next() {
        let footer_txt = node_text(footer);
        if !footer_txt.is_empty() {
            txt = txt.replace(&footer_txt, " ");
        }
    }

    txt
};




    Ok(Extracted {
        title,
        meta_description,
        h1,
        body_text,
    })
}
