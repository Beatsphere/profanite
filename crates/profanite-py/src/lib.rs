//! Python bindings for profanite via PyO3.
//!
//! JS-side parity: a single `Profanite` class constructed with an options
//! dict, exposing the three core methods. Enum-style options are
//! lower-snake-case strings (matching Python idiom) rather than the JS
//! camelCase — e.g. `normalization='basic'`, `censor_style='first_last'`.
//!
//! Built via `maturin develop` / `maturin build`. Import path is
//! `import profanite`.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use profanite_core::{
    Category as CoreCategory, CensorStyle as CoreStyle, Lang, MatchMode as CoreMatchMode,
    NormalizationLevel, Profanite as CoreProfanite,
};

fn lift_core_err(e: profanite_core::Error) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

fn parse_lang(s: &str) -> PyResult<Lang> {
    match s {
        "en" => Ok(Lang::En),
        "es" => Ok(Lang::Es),
        "hi" => Ok(Lang::Hi),
        "fr" => Ok(Lang::Fr),
        "de" => Ok(Lang::De),
        other => Err(PyValueError::new_err(format!(
            "unknown language code: {other} (expected one of: en, es, hi, fr, de)"
        ))),
    }
}

fn parse_normalization(s: &str) -> PyResult<NormalizationLevel> {
    match s.to_lowercase().as_str() {
        "none" => Ok(NormalizationLevel::None),
        "basic" => Ok(NormalizationLevel::Basic),
        "aggressive" => Ok(NormalizationLevel::Aggressive),
        other => Err(PyValueError::new_err(format!(
            "unknown normalization level: {other}"
        ))),
    }
}

fn parse_match_mode(s: &str) -> PyResult<CoreMatchMode> {
    match s.to_lowercase().replace('-', "_").as_str() {
        "word_boundary" | "wordboundary" => Ok(CoreMatchMode::WordBoundary),
        "substring" => Ok(CoreMatchMode::Substring),
        other => Err(PyValueError::new_err(format!(
            "unknown match mode: {other}"
        ))),
    }
}

fn parse_censor_style(s: &str) -> PyResult<CoreStyle> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "lengthpreserving" => Ok(CoreStyle::LengthPreserving),
        "firstlast" => Ok(CoreStyle::FirstLast),
        "fullmask" => Ok(CoreStyle::FullMask),
        "grawlix" => Ok(CoreStyle::Grawlix),
        other => Err(PyValueError::new_err(format!(
            "unknown censor style: {other}"
        ))),
    }
}

fn parse_category(s: &str) -> PyResult<CoreCategory> {
    match s.to_lowercase().as_str() {
        "mild" => Ok(CoreCategory::Mild),
        "strong" => Ok(CoreCategory::Strong),
        "sexual" => Ok(CoreCategory::Sexual),
        "slur" => Ok(CoreCategory::Slur),
        "slang" => Ok(CoreCategory::Slang),
        other => Err(PyValueError::new_err(format!("unknown category: {other}"))),
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

/// Result of `find()` — one entry per profanity hit.
#[pyclass(frozen)]
#[derive(Clone)]
struct Match {
    #[pyo3(get)]
    word_id: u32,
    #[pyo3(get)]
    start: u32,
    #[pyo3(get)]
    end: u32,
    #[pyo3(get)]
    normalized_start: u32,
    #[pyo3(get)]
    normalized_end: u32,
    #[pyo3(get)]
    category: String,
    #[pyo3(get)]
    severity: u32,
}

#[pymethods]
impl Match {
    fn __repr__(&self) -> String {
        format!(
            "Match(start={}, end={}, category='{}', severity={})",
            self.start, self.end, self.category, self.severity
        )
    }
}

/// Main filter. Construct once, reuse.
///
/// Options (keyword-only, all optional):
///   languages:        list[str]   e.g. ['en', 'es']
///   normalization:    str         'none' | 'basic' | 'aggressive'
///   match_mode:       str         'word_boundary' | 'substring'
///   censor_style:     str         'length_preserving' | 'first_last' | 'full_mask' | 'grawlix'
///   mask_char:        str         single character
///   add_words:        list[dict]  [{'word': ..., 'category': ..., 'severity': int, 'strict': bool}]
///   remove_words:     list[str]
///   allowlist:        list[str]
///   without_bundled:  bool
#[pyclass(frozen)]
struct Profanite {
    inner: CoreProfanite,
}

#[pymethods]
impl Profanite {
    #[new]
    #[pyo3(signature = (options = None))]
    fn new(options: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let mut builder = CoreProfanite::builder();

        if let Some(opts) = options {
            let without_bundled = opts
                .get_item("without_bundled")?
                .map(|v| v.extract::<bool>())
                .transpose()?
                .unwrap_or(false);

            if without_bundled {
                builder = builder.without_bundled();
            } else if let Some(langs) = opts.get_item("languages")? {
                let langs: Vec<String> = langs.extract()?;
                for l in langs {
                    builder = builder.language(parse_lang(&l)?);
                }
            }

            if let Some(n) = opts.get_item("normalization")? {
                let s: String = n.extract()?;
                builder = builder.normalization(parse_normalization(&s)?);
            }
            if let Some(m) = opts.get_item("match_mode")? {
                let s: String = m.extract()?;
                builder = builder.match_mode(parse_match_mode(&s)?);
            }
            if let Some(s_val) = opts.get_item("censor_style")? {
                let s: String = s_val.extract()?;
                builder = builder.censor_style(parse_censor_style(&s)?);
            }
            if let Some(mc) = opts.get_item("mask_char")? {
                let s: String = mc.extract()?;
                let mut chars = s.chars();
                let first = chars
                    .next()
                    .ok_or_else(|| PyValueError::new_err("mask_char must be a non-empty string"))?;
                if chars.next().is_some() {
                    return Err(PyValueError::new_err(
                        "mask_char must be a single character",
                    ));
                }
                builder = builder.mask_char(first);
            }
            if let Some(add) = opts.get_item("add_words")? {
                let entries: Vec<Bound<'_, PyDict>> = add.extract()?;
                let mut rows: Vec<(String, CoreCategory, u8, bool)> =
                    Vec::with_capacity(entries.len());
                for d in entries {
                    let word: String = d
                        .get_item("word")?
                        .ok_or_else(|| PyValueError::new_err("add_words entry missing 'word'"))?
                        .extract()?;
                    let cat_s: String = d
                        .get_item("category")?
                        .ok_or_else(|| PyValueError::new_err("add_words entry missing 'category'"))?
                        .extract()?;
                    let severity: u8 = d
                        .get_item("severity")?
                        .ok_or_else(|| PyValueError::new_err("add_words entry missing 'severity'"))?
                        .extract()?;
                    let strict: bool = d
                        .get_item("strict")?
                        .ok_or_else(|| PyValueError::new_err("add_words entry missing 'strict'"))?
                        .extract()?;
                    rows.push((word, parse_category(&cat_s)?, severity, strict));
                }
                builder = builder.add_words(rows);
            }
            if let Some(rm) = opts.get_item("remove_words")? {
                let words: Vec<String> = rm.extract()?;
                builder = builder.remove_words(words);
            }
            if let Some(al) = opts.get_item("allowlist")? {
                let words: Vec<String> = al.extract()?;
                builder = builder.allowlist(words);
            }
        }

        Ok(Self {
            inner: builder.build().map_err(lift_core_err)?,
        })
    }

    fn contains_profanity(&self, text: &str) -> bool {
        self.inner.contains_profanity(text)
    }

    fn censor(&self, text: &str) -> String {
        self.inner.censor(text)
    }

    fn find(&self, text: &str) -> Vec<Match> {
        self.inner
            .find(text)
            .into_iter()
            .map(|m| Match {
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

#[pymodule]
fn profanite(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Profanite>()?;
    m.add_class::<Match>()?;
    Ok(())
}
