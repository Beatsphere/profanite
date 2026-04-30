//! Spanish wordlist (LDNOOBW, CC0).
//!
//! Curated overrides are empty at v0.1 — all entries carry the default
//! severity=2, category=Strong, and auto-tier strict flag. Platform
//! integrators wanting finer tiers should supply overrides via `add_words`.

use super::StaticEntry;

mod generated {
    #![allow(clippy::needless_raw_string_hashes, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/ldnoobw_es.rs"));
}

pub(crate) static OVERRIDES: &[StaticEntry] = &[];

pub(crate) fn entries() -> Vec<super::WordEntry> {
    super::merge(generated::WORDS, OVERRIDES)
}
