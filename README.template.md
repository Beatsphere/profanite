# profanite

**Kryptonite for profanities.** A lightweight, obfuscation-resistant profanity filter designed to drop into any language or framework.

> **Do not edit `README.md` directly.** It is regenerated from
> `README.template.md` + the canonical examples. Run
> `python3 scripts/sync-readme.py` after changing the template or
> examples. CI enforces this via `--check`.

---

## Status

- **Version:** `{{VERSION}}`
- **Bundled languages:** {{LANGUAGES}}
- **Targets:** Rust (native) · Node.js (napi-rs binding) · Python (planned)
- **MSRV:** Rust `{{MSRV}}`

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
profanite-core = "{{VERSION}}"
```

Feature flags select which bundled language lists compile in. Default is `lang-en`. Turn on others explicitly, or enable `all-langs`:

```toml
profanite-core = { version = "{{VERSION}}", features = ["all-langs"] }
```

### Node.js

> Publishing to npm is planned for v0.1 release (task M7). For now, build locally from source:

```bash
git clone https://github.com/Beatsphere/profanite
cd profanite
cargo build --release -p profanite-node
# The .node binary lands under target/release/; see crates/profanite-node/index.js
# for the resolver that loads it. A CI-driven prebuilt-binary flow is task M7.
```

### Python

Coming in v0.1 (task M5, PyO3 + maturin).

---

## Usage — Rust

<!-- BEGIN: examples/quickstart.rs -->
```rust
{{RUST_EXAMPLE}}
```
<!-- END: examples/quickstart.rs -->

Run it:

```bash
cargo run -p profanite-core --example quickstart
```

## Usage — Node.js

<!-- BEGIN: examples/quickstart.js -->
```js
{{NODE_EXAMPLE}}
```
<!-- END: examples/quickstart.js -->

Types ship in `index.d.ts` and cover every option, category, and return field.

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

This snapshot is captured by the benchmark harness in `bench/`. Reproduce with `cargo run --release -p profanite-bench -- fast`.

| Suite                       | Cases | Metric            | Gate         | Observed  |
| --------------------------- | ----: | ----------------- | -----------: | --------: |
| synthetic bypass            | 137   | recall            | ≥ 0.85       | **0.986** |
| HateCheck ES/FR/DE profane  | 413   | recall            | ≥ 0.35       | **0.387** |
| HateCheck ES/FR/DE leet     | 491   | recall            | ≥ 0.10       | **0.112** |

See `BENCHMARK.md` for per-category tables, known ceilings (edit-distance matching, slur coverage), and the baseline-diff workflow.

---

## License

GPL-3.0-or-later. The bundled wordlists are derived from [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) (CC0) and the HateCheck benchmark is CC-BY-4.0; both are credited in the tree they sit in.
