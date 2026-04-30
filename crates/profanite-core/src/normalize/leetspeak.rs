//! Conservative leetspeak substitution.
//!
//! Digits and common symbols used as letter substitutes are mapped to their
//! letter equivalents. Aggressive combinations (`|_|` → u, `|\|` → n) are
//! intentionally not handled — they'd require multi-char context and would
//! generate false positives on legitimate text (URLs, math, code snippets).

use super::NormalizedText;

fn lookup(ch: char) -> Option<char> {
    match ch {
        '0' => Some('o'),
        '1' => Some('i'),
        '3' => Some('e'),
        '4' => Some('a'),
        '5' => Some('s'),
        '7' => Some('t'),
        '8' => Some('b'),
        '9' => Some('g'),
        '@' => Some('a'),
        '$' => Some('s'),
        '!' => Some('i'),
        _ => None,
    }
}

pub(super) fn apply(input: NormalizedText) -> NormalizedText {
    let mut text = String::with_capacity(input.text.len());
    let mut spans = Vec::with_capacity(input.spans.len());
    for (idx, ch) in input.text.chars().enumerate() {
        let src = input.spans[idx];
        match lookup(ch) {
            Some(mapped) => {
                text.push(mapped);
                spans.push(src);
            }
            None => {
                text.push(ch);
                spans.push(src);
            }
        }
    }
    NormalizedText { text, spans }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_substituted() {
        let out = apply(NormalizedText::identity("f0ck"));
        assert_eq!(out.text, "fock");
    }

    #[test]
    fn symbols_substituted() {
        let out = apply(NormalizedText::identity("@ss"));
        assert_eq!(out.text, "ass");
    }

    #[test]
    fn unknown_chars_preserved() {
        let out = apply(NormalizedText::identity("hello"));
        assert_eq!(out.text, "hello");
    }
}
