//! Strip invisible / bypass-enabling control characters.
//!
//! Three families get dropped here:
//!
//! 1. **Bidi overrides** (U+202A–U+202E, U+2066–U+2069, U+200E/U+200F).
//!    Used to hide profanity in display without affecting rendering.
//! 2. **Zero-width characters** (U+200B ZWSP, U+200C ZWNJ, U+200D ZWJ,
//!    U+FEFF BOM). Commonly inserted mid-word as a bypass: "shi<ZWSP>t".
//! 3. **Non-breaking / figure / narrow spaces** (U+00A0, U+2007, U+202F).
//!    They render as whitespace but aren't ASCII space, so neither the
//!    matcher's word-boundary check nor the separator stripper sees them.
//!    Dropping them entirely merges the surrounding letters so bypasses
//!    like "fu<NBSP>ck" normalize to "fuck".
//!
//! All are 1→0 transformations — span for the removed char is simply not
//! emitted. Remaining chars keep their original source spans.

use super::NormalizedText;

const STRIP: &[char] = &[
    // Bidi overrides
    '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}', '\u{2067}', '\u{2068}',
    '\u{2069}', '\u{200E}', '\u{200F}', // Zero-width / joiner controls
    '\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}',
    // Non-breaking / figure / narrow no-break spaces
    '\u{00A0}', '\u{2007}', '\u{202F}',
];

pub(super) fn apply(input: NormalizedText) -> NormalizedText {
    let mut text = String::with_capacity(input.text.len());
    let mut spans = Vec::with_capacity(input.spans.len());
    for (idx, ch) in input.text.chars().enumerate() {
        if STRIP.contains(&ch) {
            continue;
        }
        text.push(ch);
        spans.push(input.spans[idx]);
    }
    NormalizedText { text, spans }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_rlo_preserving_remaining_spans() {
        let input = NormalizedText::identity("a\u{202E}b");
        let out = apply(input);
        assert_eq!(out.text, "ab");
        // 'a' at byte 0, 'b' at bytes 4-5 (after 3-byte RLO).
        assert_eq!(out.spans, vec![(0, 1), (4, 5)]);
    }

    #[test]
    fn passthrough_without_controls() {
        let nt = apply(NormalizedText::identity("hello"));
        assert_eq!(nt.text, "hello");
    }
}
