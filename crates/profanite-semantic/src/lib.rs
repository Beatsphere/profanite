//! Home for real semantic-scoring implementations. The trait itself
//! lives in `profanite-core` so the builder API doesn't pull this crate
//! in — integrators who don't want ML stay on the zero-dep core.
//!
//! With the default features, this crate is just a re-export of the core
//! traits. Enable the `onnx` feature to pull in [`OnnxToxicScorer`], a
//! Xenova/toxic-bert (int8 ONNX) backend.

pub use profanite_core::{
    AlwaysProfane, MatchContext, SemanticDetector, SemanticScorer, SEMANTIC_WORD_ID,
};

#[cfg(feature = "onnx")]
mod onnx;

#[cfg(feature = "onnx")]
pub use onnx::{OnnxToxicScorer, OnnxToxicScorerError};
