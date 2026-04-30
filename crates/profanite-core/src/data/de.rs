//! German wordlist (LDNOOBW, CC0).

use super::StaticEntry;

mod generated {
    #![allow(clippy::needless_raw_string_hashes, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/ldnoobw_de.rs"));
}

pub(crate) static OVERRIDES: &[StaticEntry] = &[];

pub(crate) fn entries() -> Vec<super::WordEntry> {
    super::merge(generated::WORDS, OVERRIDES)
}
