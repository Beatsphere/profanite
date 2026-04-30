//! Pure-Rust ML oracle scaffolding using `candle`.
//!
//! This file is a deliberate STUB. It shows the intended shape of a
//! `candle`-backed `SemanticScorer` and the wiring needed to cross-check
//! profanite's keyword matcher against a trained classifier (e.g.
//! `unitary/toxic-bert`), but doesn't actually load a model.
//!
//! Why a stub instead of a working implementation: loading BERT-style
//! weights via `candle-transformers` + manually porting the tokenizer is
//! a ~1-2 day effort, and the value at v0.1 is marginal — the existing
//! synthetic + HateCheck + Jigsaw benchmark already catches regressions
//! effectively without it. This file exists so the path is clear when
//! someone picks the work up.
//!
//! Concrete v0.2 implementation sketch:
//!
//! ```text
//! 1. Add to Cargo.toml:
//!    candle-core, candle-nn, candle-transformers = "0.7"
//!    hf-hub = { version = "0.3", features = ["tokio"] }
//!    tokenizers = "0.20"
//!
//! 2. Download weights + tokenizer from HuggingFace on first run:
//!    let api = hf_hub::api::sync::Api::new()?;
//!    let repo = api.model("unitary/toxic-bert".into());
//!    let config = repo.get("config.json")?;
//!    let tokenizer_path = repo.get("tokenizer.json")?;
//!    let weights = repo.get("model.safetensors")?;
//!
//! 3. Load the BERT model:
//!    let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)?;
//!    let vb = VarBuilder::from_pth(&weights, DType::F32, &Device::Cpu)?;
//!    let model = candle_transformers::models::bert::BertModel::load(vb, &config)?;
//!
//! 4. score() impl: tokenize ctx.text, run inference, return
//!    sigmoid(toxic_logit). Cache the model once in the scorer struct.
//!
//! 5. Add a `compare` subcommand to profanite-bench that loads both
//!    profanite and this scorer, runs the Jigsaw corpus through each,
//!    and emits an agreement/disagreement table.
//! ```
//!
//! Until then: this example compiles (it's a no-op) but produces nothing
//! useful. The trait lives in profanite-core; this crate just re-exports.

use profanite_semantic::{MatchContext, SemanticScorer};

pub struct ToxicBertOracle;

impl SemanticScorer for ToxicBertOracle {
    fn score(&self, _ctx: &MatchContext<'_>) -> f32 {
        // Stub: always says "yep, profane". A real implementation would
        // tokenize ctx.text, run BERT inference, return the toxic
        // classifier-head logit through sigmoid.
        1.0
    }
}

fn main() {
    eprintln!("ToxicBertOracle is a v0.2 stub. See the module docs for the");
    eprintln!("implementation plan. For now, profanite-bench's existing fast");
    eprintln!("and full suites provide sufficient signal.");
}
