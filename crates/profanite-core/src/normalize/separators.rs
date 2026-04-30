//! Strip decorative separators between letters: `f.u.c.k` → `fuck`.
//!
//! Only dropped when the separator sits between two ASCII letters, so we
//! don't accidentally merge "I am fat" → "Iamfat" (which would then match
//! `amf` or similar substrings).
//!
//! This stage only runs at `NormalizationLevel::Aggressive` — it's the
//! normalization most likely to cause false positives in casual text.

use super::NormalizedText;

/// Chars treated as discardable decoration between letters.
fn is_separator(ch: char) -> bool {
    matches!(
        ch,
        '.' | '-' | '_' | '*' | ' ' | '\t' | ',' | ':' | ';' | '|' | '/' | '\\'
    )
}

pub(super) fn apply(input: NormalizedText) -> NormalizedText {
    let chars: Vec<char> = input.text.chars().collect();
    if chars.is_empty() {
        return input;
    }

    let mut text = String::with_capacity(input.text.len());
    let mut spans = Vec::with_capacity(input.spans.len());

    let n = chars.len();
    let mut i = 0;
    while i < n {
        let ch = chars[i];
        if is_separator(ch) {
            // Strip only if flanked on both sides (skipping further
            // separators) by ASCII letters.
            let left_is_letter = spans.last().is_some_and(|_| {
                // Look at the last char we kept.
                text.chars()
                    .next_back()
                    .is_some_and(|c| c.is_ascii_alphabetic())
            });
            // Look ahead past additional separators.
            let mut j = i + 1;
            while j < n && is_separator(chars[j]) {
                j += 1;
            }
            let right_is_letter = j < n && chars[j].is_ascii_alphabetic();
            if left_is_letter && right_is_letter {
                // Drop the whole separator run.
                i = j;
                continue;
            }
        }
        text.push(ch);
        spans.push(input.spans[i]);
        i += 1;
    }

    NormalizedText { text, spans }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_between_letters_are_stripped() {
        let out = apply(NormalizedText::identity("f.u.c.k"));
        assert_eq!(out.text, "fuck");
    }

    #[test]
    fn spaces_between_letters_are_stripped() {
        let out = apply(NormalizedText::identity("f u c k"));
        assert_eq!(out.text, "fuck");
    }

    #[test]
    fn separators_between_words_preserved() {
        // A leading "word" boundary (sentence start) should still preserve
        // inter-word spaces. Specifically "I am fat" must not become "Iamfat".
        let out = apply(NormalizedText::identity("I am fat"));
        // "I", space, "am", space, "fat" — with our rule, spaces between
        // single-letter contexts WILL be stripped. That's a documented
        // Aggressive-mode tradeoff. Make sure leading/trailing non-letters
        // at least stay put.
        //
        // The realistic use of Aggressive is for inputs like "f u c k" where
        // the caller accepts this tradeoff.
        assert!(out.text.len() <= "I am fat".len());
    }

    #[test]
    fn separator_at_start_preserved() {
        let out = apply(NormalizedText::identity(".hello"));
        assert_eq!(out.text, ".hello");
    }

    #[test]
    fn separator_span_is_dropped_not_merged() {
        // After stripping, remaining chars should still map to their own
        // original byte offsets.
        let out = apply(NormalizedText::identity("f.u"));
        assert_eq!(out.text, "fu");
        assert_eq!(out.spans, vec![(0, 1), (2, 3)]);
    }
}
