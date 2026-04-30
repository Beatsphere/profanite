//! Corpus loading. All corpora share a single JSONL schema:
//!
//! ```json
//! {"text": "...", "label": "profane" | "benign", "category": "leet", "lang": "en", "note": "..."}
//! ```
//!
//! - `text`: the input to feed to the filter.
//! - `label`: ground-truth — `profane` means `contains_profanity` SHOULD return true.
//! - `category`: optional sub-bucket ("leet", "homoglyph", "concat", ...). Used for per-category stats.
//! - `lang`: optional language hint, defaulting to "en". Determines which bundle to load.
//! - `note`: optional human description, ignored by the runner.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Label {
    Profane,
    Benign,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Case {
    pub text: String,
    pub label: Label,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

pub fn load_jsonl(path: &Path) -> Result<Vec<Case>> {
    let file = File::open(path).with_context(|| format!("opening corpus {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut cases = Vec::new();
    for (lineno, line) in reader.lines().enumerate() {
        let line =
            line.with_context(|| format!("reading {} line {}", path.display(), lineno + 1))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let case: Case = serde_json::from_str(trimmed).with_context(|| {
            format!(
                "parsing {} line {}: {}",
                path.display(),
                lineno + 1,
                trimmed
            )
        })?;
        cases.push(case);
    }
    Ok(cases)
}
