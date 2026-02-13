use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "intentcheck", version, about = "Checks content consistency for SEO / AI search.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze a local HTML file
    Analyze {
        /// Path to an HTML file
        input: String,

        /// Output JSON report to this path (optional)
        #[arg(long)]
        json: Option<String>,

        /// Fail (exit code 1) if score is below this value (0-100)
        #[arg(long)]
        fail_under: Option<u8>,

        /// Minimum word count before we warn about thin content
        #[arg(long, default_value_t = 150)]
        min_words: usize,

        /// How many worst pages to show in folder mode
        #[arg(long, default_value_t = 5)]
        top: usize,

        /// Write a Markdown report to this file (optional)
        #[arg(long)]
        report_md: Option<String>,

    },
}
