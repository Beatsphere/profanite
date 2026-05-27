//! ONNX-Runtime-backed toxicity scorer.
//!
//! Wraps `Xenova/toxic-bert` — a pre-quantized int8 ONNX export of
//! `unitary/toxic-bert` (BERT-base, 110M params, English-only). On CPU
//! this runs ~10x faster than the equivalent f32 candle path, which is
//! the difference between a hatecheck pass that takes 2 hours and one
//! that takes 10 minutes.
//!
//! Suppression-only role wires through [`SemanticScorer`]; recall path
//! wires through [`SemanticDetector`]. Both call the same underlying
//! inference; one [`OnnxToxicScorer`] instance can fill both via
//! `Arc::clone`.
//!
//! Output head: toxic-bert is multi-label across 6 categories
//! (`toxic`, `severe_toxic`, `obscene`, `threat`, `insult`,
//! `identity_hate`). For profanity filtering we collapse to
//! `max(sigmoid(logit_toxic), sigmoid(logit_obscene))` — those are the
//! two heads that align with profanite's actual job.

use std::path::PathBuf;
use std::sync::Mutex;

use ndarray::Array2;
use ort::{
    inputs,
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use tokenizers::Tokenizer;

use profanite_core::{MatchContext, SemanticDetector, SemanticScorer};

/// Default model on the Hub. Pre-quantized int8 ONNX export.
const DEFAULT_MODEL_REPO: &str = "Xenova/toxic-bert";
/// File path within the repo for the int8-quantized graph.
const DEFAULT_MODEL_FILE: &str = "onnx/model_quantized.onnx";
/// BERT's hard limit on this checkpoint.
const MAX_SEQ_LEN: usize = 512;

/// Index of the `toxic` head in toxic-bert's output.
const TOXIC_IDX: usize = 0;
/// Index of the `obscene` head in toxic-bert's output.
const OBSCENE_IDX: usize = 2;

#[derive(Debug, thiserror::Error)]
pub enum OnnxToxicScorerError {
    #[error("hf-hub error: {0}")]
    HfHub(String),
    #[error("tokenizer error: {0}")]
    Tokenizer(String),
    #[error("ort error: {0}")]
    Ort(String),
    #[error("inference error: {0}")]
    Inference(String),
}

// Helper: any ort error → our error wrapper. ort 2.0-rc has multiple
// error types in different submodules; map through Display.
fn ort_err<E: std::fmt::Display>(e: E) -> OnnxToxicScorerError {
    OnnxToxicScorerError::Ort(e.to_string())
}

/// Loaded once, reused across many `score()` / `detect()` calls.
pub struct OnnxToxicScorer {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    /// Set on first inference based on `session.inputs` — some BERT
    /// exports take token_type_ids, some don't.
    needs_token_type_ids: bool,
}

impl OnnxToxicScorer {
    /// Download (if necessary) and load `Xenova/toxic-bert` int8 quantized.
    /// First call materializes the ONNX file (~30 MB) and `tokenizer.json`
    /// (~700 KB) into the hf-hub cache; subsequent calls are fast.
    pub fn from_pretrained() -> Result<Self, OnnxToxicScorerError> {
        Self::from_pretrained_with(DEFAULT_MODEL_REPO, DEFAULT_MODEL_FILE)
    }

    pub fn from_pretrained_with(
        repo: &str,
        model_file: &str,
    ) -> Result<Self, OnnxToxicScorerError> {
        let api = hf_hub::api::sync::Api::new()
            .map_err(|e| OnnxToxicScorerError::HfHub(e.to_string()))?;
        let model_api = api.model(repo.to_string());
        let model_path = model_api
            .get(model_file)
            .map_err(|e| OnnxToxicScorerError::HfHub(e.to_string()))?;
        let tokenizer_path = model_api
            .get("tokenizer.json")
            .map_err(|e| OnnxToxicScorerError::HfHub(e.to_string()))?;
        Self::from_paths(&model_path, &tokenizer_path)
    }

    pub fn from_paths(
        model_path: &PathBuf,
        tokenizer_path: &PathBuf,
    ) -> Result<Self, OnnxToxicScorerError> {
        let session = Session::builder()
            .map_err(ort_err)?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(ort_err)?
            .commit_from_file(model_path)
            .map_err(ort_err)?;

        let needs_token_type_ids = session
            .inputs()
            .iter()
            .any(|i| i.name() == "token_type_ids");

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| OnnxToxicScorerError::Tokenizer(e.to_string()))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            needs_token_type_ids,
        })
    }

    /// Run the encoder on a single string. Returns
    /// `max(sigmoid(toxic_logit), sigmoid(obscene_logit))`.
    pub fn score_text(&self, text: &str) -> Result<f32, OnnxToxicScorerError> {
        let encoded = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| OnnxToxicScorerError::Tokenizer(e.to_string()))?;
        let mut ids: Vec<i64> = encoded.get_ids().iter().map(|&x| x as i64).collect();
        let mut mask: Vec<i64> = encoded
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();
        if ids.len() > MAX_SEQ_LEN {
            ids.truncate(MAX_SEQ_LEN);
            mask.truncate(MAX_SEQ_LEN);
        }
        let seq_len = ids.len();

        let input_ids_arr = Array2::from_shape_vec((1, seq_len), ids)
            .map_err(|e| OnnxToxicScorerError::Inference(e.to_string()))?;
        let attention_mask_arr = Array2::from_shape_vec((1, seq_len), mask)
            .map_err(|e| OnnxToxicScorerError::Inference(e.to_string()))?;

        let input_ids = Tensor::from_array(input_ids_arr).map_err(ort_err)?;
        let attention_mask = Tensor::from_array(attention_mask_arr).map_err(ort_err)?;

        let mut session = self.session.lock().expect("scorer session mutex poisoned");
        let outputs = if self.needs_token_type_ids {
            let token_type_ids =
                Tensor::from_array(Array2::<i64>::zeros((1, seq_len))).map_err(ort_err)?;
            session
                .run(inputs![
                    "input_ids" => input_ids,
                    "attention_mask" => attention_mask,
                    "token_type_ids" => token_type_ids,
                ])
                .map_err(ort_err)?
        } else {
            session
                .run(inputs![
                    "input_ids" => input_ids,
                    "attention_mask" => attention_mask,
                ])
                .map_err(ort_err)?
        };

        // Single output named "logits" with shape [1, 6].
        let (_shape, logits) = outputs[0].try_extract_tensor::<f32>().map_err(ort_err)?;
        let toxic = sigmoid(logits[TOXIC_IDX]);
        let obscene = sigmoid(logits[OBSCENE_IDX]);
        Ok(toxic.max(obscene))
    }
}

impl SemanticScorer for OnnxToxicScorer {
    fn score(&self, ctx: &MatchContext<'_>) -> f32 {
        // Score the surrounding text, not just the matched token, so
        // sentences like "i love this fucking song" can pull below the
        // confidence threshold while "fuck off" stays high.
        match self.score_text(ctx.text) {
            Ok(p) => p,
            Err(_) => 1.0, // Fail-open: trust the keyword matcher on inference error.
        }
    }
}

impl SemanticDetector for OnnxToxicScorer {
    fn detect(&self, text: &str) -> f32 {
        // Recall-recovery role: if the keyword matcher missed everything,
        // ask the encoder. Fail-closed here so an inference error doesn't
        // synthesize a false positive on benign input.
        self.score_text(text).unwrap_or(0.0)
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
