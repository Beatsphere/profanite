//! Pluggable semantic scoring hook.
//!
//! A `SemanticScorer` runs after the keyword matcher. For every candidate
//! match, the scorer returns a confidence in `[0.0, 1.0]` that the hit is
//! genuinely profane given its surrounding context. Matches with a score
//! below the configured `min_confidence` threshold are discarded.
//!
//! v0.1 ships only the trait + a no-op default. Real implementations
//! (ONNX Runtime, candle, etc.) live in downstream crates.

use crate::matcher::Match;

/// Surrounding-text context for one candidate match.
#[derive(Debug, Clone, Copy)]
pub struct MatchContext<'a> {
    pub text: &'a str,
    pub match_info: &'a Match,
}

/// A pluggable scoring hook.
pub trait SemanticScorer: Send + Sync {
    /// Returns a confidence in `[0.0, 1.0]` that the match is genuinely
    /// profane. Implementations MAY clamp out-of-range values; the runtime
    /// treats anything >= the configured threshold as "keep".
    fn score(&self, ctx: &MatchContext<'_>) -> f32;
}

/// Scorer that always returns `1.0` — i.e. trusts the keyword matcher
/// completely. This is the default when no scorer is attached.
#[derive(Debug, Clone, Copy)]
pub struct AlwaysProfane;

impl SemanticScorer for AlwaysProfane {
    fn score(&self, _ctx: &MatchContext<'_>) -> f32 {
        1.0
    }
}
