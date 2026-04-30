//! Collapse repeated characters: `fuuuck` → `fuck`, `shiiiit` → `shit`.
//!
//! Only runs of length ≥ 3 are collapsed to the first occurrence. Runs of 2
//! are preserved because many legitimate English words have doubled letters
//! (`book`, `running`, `letter`). Matching at run-length 2 would break those.
//!
//! Span handling: a collapsed run's output char spans from the run's first
//! source start to its last source end — so censoring the match covers every
//! original letter the user typed.

use super::NormalizedText;

const MIN_RUN: usize = 3;

pub(super) fn apply(input: NormalizedText) -> NormalizedText {
    let chars: Vec<char> = input.text.chars().collect();
    if chars.is_empty() {
        return input;
    }

    let mut text = String::with_capacity(input.text.len());
    let mut spans = Vec::with_capacity(input.spans.len());

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let mut run_end = i + 1;
        while run_end < chars.len() && chars[run_end] == ch {
            run_end += 1;
        }
        let run_len = run_end - i;
        if run_len >= MIN_RUN {
            // Collapse: emit the char once with a span covering the whole run.
            let span_start = input.spans[i].0;
            let span_end = input.spans[run_end - 1].1;
            text.push(ch);
            spans.push((span_start, span_end));
        } else {
            for (ch, span) in chars[i..run_end].iter().zip(&input.spans[i..run_end]) {
                text.push(*ch);
                spans.push(*span);
            }
        }
        i = run_end;
    }

    NormalizedText { text, spans }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_run_of_three() {
        let out = apply(NormalizedText::identity("fuuuck"));
        assert_eq!(out.text, "fuck");
    }

    #[test]
    fn preserves_double_letters() {
        // 'oo' in 'book' must not collapse.
        let out = apply(NormalizedText::identity("book"));
        assert_eq!(out.text, "book");
    }

    #[test]
    fn collapsed_span_covers_whole_run() {
        // "fuuuck" — the 'u' run spans source bytes 1..4.
        let out = apply(NormalizedText::identity("fuuuck"));
        // Collapsed chars: f(0..1), u(1..4), c(4..5), k(5..6)
        assert_eq!(out.spans, vec![(0, 1), (1, 4), (4, 5), (5, 6)]);
    }

    #[test]
    fn empty_input_is_noop() {
        let out = apply(NormalizedText::identity(""));
        assert_eq!(out.text, "");
    }
}
