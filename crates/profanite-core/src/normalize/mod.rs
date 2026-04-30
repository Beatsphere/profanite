//! Normalization pipeline with source-index mapping.
//!
//! Each stage is a pure function `NormalizedText -> NormalizedText`. The
//! entry point `normalize()` composes them according to the configured
//! [`NormalizationLevel`]:
//!
//! | Level        | Stages                                                                    |
//! |--------------|---------------------------------------------------------------------------|
//! | None         | identity                                                                  |
//! | Basic        | bidi-strip → NFKC → ASCII casefold → homoglyph → leetspeak → collapse     |
//! | Aggressive   | Basic + separator stripping                                               |
//!
//! Multigraph normalization (e.g. `ph`→`f`) is intentionally NOT done here;
//! instead the matcher handles such variants by storing both forms in the
//! wordlist. This avoids corrupting legitimate text.

mod casefold;
mod collapse;
mod homoglyph;
mod leetspeak;
mod nfkc;
mod separators;
mod strip_bidi;

use crate::config::NormalizationLevel;

/// Text after normalization, paired with a per-char mapping back to byte
/// offsets in the original input.
///
/// Invariant: `spans.len() == text.chars().count()`. The Nth entry in
/// `spans` is the half-open byte range `[start, end)` in the *original*
/// input that produced the Nth char of `text`.
#[derive(Debug, Clone)]
pub(crate) struct NormalizedText {
    pub text: String,
    pub spans: Vec<(usize, usize)>,
}

impl NormalizedText {
    /// Identity normalization: copy the input, one span per char.
    pub(crate) fn identity(input: &str) -> Self {
        let mut spans = Vec::with_capacity(input.len());
        for (byte_idx, ch) in input.char_indices() {
            spans.push((byte_idx, byte_idx + ch.len_utf8()));
        }
        Self {
            text: input.to_string(),
            spans,
        }
    }

    /// Map a half-open byte range in the normalized text back to the
    /// corresponding byte range in the original input.
    ///
    /// Spans covering multiple source chars report `(min_start, max_end)`
    /// across the covered entries — i.e. the tightest original substring
    /// that gave rise to the normalized slice.
    pub(crate) fn source_span(&self, norm_start: usize, norm_end: usize) -> (usize, usize) {
        let mut src_start = usize::MAX;
        let mut src_end = 0;
        let mut any = false;
        for (char_idx, (byte_idx, ch)) in self.text.char_indices().enumerate() {
            let char_end = byte_idx + ch.len_utf8();
            // Is this char inside [norm_start, norm_end)?
            if byte_idx >= norm_start && char_end <= norm_end {
                let (cs, ce) = self.spans[char_idx];
                if cs < src_start {
                    src_start = cs;
                }
                if ce > src_end {
                    src_end = ce;
                }
                any = true;
            }
            if char_end >= norm_end {
                break;
            }
        }
        if any {
            (src_start, src_end)
        } else {
            (norm_start, norm_end)
        }
    }
}

/// Normalize input according to the configured level.
pub(crate) fn normalize(input: &str, level: NormalizationLevel) -> NormalizedText {
    let nt = NormalizedText::identity(input);
    match level {
        NormalizationLevel::None => nt,
        NormalizationLevel::Basic => {
            let nt = strip_bidi::apply(nt);
            let nt = nfkc::apply(nt);
            let nt = casefold::apply(nt);
            let nt = homoglyph::apply(nt);
            let nt = leetspeak::apply(nt);
            collapse::apply(nt)
        }
        NormalizationLevel::Aggressive => {
            let nt = strip_bidi::apply(nt);
            let nt = nfkc::apply(nt);
            let nt = casefold::apply(nt);
            let nt = homoglyph::apply(nt);
            let nt = leetspeak::apply(nt);
            let nt = collapse::apply(nt);
            separators::apply(nt)
        }
    }
}

/// Normalize a pattern (wordlist entry) to the canonical form used by the
/// matcher. Uses `Basic` level — `Aggressive` would strip separators inside
/// multi-word patterns like "son of a bitch", which is wrong.
///
/// Returns the normalized text only; pattern span mapping isn't needed.
pub(crate) fn normalize_pattern(input: &str) -> String {
    normalize(input, NormalizationLevel::Basic).text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_preserves_input_and_spans() {
        let nt = NormalizedText::identity("hello");
        assert_eq!(nt.text, "hello");
        assert_eq!(nt.spans, vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
    }

    #[test]
    fn identity_handles_multibyte_chars() {
        let nt = NormalizedText::identity("hé");
        // 'h' is 1 byte, 'é' is 2 bytes.
        assert_eq!(nt.spans, vec![(0, 1), (1, 3)]);
    }

    #[test]
    fn source_span_maps_identity_back() {
        let nt = NormalizedText::identity("hello world");
        // "world" is at normalized bytes [6, 11).
        assert_eq!(nt.source_span(6, 11), (6, 11));
    }

    #[test]
    fn none_level_is_identity() {
        let nt = normalize("HeLLo", NormalizationLevel::None);
        assert_eq!(nt.text, "HeLLo");
    }
}
