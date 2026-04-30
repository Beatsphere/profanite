//! Hindi wordlist (LDNOOBW, CC0).
//!
//! LDNOOBW's Hindi file is romanized (Latin script) — that's the form most
//! common in chat moderation. Devanagari coverage is a follow-up.
//!
//! No curated overrides at v0.1.

use super::StaticEntry;

mod generated {
    #![allow(clippy::needless_raw_string_hashes, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/ldnoobw_hi.rs"));
}

pub(crate) static OVERRIDES: &[StaticEntry] = &[];

pub(crate) fn entries() -> Vec<super::WordEntry> {
    super::merge(generated::WORDS, OVERRIDES)
}
