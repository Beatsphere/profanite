//! End-to-end example: load Xenova/toxic-bert (int8 ONNX) through ort
//! and score a few sentences.
//!
//! Gated behind `PROFANITE_DOWNLOAD_MODEL=1` to keep casual `cargo run`
//! from triggering a one-time ~30 MB model download. The downloaded
//! files land in `~/.cache/huggingface/hub`. Subsequent runs are fast.
//!
//! Run with:
//!   PROFANITE_DOWNLOAD_MODEL=1 cargo run --release \
//!       -p profanite-semantic --features onnx --example onnx_oracle

use profanite_semantic::OnnxToxicScorer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PROFANITE_DOWNLOAD_MODEL").ok().as_deref() != Some("1") {
        eprintln!(
            "Skipping the onnx_oracle example: this would download ~30 MB \
             from the Hugging Face Hub. Re-run with \
             PROFANITE_DOWNLOAD_MODEL=1 to actually pull the model."
        );
        return Ok(());
    }

    eprintln!("Loading Xenova/toxic-bert (int8 ONNX, ~30 MB on first run)…");
    let scorer = OnnxToxicScorer::from_pretrained()?;
    eprintln!("Model ready.");

    let cases = [
        "I love this song",
        "have a nice day",
        "you fucking idiot",
        "what the hell is going on",
        "shut the fuck up",
        // The encoder's reason for existing — keyword matcher can't catch this.
        "kys",
        "go drink bleach",
    ];

    for text in &cases {
        let p = scorer.score_text(text)?;
        println!("  toxic_or_obscene={:.3}  text={:?}", p, text);
    }

    Ok(())
}
