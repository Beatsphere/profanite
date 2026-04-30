//! Scaffolding for a semantic (embedding-based) profanity scorer.
//!
//! v1 ships the trait only. Real implementations (ONNX Runtime, candle, etc.)
//! live behind feature flags in downstream examples.

use profanite_core::Match;

/// Context passed to a `SemanticScorer` so it can reason about a candidate match
/// in surrounding text.
#[derive(Debug, Clone)]
pub struct MatchContext<'a> {
    pub text: &'a str,
    pub match_info: &'a Match,
}

/// A pluggable scoring hook. Implementors return a confidence in `[0.0, 1.0]`
/// that a match is genuinely profane given its context.
pub trait SemanticScorer: Send + Sync {
    fn score(&self, ctx: &MatchContext<'_>) -> f32;
}
