# IntentCheck

IntentCheck is a command-line SEO and content consistency analyzer written in Rust.

It evaluates how well a webpage’s content aligns with its title, meta description, and main heading. The goal is to detect mismatches, weak structure, thin content, and low semantic alignment — especially relevant in the age of AI-driven search.

---

## Why IntentCheck?

Modern search engines and AI assistants increasingly rely on semantic consistency.

Many websites:

* have titles that do not match their actual content
* use weak or missing meta descriptions
* contain thin or unfocused text
* lack structural clarity

IntentCheck provides a lightweight, local, developer-friendly way to detect these issues before publishing.

---

## Features

* Analyze a single HTML file
* Analyze an entire folder (static site)
* URL support (analyze live websites)
* Semantic overlap scoring (Title / Meta / H1 vs Body)
* Thin content detection
* Length validation for Title and Meta description
* Folder-wide keyword theme detection
* Outlier page detection (low alignment with site theme)
* CI-friendly `--fail-under` option
* JSON export
* Markdown report export

---

## Installation

Clone the repository:

```bash
git clone <your-repo-url>
cd intentcheck
```

Build the project:

```bash
cargo build --release
```

---

## Usage

### Analyze a single file

```bash
cargo run -- analyze page.html
```

### Analyze a folder

```bash
cargo run -- analyze site/
```

### Analyze a live URL

```bash
cargo run -- analyze https://example.com
```

---

## Options

| Option         | Description                                  |
| -------------- | -------------------------------------------- |
| `--min-words`  | Minimum word count threshold                 |
| `--fail-under` | Exit with code 1 if score is below threshold |
| `--top`        | Number of worst pages to show in folder mode |
| `--json`       | Export report as JSON                        |
| `--report-md`  | Export report as Markdown                    |

Example:

```bash
cargo run -- analyze site --min-words 150 --fail-under 60 --report-md report.md
```

---

## Scoring Model

Score is based on:

* Overlap between Title and Body
* Overlap between Meta Description and Body
* Overlap between H1 and Body
* Structural completeness
* Penalties for warnings (thin content, missing tags, mismatch)

Score range: **0–100**

---

## Example Output

```
IntentCheck Report
Input: page.html
Score: 87/100

Warnings:
- Meta description is quite short (45 characters).
```

---

## Target Users

* Developers building static websites
* Students learning SEO basics
* Small website owners
* Early-stage projects
* Content editors validating consistency

---

## Project Context

This tool was developed as part of a Digital Business course project.
It combines technical implementation (Rust CLI tool) with real-world business value.
