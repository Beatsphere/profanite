//! Aho-Corasick–backed multi-pattern matcher.
//!
//! The matcher scans *normalized* text. Spans it emits are translated back to
//! byte offsets in the caller's original input via `NormalizedText::source_span`.

use aho_corasick::{AhoCorasick, AhoCorasickKind, MatchKind};

use crate::config::MatchMode;
use crate::data::{Category, WordEntry};
use crate::error::Error;
use crate::normalize::{self, NormalizedText};

/// A single profanity hit in the original input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index into the internal wordlist. Stable within a `Profanite` instance.
    pub word_id: usize,
    /// Byte span in the caller's original input.
    pub original_span: (usize, usize),
    /// Byte span in the normalized text used for matching.
    pub normalized_span: (usize, usize),
    pub category: Category,
    pub severity: u8,
}

#[derive(Debug)]
pub(crate) struct Matcher {
    pub(crate) words: Vec<WordEntry>,
    ac: AhoCorasick,
}

impl Matcher {
    pub(crate) fn new(words: Vec<WordEntry>) -> Result<Self, Error> {
        // Normalize each stored pattern with the same pipeline the scanner
        // uses, so user-supplied uppercase / non-ASCII patterns match
        // correctly against normalized input.
        let patterns: Vec<String> = words
            .iter()
            .map(|w| normalize::normalize_pattern(&w.word))
            .collect();
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .kind(Some(AhoCorasickKind::DFA))
            .ascii_case_insensitive(false)
            .build(&patterns)
            .map_err(|e| Error::AutomatonBuild(e.to_string()))?;
        Ok(Self { words, ac })
    }

    pub(crate) fn scan(&self, norm: &NormalizedText, mode: MatchMode) -> Vec<Match> {
        let haystack = norm.text.as_bytes();
        let mut hits = Vec::new();
        for m in self.ac.find_iter(&norm.text) {
            let word_id = m.pattern().as_usize();
            let entry = &self.words[word_id];
            let ns = m.start();
            let ne = m.end();

            // Per-word strict flag overrides the global mode: strict entries
            // always match (slurs/fixed strings), non-strict entries respect
            // the caller's MatchMode.
            let needs_boundary = !entry.strict && matches!(mode, MatchMode::WordBoundary);
            if needs_boundary && !has_word_boundary(haystack, ns, ne) {
                continue;
            }

            let (os, oe) = norm.source_span(ns, ne);
            hits.push(Match {
                word_id,
                original_span: (os, oe),
                normalized_span: (ns, ne),
                category: entry.category,
                severity: entry.severity,
            });
        }
        hits
    }
}

/// True if the byte range `[start, end)` in `haystack` is flanked by non-word
/// bytes (or string boundaries). "Word byte" here is ASCII `[A-Za-z0-9]`; we
/// rely on the normalization pipeline to have stripped decorative characters
/// so this check stays cheap.
///
/// Note on non-ASCII boundaries: a non-ASCII char's UTF-8 bytes all have the
/// high bit set, so they read as non-word under this test. That's fine for
/// English matching against English words; multi-language M3 work will revisit
/// this if needed.
fn has_word_boundary(haystack: &[u8], start: usize, end: usize) -> bool {
    let left_ok = start == 0 || !is_word_byte(haystack[start - 1]);
    let right_ok = end == haystack.len() || !is_word_byte(haystack[end]);
    left_ok && right_ok
}

#[inline]
fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
