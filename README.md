# profanite

**Kryptonite for profanities.** A lightweight, obfuscation-resistant profanity filter designed to drop into any language or framework.

> **Do not edit `README.md` directly.** It is regenerated from
> `README.template.md` + the canonical examples. Run
> `python3 scripts/sync-readme.py` after changing the template or
> examples. CI enforces this via `--check`.

---

## Status

- **Version:** `0.1.7`
- **Bundled languages:** English (`en`), Spanish (`es`), Hindi (romanized) (`hi`), French (`fr`), German (`de`)
- **Targets:** Rust (native) · Node.js (napi-rs binding) · Python (planned)
- **MSRV:** Rust `1.77`

---

## What you get

- `contains_profanity(text) → bool` / `censor(text) → string` / `find(text) → spans`
- Unicode normalization pipeline: bidi-strip, NFKC, casefold, homoglyph fold, conservative leet substitution, repeated-char collapse, optional aggressive separator stripping
- Tiered wordlist: short ambiguous stems (e.g. `ass`, `hell`) require word boundaries; unambiguous compounds (e.g. `motherfucker`, `bullshit`) match anywhere so bypasses like `Hemoglomotherfuckerbin` still fire
- Allowlist escape hatch for the Scunthorpe problem
- Bundled dictionaries from the CC0 [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) list, with curated English overrides layered on top
- Continuous benchmark harness with release gates (see `BENCHMARK.md`)

---

## Install

### Rust

Add to `Cargo.toml`:

```toml
[dependencies]
profanite-core = "0.1.7"
```

Feature flags select which bundled language lists compile in. Default is `lang-en`. Turn on others explicitly, or enable `all-langs`:

```toml
profanite-core = { version = "0.1.7", features = ["all-langs"] }
```

### Node.js

```bash
npm install @beatsphere/profanite
```

Platform-specific native binaries ship via `optionalDependencies`; npm picks the right one for your OS/arch automatically (Linux x64/arm64 gnu + musl, macOS x64/arm64, Windows x64).

### Python

```bash
pip install profanite
```

Prebuilt wheels for Linux (manylinux + musllinux, x86_64 + aarch64), macOS (x86_64 + arm64), and Windows x64. Python 3.8+ via the stable `abi3` ABI.

---

## Usage — Rust

<!-- BEGIN: examples/quickstart.rs -->
```rust
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
```
<!-- END: examples/quickstart.rs -->

Run it:

```bash
cargo run -p profanite-core --example quickstart
```

## Usage — Node.js

<!-- BEGIN: examples/quickstart.js -->
```js
/**
 * Quickstart example — this file is the canonical Node usage snippet.
 *
 * The README pulls its JS code block directly from here via
 * `scripts/sync-readme.py`. If you change this example, the README
 * regenerates automatically; if this example breaks, CI fails.
 */

const { Profanite } = require('@beatsphere/profanite');

// Build a filter once, reuse for many inputs.
const filter = new Profanite({
  languages: ['en'],
  censorStyle: 'lengthPreserving',
});

// Detect.
console.assert(filter.containsProfanity('what the fuck') === true);
console.assert(filter.containsProfanity('have a nice day') === false);

// Censor. Default style masks each character with '*'.
console.assert(filter.censor('what the fuck') === 'what the ****');

// Locate. Each match carries spans + category + severity.
const hits = filter.find('oh fuck that');
console.assert(hits.length === 1);
console.assert(hits[0].start === 3 && hits[0].end === 7);

// Obfuscation-resistant matching covers leet, homoglyphs, repeats,
// zero-width chars, fullwidth, and bidi overrides.
console.assert(filter.containsProfanity('what the fuсk')); // Cyrillic 'с'
console.assert(filter.containsProfanity('fuuuuuuck'));
console.assert(filter.containsProfanity('ＦＵＣＫ'));

console.log('quickstart ok');
```
<!-- END: examples/quickstart.js -->

Types ship in `index.d.ts` and cover every option, category, and return field.

## Usage — Python

<!-- BEGIN: examples/quickstart.py -->
```python
"""Quickstart example — canonical Python usage snippet.

The README pulls this file's content verbatim via
`scripts/sync-readme.py`. If you change this example, the README
regenerates automatically; if this example breaks, CI fails.
"""

from profanite import Profanite

# Build once, reuse for many inputs.
p = Profanite({
    "languages": ["en"],
    "censor_style": "length_preserving",
})

# Detect.
assert p.contains_profanity("what the fuck") is True
assert p.contains_profanity("have a nice day") is False

# Censor. Default style masks each character with '*'.
assert p.censor("what the fuck") == "what the ****"

# Locate. Each match carries spans + category + severity.
hits = p.find("oh fuck that")
assert len(hits) == 1
assert hits[0].start == 3 and hits[0].end == 7

# Obfuscation-resistant matching covers leet, homoglyphs, repeats,
# zero-width chars, fullwidth, and bidi overrides.
assert p.contains_profanity("what the fuсk")  # Cyrillic 'с'
assert p.contains_profanity("fuuuuuuck")
assert p.contains_profanity("ＦＵＣＫ")

print("quickstart ok")
```
<!-- END: examples/quickstart.py -->

---

## Configuration reference

| Option (Rust builder / JS option)        | Values                                                            | Default             |
| ---------------------------------------- | ----------------------------------------------------------------- | ------------------- |
| `language()` / `languages`                | `En`, `Es`, `Hi`, `Fr`, `De`                                      | `[En]`              |
| `normalization()` / `normalization`       | `None`, `Basic`, `Aggressive`                                     | `Basic`             |
| `match_mode()` / `matchMode`              | `WordBoundary`, `Substring`                                       | `WordBoundary`      |
| `censor_style()` / `censorStyle`          | `LengthPreserving`, `FirstLast`, `FullMask`, `Grawlix`            | `LengthPreserving`  |
| `mask_char()` / `maskChar`                | single char                                                        | `*`                 |
| `add_words()` / `addWords`                | extra entries with category + severity + strict                   | —                   |
| `remove_words()` / `removeWords`          | drop from bundled list (case-insensitive)                         | —                   |
| `allowlist()` / `allowlist`               | substrings where matches are suppressed                           | —                   |
| `without_bundled()` / `withoutBundled`    | start empty; caller supplies the whole list                       | `false`             |

Severity is a `1..=3` band (1 = mild, 3 = most severe). `strict: true` tells the matcher to ignore word boundaries for that entry — the right choice for long unambiguous compounds.

---

## What the benchmark says

This snapshot is generated by `cargo run -p profanite-bench -- snapshot`; the README resync then splices it in. Reproduce with `cargo run --release -p profanite-bench -- fast` (or `full` to include Jigsaw).

<!-- @generated by `cargo run -p profanite-bench -- snapshot`. Do not edit. -->

| Suite | Mode | n | recall | precision | fp_rate | f1 |
|---|---|---:|---:|---:|---:|---:|
| synthetic | basic | 137 | 0.986 | 1.000 | 0.000 | 0.993 |
| hatecheck | basic | 3146 | 0.118 | 1.000 | 0.000 | 0.211 |
| jigsaw | basic | 23353 | 0.770 | 0.987 | 0.046 | 0.865 |

See [`BENCHMARK.md`](BENCHMARK.md) for per-category tables, known ceilings (edit-distance matching, slur coverage), and the baseline-diff workflow. The [design philosophy](PHILOSOPHY.md) spells out what profanite is and is not.

---

## License

GPL-3.0-or-later. The bundled wordlists are derived from [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) (CC0) and the HateCheck benchmark is CC-BY-4.0; both are credited in the tree they sit in.
