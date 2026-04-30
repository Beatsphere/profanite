//! Output censoring strategies.

use crate::matcher::Match;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CensorStyle {
    /// Replace the match with `mask_char` repeated to match the char-count of
    /// the original span.
    #[default]
    LengthPreserving,
    /// Keep first and last char of the match; mask the middle. Matches of 2
    /// chars or fewer fall back to full masking.
    FirstLast,
    /// Replace the match with the fixed string `****`.
    FullMask,
    /// Replace the match with rotating grawlix symbols (`#@$%&!*`).
    Grawlix,
}

const GRAWLIX: &[char] = &['#', '@', '$', '%', '&', '!', '*'];

/// Render `original` with every match in `hits` replaced according to `style`.
///
/// `hits` is assumed to be sorted by `original_span.0` and non-overlapping.
/// `aho-corasick`'s `LeftmostLongest` mode guarantees that in practice.
pub(crate) fn apply(original: &str, hits: &[Match], style: CensorStyle, mask_char: char) -> String {
    if hits.is_empty() {
        return original.to_string();
    }
    let mut out = String::with_capacity(original.len());
    let mut cursor = 0;
    for hit in hits {
        let (s, e) = hit.original_span;
        if s < cursor {
            // Overlapping — skip. Shouldn't happen with LeftmostLongest but
            // be defensive rather than panic.
            continue;
        }
        out.push_str(&original[cursor..s]);
        let matched = &original[s..e];
        out.push_str(&mask(matched, style, mask_char));
        cursor = e;
    }
    out.push_str(&original[cursor..]);
    out
}

fn mask(segment: &str, style: CensorStyle, mask_char: char) -> String {
    let char_count = segment.chars().count();
    match style {
        CensorStyle::LengthPreserving => mask_char.to_string().repeat(char_count),
        CensorStyle::FullMask => "****".to_string(),
        CensorStyle::FirstLast if char_count <= 2 => mask_char.to_string().repeat(char_count),
        CensorStyle::FirstLast => {
            let mut chars = segment.chars();
            let first = chars.next().unwrap();
            let last = segment.chars().next_back().unwrap();
            let middle_count = char_count - 2;
            let mut out = String::new();
            out.push(first);
            for _ in 0..middle_count {
                out.push(mask_char);
            }
            out.push(last);
            out
        }
        CensorStyle::Grawlix => {
            let mut out = String::new();
            for i in 0..char_count {
                out.push(GRAWLIX[i % GRAWLIX.len()]);
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Category;

    fn fake_match(s: usize, e: usize) -> Match {
        Match {
            word_id: 0,
            original_span: (s, e),
            normalized_span: (s, e),
            category: Category::Mild,
            severity: 1,
        }
    }

    #[test]
    fn length_preserving_masks_full_span() {
        let out = apply(
            "hello badword!",
            &[fake_match(6, 13)],
            CensorStyle::LengthPreserving,
            '*',
        );
        assert_eq!(out, "hello *******!");
    }

    #[test]
    fn first_last_keeps_ends() {
        let out = apply(
            "hello badword!",
            &[fake_match(6, 13)],
            CensorStyle::FirstLast,
            '*',
        );
        assert_eq!(out, "hello b*****d!");
    }

    #[test]
    fn first_last_falls_back_for_short_words() {
        let out = apply("hi ab cd", &[fake_match(3, 5)], CensorStyle::FirstLast, '*');
        assert_eq!(out, "hi ** cd");
    }

    #[test]
    fn full_mask_uses_fixed_length() {
        let out = apply(
            "a badword b",
            &[fake_match(2, 9)],
            CensorStyle::FullMask,
            '*',
        );
        assert_eq!(out, "a **** b");
    }

    #[test]
    fn grawlix_rotates_symbols() {
        let out = apply(
            "xx badword yy",
            &[fake_match(3, 10)],
            CensorStyle::Grawlix,
            '*',
        );
        assert_eq!(out, "xx #@$%&!* yy");
    }

    #[test]
    fn multiple_matches_preserve_gaps() {
        let hits = vec![fake_match(0, 3), fake_match(8, 11)];
        let out = apply("bad and bad!", &hits, CensorStyle::LengthPreserving, '*');
        assert_eq!(out, "*** and ***!");
    }
}
