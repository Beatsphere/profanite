//! English wordlist.
//!
//! The bulk comes from LDNOOBW (CC0) via `build.rs`, which generates a
//! `WORDS` array of `StaticEntry` values with:
//!   - category = Strong
//!   - severity = 2
//!   - strict   = true if phrase or >= 12 chars, else false
//!
//! Curated overrides layered on top refine category, severity, and strict
//! for high-signal terms. Merging rules:
//!   - If a curated entry's word matches a generated entry (case-insensitive),
//!     the curated version wins.
//!   - Additional curated entries extend the list.
//!
//! See `super::merge_static_entries` for how this stacks at load time.

use super::{Category, StaticEntry};

mod generated {
    #![allow(clippy::needless_raw_string_hashes, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/ldnoobw_en.rs"));
}

/// Hand-curated overrides that refine severity / category / strict for terms
/// already in the generated list, or add terms LDNOOBW missed.
pub(crate) static OVERRIDES: &[StaticEntry] = &[
    // Top-tier unambiguous compounds — always strict.
    StaticEntry {
        word: "motherfucker",
        category: Category::Strong,
        severity: 3,
        strict: true,
    },
    StaticEntry {
        word: "cocksucker",
        category: Category::Sexual,
        severity: 3,
        strict: true,
    },
    StaticEntry {
        word: "asshole",
        category: Category::Strong,
        severity: 3,
        strict: true,
    },
    StaticEntry {
        word: "bullshit",
        category: Category::Strong,
        severity: 3,
        strict: true,
    },
    StaticEntry {
        word: "dickhead",
        category: Category::Strong,
        severity: 3,
        strict: true,
    },
    // Tier-3 strong, word-bounded.
    StaticEntry {
        word: "fuck",
        category: Category::Strong,
        severity: 3,
        strict: false,
    },
    StaticEntry {
        word: "fucker",
        category: Category::Strong,
        severity: 3,
        strict: false,
    },
    StaticEntry {
        word: "fucking",
        category: Category::Strong,
        severity: 3,
        strict: false,
    },
    StaticEntry {
        word: "cunt",
        category: Category::Sexual,
        severity: 3,
        strict: false,
    },
    // Tier-1 mild — downgrade from LDNOOBW's default severity=2 where
    // appropriate, and keep them non-strict to avoid Scunthorpe.
    StaticEntry {
        word: "ass",
        category: Category::Mild,
        severity: 1,
        strict: false,
    },
    StaticEntry {
        word: "damn",
        category: Category::Mild,
        severity: 1,
        strict: false,
    },
    StaticEntry {
        word: "hell",
        category: Category::Mild,
        severity: 1,
        strict: false,
    },
    StaticEntry {
        word: "crap",
        category: Category::Mild,
        severity: 1,
        strict: false,
    },
    StaticEntry {
        word: "piss",
        category: Category::Mild,
        severity: 1,
        strict: false,
    },
];

pub(crate) fn entries() -> Vec<super::WordEntry> {
    super::merge(generated::WORDS, OVERRIDES)
}
