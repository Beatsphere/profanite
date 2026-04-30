//! Builder API and the main `Profanite` type.

use crate::allowlist::Allowlist;
use crate::censor::{self, CensorStyle};
use crate::data::{bundled_for, Category, WordEntry};
use crate::error::Error;
use crate::lang::Lang;
use crate::matcher::{Match, Matcher};
use crate::normalize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// Require letter/digit boundaries on either side of the match.
    #[default]
    WordBoundary,
    /// Match anywhere, including inside longer words.
    Substring,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationLevel {
    None,
    #[default]
    Basic,
    Aggressive,
}

#[derive(Debug)]
pub struct Profanite {
    matcher: Matcher,
    allowlist: Allowlist,
    mask_char: char,
    censor_style: CensorStyle,
    match_mode: MatchMode,
    normalization: NormalizationLevel,
}

impl Profanite {
    pub fn builder() -> ProfaniteBuilder {
        ProfaniteBuilder::default()
    }

    /// Whether `text` contains any profanity.
    pub fn contains_profanity(&self, text: &str) -> bool {
        !self.find(text).is_empty()
    }

    /// Return every profanity hit in `text`.
    pub fn find(&self, text: &str) -> Vec<Match> {
        let norm = normalize::normalize(text, self.normalization);
        let hits = self.matcher.scan(&norm, self.match_mode);
        self.allowlist.filter(text, hits)
    }

    /// Return `text` with all profanities masked according to the configured style.
    pub fn censor(&self, text: &str) -> String {
        let hits = self.find(text);
        censor::apply(text, &hits, self.censor_style, self.mask_char)
    }
}

#[derive(Default)]
pub struct ProfaniteBuilder {
    languages: Vec<Lang>,
    added: Vec<WordEntry>,
    removed: Vec<String>,
    allowlist: Vec<String>,
    mask_char: Option<char>,
    censor_style: Option<CensorStyle>,
    match_mode: Option<MatchMode>,
    normalization: Option<NormalizationLevel>,
    skip_bundled: bool,
}

impl ProfaniteBuilder {
    pub fn language(mut self, lang: Lang) -> Self {
        self.languages.push(lang);
        self
    }

    pub fn languages(mut self, langs: impl IntoIterator<Item = Lang>) -> Self {
        self.languages.extend(langs);
        self
    }

    pub fn add_words<I, S>(mut self, words: I) -> Self
    where
        I: IntoIterator<Item = (S, Category, u8, bool)>,
        S: Into<String>,
    {
        self.added.extend(
            words
                .into_iter()
                .map(|(w, c, s, strict)| WordEntry::new(w, c, s, strict)),
        );
        self
    }

    pub fn remove_words<I, S>(mut self, words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.removed.extend(words.into_iter().map(Into::into));
        self
    }

    pub fn allowlist<I, S>(mut self, words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowlist.extend(words.into_iter().map(Into::into));
        self
    }

    pub fn mask_char(mut self, c: char) -> Self {
        self.mask_char = Some(c);
        self
    }

    pub fn censor_style(mut self, style: CensorStyle) -> Self {
        self.censor_style = Some(style);
        self
    }

    pub fn match_mode(mut self, mode: MatchMode) -> Self {
        self.match_mode = Some(mode);
        self
    }

    pub fn normalization(mut self, level: NormalizationLevel) -> Self {
        self.normalization = Some(level);
        self
    }

    /// Do not load any bundled wordlists. The caller must provide words via
    /// `add_words`. Useful for "bring your own dictionary" integrations.
    pub fn without_bundled(mut self) -> Self {
        self.skip_bundled = true;
        self
    }

    pub fn build(self) -> Result<Profanite, Error> {
        let mut words: Vec<WordEntry> = Vec::new();
        if !self.skip_bundled {
            let langs = if self.languages.is_empty() {
                vec![Lang::En]
            } else {
                self.languages
            };
            for lang in langs {
                words.extend(bundled_for(lang));
            }
        }
        words.extend(self.added);

        if !self.removed.is_empty() {
            let removed: std::collections::HashSet<String> = self
                .removed
                .into_iter()
                .map(|w| w.to_ascii_lowercase())
                .collect();
            words.retain(|w| !removed.contains(&w.word.to_ascii_lowercase()));
        }

        // Canonical ordering so automaton construction is deterministic
        // (affects LeftmostLongest tiebreaks).
        words.sort_by(|a, b| a.word.cmp(&b.word));
        words.dedup_by(|a, b| a.word == b.word);

        if words.is_empty() {
            return Err(Error::EmptyWordlist);
        }

        let matcher = Matcher::new(words)?;

        Ok(Profanite {
            matcher,
            allowlist: Allowlist::new(self.allowlist),
            mask_char: self.mask_char.unwrap_or('*'),
            censor_style: self.censor_style.unwrap_or_default(),
            match_mode: self.match_mode.unwrap_or_default(),
            normalization: self.normalization.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Isolated matcher with a fixed wordlist so tests don't drift when the
    /// bundled English list changes.
    fn p() -> Profanite {
        Profanite::builder()
            .without_bundled()
            .add_words([
                ("fuck", Category::Strong, 3, false),
                ("ass", Category::Mild, 2, false),
                ("slur", Category::Slur, 3, true),
            ])
            .build()
            .expect("builds")
    }

    #[test]
    fn contains_profanity_detects_simple_hit() {
        assert!(p().contains_profanity("what the fuck"));
    }

    #[test]
    fn contains_profanity_no_false_positive() {
        assert!(!p().contains_profanity("hello world"));
    }

    #[test]
    fn word_boundary_suppresses_scunthorpe() {
        let p = p();
        assert!(!p.contains_profanity("class assignment"));
        assert!(!p.contains_profanity("assassin"));
        assert!(!p.contains_profanity("glass"));
    }

    #[test]
    fn strict_word_matches_as_substring() {
        assert!(p().contains_profanity("slurring slurry slurs"));
    }

    #[test]
    fn substring_mode_allows_inside_words() {
        let p = Profanite::builder()
            .without_bundled()
            .add_words([("ass", Category::Mild, 2, false)])
            .match_mode(MatchMode::Substring)
            .build()
            .unwrap();
        assert!(p.contains_profanity("class"));
    }

    #[test]
    fn find_returns_correct_spans() {
        let hits = p().find("oh fuck that");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].original_span, (3, 7));
    }

    #[test]
    fn censor_default_is_length_preserving() {
        assert_eq!(p().censor("oh fuck that"), "oh **** that");
    }

    #[test]
    fn censor_with_first_last_style() {
        let p = Profanite::builder()
            .without_bundled()
            .add_words([("fuck", Category::Strong, 3, false)])
            .censor_style(CensorStyle::FirstLast)
            .build()
            .unwrap();
        assert_eq!(p.censor("oh fuck that"), "oh f**k that");
    }

    #[test]
    fn allowlist_suppresses_overlapping_match() {
        let p = Profanite::builder()
            .without_bundled()
            .add_words([("ass", Category::Mild, 2, false)])
            .match_mode(MatchMode::Substring)
            .allowlist(["scunthorpe-assoc"])
            .build()
            .unwrap();
        assert!(!p.contains_profanity("this Scunthorpe-Assoc membership"));
        assert!(p.contains_profanity("ass"));
    }

    #[test]
    fn empty_wordlist_errors() {
        let err = Profanite::builder().without_bundled().build().unwrap_err();
        assert!(matches!(err, Error::EmptyWordlist));
    }

    #[test]
    fn custom_mask_char() {
        let p = Profanite::builder()
            .without_bundled()
            .add_words([("fuck", Category::Strong, 3, false)])
            .mask_char('#')
            .build()
            .unwrap();
        assert_eq!(p.censor("oh fuck"), "oh ####");
    }

    // ---- Integration tests against the bundled English list ----

    #[test]
    fn bundled_english_catches_common_profanity() {
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("what the fuck"));
        assert!(p.contains_profanity("oh shit"));
        assert!(p.contains_profanity("you asshole"));
    }

    #[test]
    fn tier3_strict_words_catch_concat_bypass() {
        // Top-tier compounds (motherfucker, cocksucker, asshole, bullshit,
        // dickhead) are unambiguous anywhere they appear, so they match
        // even when embedded inside a longer token.
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("Hemoglomotherfuckerbin"));
        assert!(p.contains_profanity("Superbullshitasaurus"));
        assert!(p.contains_profanity("classcocksuckerroom"));
        assert!(p.contains_profanity("megaassholebot"));
    }

    #[test]
    fn tier1_2_words_still_respect_boundaries_after_tiering() {
        // Re-confirm that short/ambiguous words (ass, hell, damn, shit,
        // bitch) didn't get swept into strict mode by the re-tiering.
        let p = Profanite::builder().build().unwrap();
        for benign in [
            "Passing",
            "Hemoglobin",
            "classroom",
            "assignment",
            "hello",
            "shellfish",
            "massage",
            "bitchy-sounding-but-fake", // artificial; ensures "bitch" wasn't promoted
        ] {
            // "bitchy-sounding-but-fake" contains "bitch" but with a letter
            // on the right, so WordBoundary correctly suppresses it.
            assert!(!p.contains_profanity(benign), "should not flag {benign:?}");
        }
    }

    #[test]
    fn bundled_english_respects_word_boundaries() {
        let p = Profanite::builder().build().unwrap();
        // Scunthorpe-style false positives must not fire.
        assert!(!p.contains_profanity("classroom assignment"));
        assert!(!p.contains_profanity("Scunthorpe"));
        assert!(!p.contains_profanity("assess the situation"));
        assert!(!p.contains_profanity("hello there"));
        // "ass" inside common English words.
        for text in [
            "Passing",
            "passing",
            "grass",
            "mass",
            "harass",
            "compass",
            "embarrass",
            "assassin",
        ] {
            assert!(!p.contains_profanity(text), "should not flag {text:?}");
        }
    }

    #[test]
    fn bundled_english_match_metadata_is_populated() {
        let p = Profanite::builder().build().unwrap();
        let hits = p.find("this is fucking bad");
        assert_eq!(hits.len(), 1);
        let hit = &hits[0];
        assert_eq!(
            &"this is fucking bad"[hit.original_span.0..hit.original_span.1],
            "fucking"
        );
        assert_eq!(hit.category, Category::Strong);
        assert_eq!(hit.severity, 3);
    }

    // ---- M2 obfuscation-resistance end-to-end tests ----

    #[test]
    fn basic_normalization_catches_uppercase() {
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("What the FUCK"));
    }

    #[test]
    fn basic_normalization_catches_fullwidth() {
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("what the ＦＵＣＫ"));
    }

    #[test]
    fn basic_normalization_catches_leetspeak() {
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("@ss hat"));
        assert!(p.contains_profanity("bullsh1t"));
    }

    #[test]
    fn basic_normalization_catches_repeated_chars() {
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("what the fuuuck"));
        assert!(p.contains_profanity("shiiiit"));
    }

    #[test]
    fn basic_normalization_catches_homoglyphs() {
        // 'с' is Cyrillic es (U+0441) — visually identical to ASCII 'c'.
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("what the fuсk"));
    }

    #[test]
    fn basic_normalization_catches_bidi_hidden() {
        // A bidi RLO inserted mid-word must not hide the profanity.
        let p = Profanite::builder().build().unwrap();
        assert!(p.contains_profanity("fu\u{202E}ck"));
    }

    #[test]
    fn aggressive_normalization_catches_separators() {
        let p = Profanite::builder()
            .normalization(NormalizationLevel::Aggressive)
            .build()
            .unwrap();
        assert!(p.contains_profanity("f.u.c.k"));
        assert!(p.contains_profanity("f u c k"));
        assert!(p.contains_profanity("f-u-c-k"));
    }

    #[test]
    fn basic_normalization_does_not_strip_separators() {
        // Separators are Aggressive-only, so Basic must NOT catch "f.u.c.k".
        let p = Profanite::builder().build().unwrap();
        assert!(!p.contains_profanity("f.u.c.k"));
    }

    #[test]
    fn obfuscated_match_spans_point_at_original() {
        // "fuuuck" at byte 5 — original span should cover the full bypass attempt.
        let p = Profanite::builder().build().unwrap();
        let text = "omg, fuuuck";
        let hits = p.find(text);
        assert_eq!(hits.len(), 1);
        assert_eq!(
            &text[hits[0].original_span.0..hits[0].original_span.1],
            "fuuuck"
        );
    }

    #[test]
    fn obfuscated_censor_masks_full_original_span() {
        let p = Profanite::builder().build().unwrap();
        assert_eq!(p.censor("omg, fuuuck"), "omg, ******");
    }

    #[test]
    fn homoglyph_wordlist_entry_is_normalized() {
        // User supplies a pattern with a homoglyph; it must still match plain
        // ASCII input after normalization of both sides.
        let p = Profanite::builder()
            .without_bundled()
            .add_words([("fuсk", Category::Strong, 3, false)]) // Cyrillic 'с'
            .build()
            .unwrap();
        assert!(p.contains_profanity("what the fuck"));
    }
}
