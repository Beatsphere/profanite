//! Unicode NFKC normalization.
//!
//! NFKC decomposes compatibility forms (fullwidth ASCII, ligatures like `ﬃ`,
//! mathematical alphanumerics) and recomposes them. This is where
//! `ＦＵＣＫ` becomes `FUCK`.
//!
//! Span handling: when one source char expands to multiple normalized chars
//! (e.g. `ﬃ` → `ffi`), every output char maps to the original char's span.

use unicode_normalization::UnicodeNormalization;

use super::NormalizedText;

pub(super) fn apply(input: NormalizedText) -> NormalizedText {
    let mut text = String::with_capacity(input.text.len());
    let mut spans = Vec::with_capacity(input.spans.len());
    for (idx, ch) in input.text.chars().enumerate() {
        let src_span = input.spans[idx];
        let mut expanded = false;
        for out_ch in ch.to_string().nfkc() {
            text.push(out_ch);
            spans.push(src_span);
            expanded = true;
        }
        // `nfkc()` always produces at least one char for any valid char, so
        // `expanded` will always be true — but guard anyway.
        debug_assert!(expanded, "NFKC produced no output for char");
    }
    NormalizedText { text, spans }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullwidth_uppercase_becomes_ascii() {
        let out = apply(NormalizedText::identity("ＦＵＣＫ"));
        assert_eq!(out.text, "FUCK");
    }

    #[test]
    fn ligature_expands() {
        let out = apply(NormalizedText::identity("ﬃ"));
        assert_eq!(out.text, "ffi");
        // All three output chars map to the single source char's span.
        let src_span = (0_usize, '\u{FB03}'.len_utf8());
        assert_eq!(out.spans, vec![src_span, src_span, src_span]);
    }

    #[test]
    fn ascii_is_untouched() {
        let out = apply(NormalizedText::identity("hello"));
        assert_eq!(out.text, "hello");
    }
}
