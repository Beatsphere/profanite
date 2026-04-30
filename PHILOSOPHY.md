# Design philosophy and non-goals

profanite is a **keyword-based** profanity filter. Before using it, understand what that means.

## What profanite IS

- A fast, obfuscation-resistant matcher that flags text containing entries from a wordlist.
- A drop-in pre-moderation filter for chat rooms, comment sections, user-generated text forms, and similar.
- Language-pluggable: bundled dictionaries for English, Spanish, Hindi, French, German — with a first-class API for supplying your own.
- Designed to be **boringly correct**: if it flags something, it's because of a literal word match (possibly through normalization), not a guess.

## What profanite is NOT

1. **Not a hate-speech detector.** HateCheck's `identity_attack` or `derogatory_rhetoric` functionalities are out of scope — identity-based hate that doesn't use profane words will not be caught. Use a dedicated toxicity classifier for that.
2. **Not a legal or compliance tool.** This is a pragmatic filter for moderation, not a guarantee of regulatory fitness. If you need compliance attestation, talk to a lawyer.
3. **Not a bulletproof bypass defense.** Determined users will always find new bypasses. profanite catches ~98% of synthetic bypasses in our corpus and ~77% recall on real Twitter data — excellent for a keyword filter, but not a replacement for rate limits, user reports, and human review.
4. **Not a slur dictionary.** The bundled wordlists deliberately omit slurs. Slurs require platform-specific policy calls about reclaimed terms, regional variance, and enforcement severity. Supply your own curated slur list via `add_words`.
5. **Not context-aware.** "I love this fucking place" and "fuck off" both flag identically. If you need sentiment-aware handling, layer a `SemanticScorer` over the keyword matcher (the hook is there).

## Expected false-positive and false-negative rates

Every release publishes these. The current numbers live in [BENCHMARK.md](BENCHMARK.md) and the README. If you see them drift in a direction you don't like, the benchmark suite is your vocabulary for discussing it.

## What "true" means here

A match is "true" when the input contains a wordlist entry under the configured normalization. Not when the input is "actually profane" — that's a subjective judgment we don't try to make. This matters when you're debugging: if `p.contains_profanity("my classmate")` returns true, the bug is almost certainly in your wordlist (someone added `ass` as a non-strict mild-tier entry, which is correct) or your configuration (you turned on `Substring` mode, which disables word-boundary protection).

## When to NOT use profanite

- You need to moderate identity-based hate speech or harassment that doesn't rely on profane words. Use a trained classifier.
- You need typo-tolerant matching (edit-distance, fuzzy matching). v0.1 doesn't do this; v0.2 might.
- You're on a platform where flagging must have perfect recall AND perfect precision. That combination is not achievable by any known system. Decide which matters more to you and tune accordingly.

## Changelog semantics

- **Patch versions** (0.1.x) will never reduce recall or increase FP rate below the release gates. Regressions in any gated metric require a minor version bump.
- **Minor versions** (0.x.0) may adjust gate thresholds upward (stricter), never downward.
- **Major versions** (x.0.0) may change wordlists, remove configuration options, or break the API.
