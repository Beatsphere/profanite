//! Allowlist: suppress matches that fall inside caller-approved substrings.
//!
//! Integrators hit the Scunthorpe problem constantly. The allowlist is the
//! caller's primary escape hatch: if "scunthorpe" is on the list, any
//! profanity match whose span overlaps an occurrence of "scunthorpe" in the
//! original text is dropped.
//!
//! Matching is case-insensitive on ASCII (same as English normalization) and
//! operates on the original text, not normalized — the caller's intent is to
//! protect surface forms they know are safe.

use crate::matcher::Match;

#[derive(Debug, Default, Clone)]
pub(crate) struct Allowlist {
    entries: Vec<String>,
}

impl Allowlist {
    pub(crate) fn new(entries: Vec<String>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|s| s.to_ascii_lowercase())
                .collect(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drop any match whose original span overlaps an allowlisted substring.
    pub(crate) fn filter(&self, original: &str, hits: Vec<Match>) -> Vec<Match> {
        if self.is_empty() {
            return hits;
        }
        let lower = original.to_ascii_lowercase();
        hits.into_iter()
            .filter(|h| !self.covers(&lower, h.original_span))
            .collect()
    }

    fn covers(&self, lower: &str, (s, e): (usize, usize)) -> bool {
        for needle in &self.entries {
            let mut search_from = 0;
            while let Some(rel) = lower[search_from..].find(needle) {
                let abs_start = search_from + rel;
                let abs_end = abs_start + needle.len();
                // Overlap test for half-open intervals [s, e) vs [abs_start, abs_end).
                if abs_start < e && s < abs_end {
                    return true;
                }
                search_from = abs_start + 1;
            }
        }
        false
    }
}
