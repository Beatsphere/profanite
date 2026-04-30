# profanite evaluation harness

Two truths sit behind every decision in this library:

1. Profanity filters are inherently imperfect — false positives and false negatives are baked into the problem.
2. The only way to judge "did that change make things better or worse?" is to measure it against a labeled corpus.

This directory (`bench/`) is how we measure. Every release is gated on these numbers and every feature change should move them in the right direction.

## Quick start

```bash
# Fast suite (synthetic + HateCheck). ~1 sec. Runs in CI on every PR.
cargo run --release -p profanite-bench -- fast

# Sweep both Basic and Aggressive normalization modes side-by-side.
# Aggressive numbers are informational — gates only apply to Basic.
cargo run --release -p profanite-bench -- fast --mode-sweep

# Save machine-readable + Markdown reports and dump every FP/FN.
cargo run --release -p profanite-bench -- fast \
    --json bench-report.json \
    --markdown bench-report.md \
    --dump-failures bench-failures.jsonl

# Compare against a saved baseline and exit non-zero on regression.
cargo run --release -p profanite-bench -- fast --baseline baseline.json

# List gates and their thresholds.
cargo run --release -p profanite-bench -- gates

# Run one specific suite.
cargo run --release -p profanite-bench -- suite synthetic

# Full suite (adds Jigsaw if you've fetched it).
cargo run --release -p profanite-bench -- full
```

## What the harness guarantees

The benchmark is designed so it's hard to silently mislead yourself.

- **Failure dumps** (`--dump-failures`) emit every FP and FN as JSONL — text, label, category, mode — so a gate failure is never just a number, it's the exact cases that moved.
- **Mode sweep** (`--mode-sweep`) runs each suite under both Basic and Aggressive normalization. Aggressive often *regresses* numbers (space-stripping corrupts multi-word input), so surfacing this prevents accidentally shipping it as "just better."
- **Per-language breakdown**. HateCheck aggregates ES/FR/DE — the report also shows each language on its own so a regression in one language isn't hidden by the others.
- **Reproducibility metadata**. Every JSON report records the corpus SHA-256, git rev, timestamp, bench version, and a snapshot of every gate threshold in effect at runtime. Numbers are always tied to exact data.
- **Baseline comparison** (`--baseline`) computes per-suite-mode deltas (Δrecall, Δfp_rate, Δf1). Regressions beyond `--baseline-noise` (default 0.005) exit non-zero. Threshold changes between baseline and current runs are also surfaced.
- **Markdown report** (`--markdown`) — GitHub-flavored, suitable for PR comments. CI posts this on every PR.
- **Harness unit tests** (`cargo test -p profanite-bench`) — 9 tests covering metric computation edge cases, gate evaluation, JSON round-trip, and per-category/per-language aggregation.
- **Gate calibration**. Gates only apply to Basic mode (the default users run). Aggressive-mode scores are reported but never fail the build.

## Release gates

The CLI exits non-zero if any gate fails.

| Gate | Suite / Category | Threshold | Purpose |
|---|---|---|---|
| `bypass_catch_rate` | synthetic (overall) | recall ≥ 0.85 | Hand-crafted bypass corpus: leet, homoglyph, repeat, concat, bidi, zero-width, etc. |
| `hatecheck_profanity_recall` | hatecheck / `profanity_h` | recall ≥ 0.35 | Multilingual recall on HateCheck's profanity-bearing sentences. |
| `hatecheck_leet_recall` | hatecheck / `spell_leet_h` | recall ≥ 0.10 | Leet-bypass robustness across ES/FR/DE. |
| `jigsaw_recall` | jigsaw (overall) | recall ≥ 0.80 | Jigsaw `obscene` slice — realistic English chat. |
| `jigsaw_fp_rate` | jigsaw (overall) | fp_rate ≤ 0.03 | Jigsaw `non-toxic` slice — benign English chat. |

Jigsaw is not fetched by default because it's ~100 MB. The fast suite (synthetic + hatecheck) is what CI runs on every PR.

## Current numbers (v0.1)

Run `cargo run --release -p profanite-bench -- fast --mode-sweep` to reproduce.

```
synthetic [basic] (137 cases)
  precision=1.000  recall=0.986  f1=0.993  fp_rate=0.000

synthetic [aggressive] (137 cases)
  precision=1.000  recall=0.736  f1=0.848   ← Aggressive is WORSE for typical input
                                              (space-stripping crushes multi-word sentences)

hatecheck [basic] (3146 cases)
  profanity_h        recall=0.387     ← gated
  slur_h             recall=0.237     [not gated — slurs aren't bundled]
  spell_leet_h       recall=0.112     ← gated
  spell_char_del_h   recall=0.055     [not gated — needs edit distance]
  spell_char_swap_h  recall=0.055     [not gated — needs edit distance]
  spell_space_add_h  recall=0.016     [not gated — Aggressive mode only]
  spell_space_del_h  recall=0.002     [not gated — Aggressive mode only]

  per-language:
    de  recall=0.104  n=1005
    es  recall=0.142  n=1067
    fr  recall=0.107  n=1074
```

**Known ceilings and why they're not hidden by the gates:**
- `char_del` / `char_swap` need edit-distance matching — a v0.2 feature.
- `space_add` / `space_del` only register under `NormalizationLevel::Aggressive`, which is off by default to protect precision.
- Slurs aren't bundled — integrators supply curated slur lists appropriate to their platform.

## Data sources

- **`bench/data/bypass_corpus.jsonl`** — hand-crafted. Apache-2.0. Committed.
- **`bench/data/hatecheck.jsonl`** — generated from Paul et al.'s HateCheck-{Spanish,French,German} (CC-BY-4.0). Filtered to profanity-relevant functionalities. Re-fetch with `bash bench/scripts/convert_hatecheck.sh`.
- **`bench/data/jigsaw.jsonl`** — generated from `google/jigsaw_toxicity_pred` (CC0). Not committed; fetch with the Jigsaw script (task #12).

## Corpus schema

Every corpus is JSONL with:

```json
{"text": "...", "label": "profane"|"benign", "category": "leet", "lang": "en"}
```

Lines starting with `#` and blank lines are skipped.

## Report schema

`--json` writes `FullReport`:

- `metadata`: timestamp, git rev, bench version
- `suites[]`: one entry per (suite, mode) with `corpus_sha256`, metrics, gates
- `gates_def[]`: snapshot of every gate's name/threshold/direction at the time of the run
- `green`: overall boolean

Baseline comparison reads this schema back to compute deltas.

## Adding a new dataset

1. Write a fetcher/converter in `bench/scripts/`.
2. Output JSONL into `bench/data/<name>.jsonl`.
3. Add a `SuiteDef` in `bench/src/suites.rs`.
4. Optionally define a `Gate` in `bench/src/gates.rs` and attach it to the suite.
5. Document the source, license, and any filtering applied in this file.
