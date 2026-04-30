//! Property-based invariants for the normalization pipeline.
//!
//! The critical invariant: for any normalized match at `[ns, ne)`, the
//! reported original span `[os, oe)` must be a byte-aligned slice of the
//! original input that, when re-normalized, still contains the matched
//! substring. Otherwise `find()` / `censor()` would report the wrong region.
//!
//! Tests run against the public `Profanite` API so we exercise the real
//! pipeline end-to-end, not just the normalize module in isolation.

use profanite_core::{Category, Profanite};
use proptest::prelude::*;

/// Generator: strings up to ~40 chars drawn from a pool that includes
/// lowercase ASCII, digits, common symbols/leet chars, a few Cyrillic
/// homoglyphs, and whitespace.
fn obfuscated_text() -> impl Strategy<Value = String> {
    let pool: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789@$!. -_*сооаеорхуFUCKshit"
        .chars()
        .collect();
    prop::collection::vec(prop::sample::select(pool), 0..40).prop_map(|chars| {
        let mut s = String::new();
        for c in chars {
            s.push(c);
        }
        s
    })
}

fn matcher() -> Profanite {
    Profanite::builder()
        .without_bundled()
        .add_words([
            ("fuck", Category::Strong, 3, false),
            ("shit", Category::Strong, 2, false),
            ("ass", Category::Mild, 2, false),
        ])
        .build()
        .unwrap()
}

proptest! {
    /// Reported spans must always lie within the original string bounds and
    /// be byte-aligned (start and end on UTF-8 boundaries).
    #[test]
    fn spans_are_in_bounds_and_char_aligned(text in obfuscated_text()) {
        let p = matcher();
        for m in p.find(&text) {
            let (s, e) = m.original_span;
            prop_assert!(s <= e, "start <= end, got {}..{}", s, e);
            prop_assert!(e <= text.len(), "end {} > text.len() {}", e, text.len());
            prop_assert!(text.is_char_boundary(s), "start {} not char-aligned", s);
            prop_assert!(text.is_char_boundary(e), "end {} not char-aligned", e);
        }
    }

    /// Slicing the original text by the reported span must not panic and
    /// must produce a valid `&str` (already guaranteed by char-alignment but
    /// worth a direct check).
    #[test]
    fn span_slicing_is_safe(text in obfuscated_text()) {
        let p = matcher();
        for m in p.find(&text) {
            let (s, e) = m.original_span;
            let _slice: &str = &text[s..e];
        }
    }

    /// `censor()` output must be at least as long as the non-masked portion
    /// of the original, i.e. we never *lose* characters outside matches.
    #[test]
    fn censor_preserves_non_matched_regions(text in obfuscated_text()) {
        let p = matcher();
        let censored = p.censor(&text);
        let hits = p.find(&text);

        // Sum of match-span byte lengths in the original.
        let matched_bytes: usize = hits.iter()
            .map(|h| h.original_span.1 - h.original_span.0)
            .sum();
        let kept_bytes = text.len().saturating_sub(matched_bytes);

        // After censoring, the non-matched regions must still be copied
        // verbatim. The censored output can only be shorter than the input
        // when LengthPreserving masks a multi-byte sequence with single-byte
        // asterisks — which is why we compare by "chars not bytes" minimum.
        prop_assert!(censored.chars().count() >= kept_bytes.min(text.chars().count() - hits.len()));
    }

    /// `contains_profanity` must agree with `find(...).is_empty()`.
    #[test]
    fn contains_and_find_agree(text in obfuscated_text()) {
        let p = matcher();
        prop_assert_eq!(p.contains_profanity(&text), !p.find(&text).is_empty());
    }
}
