//! Home for real semantic-scoring implementations (ONNX Runtime, candle,
//! etc.). The trait itself lives in `profanite-core` so the builder API
//! doesn't pull this crate in — integrators who don't want ML stay on the
//! zero-dep core.
//!
//! v0.1 re-exports the core trait so downstream crates have a single
//! import path once we add real scorer types here.

pub use profanite_core::{AlwaysProfane, MatchContext, SemanticScorer};
