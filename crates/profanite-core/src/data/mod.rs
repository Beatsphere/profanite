use crate::lang::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Mild,
    Strong,
    Sexual,
    Slur,
    Slang,
}

#[derive(Debug, Clone)]
pub struct WordEntry {
    pub word: String,
    pub category: Category,
    pub severity: u8,
    /// If true, match this word anywhere (substring). Otherwise require word boundaries.
    pub strict: bool,
}

impl WordEntry {
    pub fn new(word: impl Into<String>, category: Category, severity: u8, strict: bool) -> Self {
        Self {
            word: word.into(),
            category,
            severity,
            strict,
        }
    }
}

#[cfg(feature = "lang-de")]
mod de;
#[cfg(feature = "lang-en")]
mod en;
#[cfg(feature = "lang-es")]
mod es;
#[cfg(feature = "lang-fr")]
mod fr;
#[cfg(feature = "lang-hi")]
mod hi;

pub(crate) fn bundled_for(lang: Lang) -> Vec<WordEntry> {
    match lang {
        #[cfg(feature = "lang-en")]
        Lang::En => en::entries(),
        #[cfg(not(feature = "lang-en"))]
        Lang::En => Vec::new(),

        #[cfg(feature = "lang-hi")]
        Lang::Hi => hi::entries(),
        #[cfg(not(feature = "lang-hi"))]
        Lang::Hi => Vec::new(),

        #[cfg(feature = "lang-es")]
        Lang::Es => es::entries(),
        #[cfg(not(feature = "lang-es"))]
        Lang::Es => Vec::new(),

        #[cfg(feature = "lang-fr")]
        Lang::Fr => fr::entries(),
        #[cfg(not(feature = "lang-fr"))]
        Lang::Fr => Vec::new(),

        #[cfg(feature = "lang-de")]
        Lang::De => de::entries(),
        #[cfg(not(feature = "lang-de"))]
        Lang::De => Vec::new(),
    }
}

/// Compact static representation used by per-language modules and the
/// build-generated LDNOOBW lists.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct StaticEntry {
    pub word: &'static str,
    pub category: Category,
    pub severity: u8,
    pub strict: bool,
}

#[allow(dead_code)]
impl StaticEntry {
    pub(crate) fn to_entry(self) -> WordEntry {
        WordEntry {
            word: self.word.to_string(),
            category: self.category,
            severity: self.severity,
            strict: self.strict,
        }
    }
}

/// Merge a generated list with curated overrides. Overrides take precedence
/// on word-match (case-insensitive) and extend the list with any new terms.
#[allow(dead_code)]
pub(crate) fn merge(generated: &[StaticEntry], overrides: &[StaticEntry]) -> Vec<WordEntry> {
    use std::collections::HashMap;
    let mut by_key: HashMap<String, WordEntry> = HashMap::new();
    for e in generated {
        by_key.insert(e.word.to_ascii_lowercase(), e.to_entry());
    }
    for e in overrides {
        by_key.insert(e.word.to_ascii_lowercase(), e.to_entry());
    }
    by_key.into_values().collect()
}
