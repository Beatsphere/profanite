//! ASCII-only casefold.
//!
//! `char::to_ascii_lowercase` is a 1→1 map so spans carry over unchanged.
//! Non-ASCII chars are left alone — full Unicode casefolding requires ICU
//! data we don't want to bundle. Homoglyph handling downstream catches
//! most non-ASCII variants anyway.

use super::NormalizedText;

pub(super) fn apply(mut input: NormalizedText) -> NormalizedText {
    // Walk the original text, rebuilding only ASCII bytes.
    let mut text = String::with_capacity(input.text.len());
    for ch in input.text.chars() {
        text.push(ch.to_ascii_lowercase());
    }
    input.text = text;
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases_ascii() {
        let out = apply(NormalizedText::identity("HeLLo"));
        assert_eq!(out.text, "hello");
    }

    #[test]
    fn preserves_non_ascii() {
        let out = apply(NormalizedText::identity("Café"));
        assert_eq!(out.text, "café");
    }

    #[test]
    fn spans_unchanged() {
        let original_spans = NormalizedText::identity("ABC").spans;
        let out = apply(NormalizedText::identity("ABC"));
        assert_eq!(out.spans, original_spans);
    }
}
