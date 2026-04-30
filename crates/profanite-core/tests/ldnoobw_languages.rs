//! Integration tests covering the LDNOOBW-imported multi-language bundles.
//!
//! These guard against:
//! - the generator silently emitting an empty list (wrong path, missing file)
//! - the auto-tiering classifying unambiguous compounds as word-bounded
//! - regressions in the per-language Scunthorpe tests
//!
//! The probe words are deliberately chosen from LDNOOBW's own lists so the
//! assertions match upstream reality. Benign probes are short, widely-used
//! greetings/common words that should never be on a profanity list.

use profanite_core::{Lang, Profanite};

fn builder_for(lang: Lang) -> Profanite {
    Profanite::builder().language(lang).build().unwrap()
}

#[test]
fn english_bundle_covers_ldnoobw_core() {
    let p = builder_for(Lang::En);
    // Known LDNOOBW entries
    assert!(p.contains_profanity("fuck"));
    assert!(p.contains_profanity("motherfucker"));
    // Multi-word phrases are strict by auto-tiering
    assert!(p.contains_profanity("alabama hot pocket"));
    // Benign stays clean (validates the tiering held)
    assert!(!p.contains_profanity("hello world"));
    assert!(!p.contains_profanity("passing the test"));
}

#[test]
fn english_concat_bypass_still_caught() {
    let p = builder_for(Lang::En);
    assert!(p.contains_profanity("Hemoglomotherfuckerbin"));
    assert!(p.contains_profanity("Superbullshitasaurus"));
}

#[test]
fn spanish_bundle_handles_accented_and_plain() {
    let p = builder_for(Lang::Es);
    // Diacritic folding means both surface forms should match.
    assert!(p.contains_profanity("cabrón"));
    assert!(p.contains_profanity("cabron"));
    assert!(p.contains_profanity("pendejo"));
    assert!(!p.contains_profanity("hola amigo"));
}

#[test]
fn hindi_bundle_catches_romanized_forms() {
    let p = builder_for(Lang::Hi);
    assert!(p.contains_profanity("chutiya"));
    assert!(p.contains_profanity("madarchod"));
    assert!(!p.contains_profanity("namaste"));
    assert!(!p.contains_profanity("dhanyawad"));
}

#[test]
fn french_bundle_catches_core_terms() {
    let p = builder_for(Lang::Fr);
    assert!(p.contains_profanity("merde"));
    assert!(p.contains_profanity("connard"));
    assert!(!p.contains_profanity("bonjour monde"));
}

#[test]
fn german_bundle_catches_arschloch() {
    let p = builder_for(Lang::De);
    assert!(p.contains_profanity("arschloch"));
    assert!(p.contains_profanity("Arschloch")); // casefold
    assert!(!p.contains_profanity("guten tag"));
}

#[test]
fn multi_language_bundle_catches_cross_language_hits() {
    // Load all 5 at once. A single `p` then flags profanity across all of them.
    let p = Profanite::builder()
        .languages([Lang::En, Lang::Es, Lang::Hi, Lang::Fr, Lang::De])
        .build()
        .unwrap();

    assert!(p.contains_profanity("fuck"));
    assert!(p.contains_profanity("pendejo"));
    assert!(p.contains_profanity("chutiya"));
    assert!(p.contains_profanity("merde"));
    assert!(p.contains_profanity("arschloch"));

    // Benign multi-language probe
    assert!(!p.contains_profanity("hello namaste bonjour guten tag"));
}
