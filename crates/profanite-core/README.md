# profanite-core

The Rust core of [profanite](https://github.com/Beatsphere/profanite) — a lightweight, obfuscation-resistant profanity filter.

```toml
[dependencies]
profanite-core = "0.1"
```

```rust
use profanite_core::Profanite;

let p = Profanite::builder().build().unwrap();

p.contains_profanity("what the fuck");   // true
p.censor("what the fuck");               // "what the ****"
p.find("oh fuck that");                  // Vec<Match> with spans + category + severity
```

### Features

- **Obfuscation-resistant**: bidi-strip, NFKC, casefold, homoglyph fold, conservative leet substitution, repeated-char collapse.
- **Tiered wordlist**: short ambiguous stems (e.g. `ass`) require word boundaries; unambiguous compounds (e.g. `motherfucker`) match anywhere, catching concat bypasses like `Hemoglomotherfuckerbin`.
- **Allowlist escape hatch** for the Scunthorpe problem.
- **Bundled dictionaries** from [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) (CC0) for English, Spanish, Hindi (romanized), French, German. Enable via cargo features:

```toml
profanite-core = { version = "0.1", features = ["all-langs"] }
```

### What this is not

profanite is a *keyword* filter. It is not a hate-speech detector, not a slur dictionary (slurs are deliberately unbundled — supply your own), and not context-aware. See [PHILOSOPHY.md](https://github.com/Beatsphere/profanite/blob/main/PHILOSOPHY.md) in the main repo.

### Benchmarks

Run `cargo run -p profanite-bench -- fast` in the workspace. Current numbers live in [README.md](https://github.com/Beatsphere/profanite/blob/main/README.md#what-the-benchmark-says).

### Node.js and Python bindings

- **Node**: [`profanite`](https://www.npmjs.com/package/profanite) on npm.
- **Python**: [`profanite`](https://pypi.org/project/profanite/) on PyPI.

License: GPL-3.0-or-later.
