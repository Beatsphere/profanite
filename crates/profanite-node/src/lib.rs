//! Node.js bindings for profanite via napi-rs.
//!
//! The JavaScript surface area is deliberately minimal — a single
//! `Profanite` class with the three core methods. All configuration
//! goes through the constructor's options bag; the underlying Rust
//! builder is hidden.

#![deny(clippy::all)]

use napi::Error as NapiError;
use napi_derive::napi;
use profanite_core::{
    Category as CoreCategory, CensorStyle as CoreStyle, Lang, MatchMode as CoreMatchMode,
    NormalizationLevel, Profanite as CoreProfanite,
};

fn map_err(e: profanite_core::Error) -> NapiError {
    NapiError::from_reason(e.to_string())
}

fn parse_lang(s: &str) -> Result<Lang, NapiError> {
    match s {
        "en" => Ok(Lang::En),
        "es" => Ok(Lang::Es),
        "hi" => Ok(Lang::Hi),
        "fr" => Ok(Lang::Fr),
        "de" => Ok(Lang::De),
        other => Err(NapiError::from_reason(format!(
            "unknown language code: {other} (expected one of: en, es, hi, fr, de)"
        ))),
    }
}

fn parse_normalization(s: &str) -> Result<NormalizationLevel, NapiError> {
    match s.to_lowercase().as_str() {
        "none" => Ok(NormalizationLevel::None),
        "basic" => Ok(NormalizationLevel::Basic),
        "aggressive" => Ok(NormalizationLevel::Aggressive),
        other => Err(NapiError::from_reason(format!(
            "unknown normalization level: {other} (expected: none, basic, aggressive)"
        ))),
    }
}

fn parse_match_mode(s: &str) -> Result<CoreMatchMode, NapiError> {
    match s.to_lowercase().as_str() {
        "wordboundary" | "word_boundary" | "word-boundary" => Ok(CoreMatchMode::WordBoundary),
        "substring" => Ok(CoreMatchMode::Substring),
        other => Err(NapiError::from_reason(format!(
            "unknown match mode: {other} (expected: wordBoundary, substring)"
        ))),
    }
}

fn parse_censor_style(s: &str) -> Result<CoreStyle, NapiError> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "lengthpreserving" => Ok(CoreStyle::LengthPreserving),
        "firstlast" => Ok(CoreStyle::FirstLast),
        "fullmask" => Ok(CoreStyle::FullMask),
        "grawlix" => Ok(CoreStyle::Grawlix),
        other => Err(NapiError::from_reason(format!(
            "unknown censor style: {other} (expected: lengthPreserving, firstLast, fullMask, grawlix)"
        ))),
    }
}

fn parse_category(s: &str) -> Result<CoreCategory, NapiError> {
    match s.to_lowercase().as_str() {
        "mild" => Ok(CoreCategory::Mild),
        "strong" => Ok(CoreCategory::Strong),
        "sexual" => Ok(CoreCategory::Sexual),
        "slur" => Ok(CoreCategory::Slur),
        "slang" => Ok(CoreCategory::Slang),
        other => Err(NapiError::from_reason(format!(
            "unknown category: {other} (expected: mild, strong, sexual, slur, slang)"
        ))),
    }
}

fn category_name(c: CoreCategory) -> &'static str {
    match c {
        CoreCategory::Mild => "mild",
        CoreCategory::Strong => "strong",
        CoreCategory::Sexual => "sexual",
        CoreCategory::Slur => "slur",
        CoreCategory::Slang => "slang",
    }
}

/// Options bag accepted by `new Profanite(options?)`.
#[napi(object)]
pub struct ProfaniteOptions {
    /// Two-letter language codes: "en", "es", "hi", "fr", "de".
    /// Defaults to ["en"] when omitted.
    pub languages: Option<Vec<String>>,
    /// "none" | "basic" | "aggressive". Default "basic".
    pub normalization: Option<String>,
    /// "wordBoundary" | "substring". Default "wordBoundary".
    pub match_mode: Option<String>,
    /// "lengthPreserving" | "firstLast" | "fullMask" | "grawlix". Default "lengthPreserving".
    pub censor_style: Option<String>,
    /// Single character used to mask censored text. Default "*".
    pub mask_char: Option<String>,
    /// Additional words to include. Each entry is a tuple of
    /// { word, category, severity, strict }.
    pub add_words: Option<Vec<CustomWord>>,
    /// Words to remove from the bundled list (case-insensitive).
    pub remove_words: Option<Vec<String>>,
    /// Substrings where matches should be suppressed (case-insensitive).
    pub allowlist: Option<Vec<String>>,
    /// If true, start with an empty wordlist (no bundled languages). The
    /// caller must then populate via `addWords`.
    pub without_bundled: Option<bool>,
}

/// User-supplied wordlist entry.
#[napi(object)]
pub struct CustomWord {
    pub word: String,
    pub category: String,
    pub severity: u32,
    pub strict: bool,
}

/// A single profanity hit in the caller's input.
#[napi(object)]
pub struct ProfanityMatch {
    /// Internal wordlist index; stable within one `Profanite` instance.
    pub word_id: u32,
    /// Byte offset in the original input where the match starts.
    pub start: u32,
    /// Byte offset (exclusive) where the match ends.
    pub end: u32,
    /// Byte span in the normalized text.
    pub normalized_start: u32,
    pub normalized_end: u32,
    pub category: String,
    pub severity: u32,
}

/// Main entry point. Construct once, reuse for many inputs.
#[napi]
pub struct Profanite {
    inner: CoreProfanite,
}

#[napi]
impl Profanite {
    /// `new Profanite(options?)`.
    #[napi(constructor)]
    pub fn new(options: Option<ProfaniteOptions>) -> Result<Self, NapiError> {
        let opts = options.unwrap_or(ProfaniteOptions {
            languages: None,
            normalization: None,
            match_mode: None,
            censor_style: None,
            mask_char: None,
            add_words: None,
            remove_words: None,
            allowlist: None,
            without_bundled: None,
        });

        let mut builder = CoreProfanite::builder();

        if opts.without_bundled.unwrap_or(false) {
            builder = builder.without_bundled();
        } else if let Some(langs) = opts.languages {
            for l in langs {
                builder = builder.language(parse_lang(&l)?);
            }
        }

        if let Some(n) = opts.normalization {
            builder = builder.normalization(parse_normalization(&n)?);
        }
        if let Some(m) = opts.match_mode {
            builder = builder.match_mode(parse_match_mode(&m)?);
        }
        if let Some(s) = opts.censor_style {
            builder = builder.censor_style(parse_censor_style(&s)?);
        }
        if let Some(c) = opts.mask_char {
            let mut chars = c.chars();
            let first = chars.next().ok_or_else(|| {
                NapiError::from_reason("maskChar must be a non-empty string".to_string())
            })?;
            if chars.next().is_some() {
                return Err(NapiError::from_reason(
                    "maskChar must be a single character".to_string(),
                ));
            }
            builder = builder.mask_char(first);
        }
        if let Some(additions) = opts.add_words {
            let mut rows: Vec<(String, CoreCategory, u8, bool)> =
                Vec::with_capacity(additions.len());
            for w in additions {
                let cat = parse_category(&w.category)?;
                let sev = u8::try_from(w.severity).map_err(|_| {
                    NapiError::from_reason("severity must fit in a byte".to_string())
                })?;
                rows.push((w.word, cat, sev, w.strict));
            }
            builder = builder.add_words(rows);
        }
        if let Some(removals) = opts.remove_words {
            builder = builder.remove_words(removals);
        }
        if let Some(allow) = opts.allowlist {
            builder = builder.allowlist(allow);
        }

        let inner = builder.build().map_err(map_err)?;
        Ok(Self { inner })
    }

    /// Returns true when `text` contains any profanity.
    #[napi]
    pub fn contains_profanity(&self, text: String) -> bool {
        self.inner.contains_profanity(&text)
    }

    /// Returns `text` with every profanity masked per the configured style.
    #[napi]
    pub fn censor(&self, text: String) -> String {
        self.inner.censor(&text)
    }

    /// Returns one entry per profanity hit.
    #[napi]
    pub fn find(&self, text: String) -> Vec<ProfanityMatch> {
        self.inner
            .find(&text)
            .into_iter()
            .map(|m| ProfanityMatch {
                word_id: m.word_id as u32,
                start: m.original_span.0 as u32,
                end: m.original_span.1 as u32,
                normalized_start: m.normalized_span.0 as u32,
                normalized_end: m.normalized_span.1 as u32,
                category: category_name(m.category).to_string(),
                severity: m.severity as u32,
            })
            .collect()
    }
}
