mod analyze;
mod cli;
mod extract;
mod report;
mod score;
mod markdown;


use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
        input,
        json,
        fail_under,
        min_words,
        top,
        report_md
    } => {

            let result = analyze::analyze_path(&input, min_words)?;

            match result {
                analyze::AnalyzeResult::Single(r) => {
                    print_single(&r);

                    if let Some(path) = &report_md {
                        let md = markdown::single_to_markdown(&r);
                        std::fs::write(path, md)?;
                        println!("Saved Markdown report.");
                    }


                    if let Some(out) = json {
                        let s = serde_json::to_string_pretty(&r)?;
                        std::fs::write(out, s)?;
                        println!("Saved JSON report.");
                    }

                    if let Some(th) = fail_under {
                        if r.score < th {
                            println!("Fail-under triggered: score {} is below {}.", r.score, th);
                            std::process::exit(1);
                        }
                    }
                }

                analyze::AnalyzeResult::Folder(folder) => {
                    println!("IntentCheck Folder Report");
                    println!("Root: {}", folder.root);
                    println!(
                        "Pages: {} | Avg: {:.1} | Min: {} | Max: {}",
                        folder.pages.len(),
                        folder.average_score,
                        folder.min_score,
                        folder.max_score
                    );

                    if let Some(path) = &report_md {
                        let md = markdown::folder_to_markdown(&folder, top);
                        std::fs::write(path, md)?;
                        println!("Saved Markdown folder report.");
                    }


                    // show worst 5 pages
                    let mut pages = folder.pages.clone();
                    pages.sort_by_key(|p| p.score);
                    println!("Worst pages:");
                    for p in pages.iter().take(top) {

                        println!("- {}  ({} / 100)", p.input, p.score);
                    }

                    if let Some(out) = json {
                        let s = serde_json::to_string_pretty(&folder)?;
                        std::fs::write(out, s)?;
                        println!("Saved JSON folder report.");
                    }

                    if let Some(th) = fail_under {
                        let mut below_pages: Vec<&report::Report> =
                            folder.pages.iter().filter(|p| p.score < th).collect();
                        below_pages.sort_by_key(|p| p.score);

                        if !below_pages.is_empty() {
                            println!(
                                "Fail-under triggered: {} page(s) below {}.",
                                below_pages.len(),
                                th
                            );
                            println!("Examples:");
                            for p in below_pages.iter().take(top) {
                                println!("- {}  ({} / 100)", p.input, p.score);
                            }
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_single(r: &report::Report) {
    println!("IntentCheck Report");
    println!("Input: {}", r.input);
    println!("Score: {}/100", r.score);

    if let Some(t) = &r.title {
        println!("Title: {}", t);
    }
    if let Some(m) = &r.meta_description {
        println!("Meta description: {}", m);
    }
    if !r.h1.is_empty() {
        println!("H1: {}", r.h1.join(" | "));
    }

    if !r.top_keywords.is_empty() {
        println!("Top keywords: {}", r.top_keywords.join(", "));
    }

    if r.warnings.is_empty() {
        println!("Warnings: none ✅");
    } else {
        println!("Warnings:");
        for w in &r.warnings {
            println!("- {}", w);
        }
    }
}
