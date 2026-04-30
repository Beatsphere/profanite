//! Quickstart example — this file is the canonical Rust usage snippet.
//!
//! The README pulls its Rust code block directly from here via
//! `scripts/sync-readme.py`, so if you change this example the README
//! regenerates automatically. Conversely, if this example stops
//! compiling, CI fails and the README can't drift out of sync.

use profanite_core::{CensorStyle, Lang, Profanite};

fn main() {
    // Build a filter. One-time cost; reuse the instance for many inputs.
    let filter = Profanite::builder()
        .language(Lang::En)
        .censor_style(CensorStyle::LengthPreserving)
        .build()
        .expect("builds with defaults");

    // Detect.
    assert!(filter.contains_profanity("what the fuck"));
    assert!(!filter.contains_profanity("have a nice day"));

    // Censor. Default style masks each character with '*'.
    assert_eq!(filter.censor("what the fuck"), "what the ****");

    // Locate. Each match returns original + normalized spans plus metadata.
    let hits = filter.find("oh fuck that");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].original_span, (3, 7));

    // Obfuscation-resistant matching handles leet, homoglyphs, repeats,
    // zero-width chars, fullwidth, and bidi overrides.
    assert!(filter.contains_profanity("what the fuсk")); // Cyrillic 'с'
    assert!(filter.contains_profanity("fuuuuuuck"));
    assert!(filter.contains_profanity("ＦＵＣＫ"));

    println!("quickstart ok");
}
