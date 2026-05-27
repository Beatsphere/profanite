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
- **Targets:** Rust (native) · Node.js (napi-rs binding) · Python (maturin binding)
- **MSRV:** Rust `{{MSRV}}`

---

## What you get

- `contains_profanity(text) → bool` / `censor(text) → string` / `find(text) → spans`
- Unicode normalization pipeline: bidi-strip, NFKC, casefold, homoglyph fold, conservative leet substitution, repeated-char collapse, optional aggressive separator stripping
- Tiered wordlist: short ambiguous stems (e.g. `ass`, `hell`) require word boundaries; unambiguous compounds (e.g. `motherfucker`, `bullshit`) match anywhere so bypasses like `Hemoglomotherfuckerbin` still fire
- Allowlist escape hatch for the Scunthorpe problem
- Bundled dictionaries from the CC0 [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) list, with curated English overrides layered on top
- **Optional semantic scoring** via a small BERT-based toxicity model ([Xenova/toxic-bert](https://huggingface.co/Xenova/toxic-bert), int8 ONNX, ~30 MB). Two pluggable hooks:
  - **Suppression** (`SemanticScorer`) — re-checks keyword hits against the model to kill false positives
  - **Recall recovery** (`SemanticDetector`) — catches profanity the keyword matcher missed (paraphrases, typos, novel slang)
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

## Usage — Python

<!-- BEGIN: examples/quickstart.py -->
```python
{{PYTHON_EXAMPLE}}
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

## Semantic scoring (optional)

The keyword matcher is fast and precise, but it can only catch text that matches a wordlist entry. For cases where profanity is paraphrased, misspelled beyond normalization, or uses novel slang, profanite ships an optional BERT-based toxicity model that runs alongside the keyword pipeline.

### How it works

```
Input text
    │
    ▼
┌─────────────────┐
│ Keyword matcher  │──▶ hits found? ──yes──▶ SemanticScorer (suppression)
│ (fast, precise)  │                              │
└─────────────────┘                         score ≥ threshold? → keep hit
    │                                       score < threshold? → drop hit
    no hits
    │
    ▼
┌─────────────────────┐
│ SemanticDetector     │──▶ score ≥ threshold? → emit synthetic match
│ (recall recovery)    │    score < threshold? → clean input, nothing flagged
└─────────────────────┘
```

### Quick start (Rust)

```toml
[dependencies]
profanite-core = "{{VERSION}}"
profanite-semantic = { version = "{{VERSION}}", features = ["onnx"] }
```

```rust
use std::sync::Arc;
use profanite_core::Profanite;
use profanite_semantic::OnnxToxicScorer;

// Load once (~30 MB download on first run, cached after).
let scorer = Arc::new(OnnxToxicScorer::from_pretrained().unwrap());

let filter = Profanite::builder()
    .language(profanite_core::Lang::En)
    // Suppression: only drop keyword hits the model is very sure are FPs.
    .scorer(scorer.clone())
    .min_confidence(0.05)
    // Recall recovery: catch things the keyword matcher missed.
    .detector(scorer)
    .detector_threshold(0.5)
    .build()
    .unwrap();

filter.contains_profanity("what the fuck");  // true  (keyword hit)
filter.contains_profanity("go drink bleach"); // true  (detector recovery)
filter.contains_profanity("have a nice day"); // false
```

The model is `Xenova/toxic-bert` — a pre-quantized int8 ONNX export of `unitary/toxic-bert` (BERT-base, English). It runs in ~5 ms per inference on CPU via ONNX Runtime. The `onnx` feature is fully optional; users who don't enable it get zero extra dependencies.

---

## What the benchmark says

This snapshot is generated by `cargo run -p profanite-bench -- snapshot`; the README resync then splices it in. Reproduce with `cargo run --release -p profanite-bench -- fast` (or `full` to include Jigsaw).

{{BENCH_SNAPSHOT}}

See [`BENCHMARK.md`](BENCHMARK.md) for per-category tables, known ceilings (edit-distance matching, slur coverage), and the baseline-diff workflow. The [design philosophy](PHILOSOPHY.md) spells out what profanite is and is not.

---

## License

GPL-3.0-or-later. The bundled wordlists are derived from [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) (CC0) and the HateCheck benchmark is CC-BY-4.0; both are credited in the tree they sit in.
