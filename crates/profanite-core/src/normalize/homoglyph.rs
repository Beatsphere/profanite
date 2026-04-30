//! Homoglyph substitution: map visually-similar non-ASCII letters to their
//! ASCII lookalikes.
//!
//! Covers the most abused confusables (Cyrillic, Greek) for Latin-alphabet
//! matching. The substitution is 1→1 so spans carry over directly.
//!
//! This list is intentionally small and conservative. A full ICU confusables
//! table contains thousands of entries and would over-normalize legitimate
//! non-English text.

use super::NormalizedText;

/// Returns the ASCII letter this char should collapse to, or `None` to leave
/// it alone.
fn lookup(ch: char) -> Option<char> {
    match ch {
        // Cyrillic lowercase lookalikes
        'а' => Some('a'),
        'в' => Some('b'),
        'с' => Some('c'),
        'е' => Some('e'),
        'ѕ' => Some('s'),
        'і' => Some('i'),
        'ј' => Some('j'),
        'к' => Some('k'),
        'м' => Some('m'),
        'н' => Some('h'),
        'о' => Some('o'),
        'р' => Some('p'),
        'т' => Some('t'),
        'х' => Some('x'),
        'у' => Some('y'),
        'ѡ' => Some('w'),
        // Cyrillic uppercase (redundant after ASCII casefold but cheap)
        'А' => Some('a'),
        'В' => Some('b'),
        'С' => Some('c'),
        'Е' => Some('e'),
        'К' => Some('k'),
        'М' => Some('m'),
        'Н' => Some('h'),
        'О' => Some('o'),
        'Р' => Some('p'),
        'Т' => Some('t'),
        'Х' => Some('x'),
        'У' => Some('y'),
        // Greek
        'α' => Some('a'),
        'β' => Some('b'),
        'γ' => Some('y'),
        'ε' => Some('e'),
        'ι' => Some('i'),
        'κ' => Some('k'),
        'μ' => Some('u'),
        'ν' => Some('v'),
        'ο' => Some('o'),
        'ρ' => Some('p'),
        'τ' => Some('t'),
        'υ' => Some('u'),
        'χ' => Some('x'),
        'ω' => Some('w'),
        // Common diacriticals folded to base letter
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => Some('a'),
        'è' | 'é' | 'ê' | 'ë' => Some('e'),
        'ì' | 'í' | 'î' | 'ï' => Some('i'),
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => Some('o'),
        'ù' | 'ú' | 'û' | 'ü' => Some('u'),
        'ñ' => Some('n'),
        'ç' => Some('c'),
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
    fn cyrillic_lookalikes_become_ascii() {
        let out = apply(NormalizedText::identity("fцck"));
        // 'ц' is not a lookalike in our table — intentionally, it's not
        // visually identical to any ASCII letter. So "fцck" stays as-is
        // through homoglyph, and NFKC won't help either. This test is a
        // reminder that our table is conservative.
        assert_eq!(out.text, "fцck");

        // 'с' (Cyrillic es) IS a lookalike → ASCII 'c'.
        let out = apply(NormalizedText::identity("fuсk"));
        assert_eq!(out.text, "fuck");
    }

    #[test]
    fn diacritics_folded() {
        let out = apply(NormalizedText::identity("café"));
        assert_eq!(out.text, "cafe");
    }
}
