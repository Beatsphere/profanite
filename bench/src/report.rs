//! Human-readable console output + machine-readable JSON / Markdown.

use crate::corpus::Case;
use crate::gates::{Gate, GateInfo, GateResult};
use crate::metrics::{EvalResult, Outcome};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// One evaluation pass of a suite under a specific normalization mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteReport {
    pub suite: String,
    pub mode: String, // "basic" | "aggressive"
    pub cases: usize,
    pub corpus_sha256: String,
    pub eval: EvalResult,
    pub gates: Vec<GateResult>,
    /// True when this run had the semantic scorer attached. Old JSON
    /// reports without this field default to false.
    #[serde(default)]
    pub semantic_attached: bool,
}

/// Top-level report bundling all suites we ran plus reproducibility metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FullReport {
    pub metadata: Metadata,
    pub suites: Vec<SuiteReport>,
    pub gates_def: Vec<GateInfo>,
    pub green: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub timestamp: String,
    pub git_rev: Option<String>,
    pub bench_version: String,
}

impl Metadata {
    pub fn current() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            git_rev: detect_git_rev(),
            bench_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

fn detect_git_rev() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// SHA-256 over raw file bytes. Kept for reproducibility traceability.
pub fn sha256_of(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

// Inline hex encoder (avoids pulling `hex` crate just for this).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        const HEX: &[u8] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.as_ref().len() * 2);
        for b in bytes.as_ref() {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0F) as usize] as char);
        }
        out
    }
}

pub fn print_suite(report: &SuiteReport) {
    let m = &report.eval.overall;
    let semantic_tag = if report.semantic_attached {
        " + semantic"
    } else {
        ""
    };
    println!(
        "\n── {} [{}{}] ({} cases) ──",
        report.suite, report.mode, semantic_tag, report.cases
    );
    println!(
        "  precision={:.3}  recall={:.3}  f1={:.3}  fp_rate={:.3}  accuracy={:.3}",
        m.precision, m.recall, m.f1, m.fp_rate, m.accuracy
    );
    println!(
        "  counts: tp={}  fn={}  fp={}  tn={}",
        m.tp, m.fn_, m.fp, m.tn
    );

    if !report.eval.per_category.is_empty() {
        println!("  per-category:");
        for (cat, metrics) in &report.eval.per_category {
            println!(
                "    {cat:<20} recall={:.3}  fp_rate={:.3}  (n={})",
                metrics.recall, metrics.fp_rate, metrics.total
            );
        }
    }
    if report.eval.per_language.len() > 1 {
        println!("  per-language:");
        for (lang, metrics) in &report.eval.per_language {
            println!(
                "    {lang:<8} recall={:.3}  precision={:.3}  fp_rate={:.3}  (n={})",
                metrics.recall, metrics.precision, metrics.fp_rate, metrics.total
            );
        }
    }
    for gr in &report.gates {
        let tag = if gr.passed { "PASS" } else { "FAIL" };
        let scope = gr.gate.category.as_deref().map_or_else(
            || gr.gate.suite.clone(),
            |c| format!("{}/{}", gr.gate.suite, c),
        );
        println!(
            "  [{tag}] {} ({scope}) :: observed {:.3} (threshold {:.3})",
            gr.gate.name, gr.value, gr.gate.threshold
        );
    }
}

/// Print a side-by-side delta block comparing keyword-only vs
/// keyword+semantic on the same suite/mode. Only the keyword-only run's
/// gate set is meaningful (gates are calibrated against that).
pub fn print_semantic_delta(baseline: &SuiteReport, semantic: &SuiteReport) {
    assert!(!baseline.semantic_attached, "baseline must be keyword-only");
    assert!(semantic.semantic_attached, "comparison must have scorer attached");

    let b = &baseline.eval.overall;
    let s = &semantic.eval.overall;

    println!(
        "\n  Δ semantic [{}/{}]: \
         Δrecall={:+.3}  Δprecision={:+.3}  Δfp_rate={:+.3}  Δf1={:+.3}",
        baseline.suite,
        baseline.mode,
        s.recall - b.recall,
        s.precision - b.precision,
        s.fp_rate - b.fp_rate,
        s.f1 - b.f1,
    );

    // Per-category: surface the cases where the encoder moves the needle.
    let mut moved_cats: Vec<(&String, f64)> = Vec::new();
    for (cat, sm) in &semantic.eval.per_category {
        if let Some(bm) = baseline.eval.per_category.get(cat) {
            let d = sm.recall - bm.recall;
            if d.abs() >= 0.01 {
                moved_cats.push((cat, d));
            }
        }
    }
    if !moved_cats.is_empty() {
        moved_cats.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        println!("    per-category Δrecall (|Δ| ≥ 0.01):");
        for (cat, d) in moved_cats {
            println!("      {cat:<20} {:+.3}", d);
        }
    }
}

pub fn print_summary(full: &FullReport) {
    println!("\n==================================================");
    if full.green {
        println!("ALL GATES GREEN");
    } else {
        let failed: Vec<String> = full
            .suites
            .iter()
            .flat_map(|s| {
                s.gates
                    .iter()
                    .filter(|g| !g.passed)
                    .map(|g| format!("{}[{}]", g.gate.name, s.mode))
            })
            .collect();
        println!("GATES FAILED: {}", failed.join(", "));
    }
    println!("==================================================");
}

pub fn write_json(path: &Path, report: &FullReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn write_markdown(path: &Path, report: &FullReport) -> Result<()> {
    std::fs::write(path, render_markdown(report))?;
    Ok(())
}

pub fn render_markdown(report: &FullReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# profanite benchmark report\n\nStatus: **{}**\n",
        if report.green { "PASS" } else { "FAIL" }
    );
    let _ = writeln!(
        out,
        "Git: `{}` · Generated: `{}`\n",
        report.metadata.git_rev.as_deref().unwrap_or("unknown"),
        report.metadata.timestamp
    );
    for s in &report.suites {
        let m = &s.eval.overall;
        let _ = writeln!(
            out,
            "## {} — mode `{}` ({} cases)\n\n\
             | precision | recall | f1 | fp_rate | accuracy |\n\
             |---:|---:|---:|---:|---:|\n\
             | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |\n",
            s.suite, s.mode, s.cases, m.precision, m.recall, m.f1, m.fp_rate, m.accuracy
        );
        if !s.eval.per_category.is_empty() {
            out.push_str(
                "\n**Per category**\n\n| category | n | recall | fp_rate |\n|---|---:|---:|---:|\n",
            );
            for (cat, m) in &s.eval.per_category {
                let _ = writeln!(
                    out,
                    "| `{cat}` | {} | {:.3} | {:.3} |",
                    m.total, m.recall, m.fp_rate
                );
            }
        }
        if s.eval.per_language.len() > 1 {
            out.push_str("\n**Per language**\n\n| lang | n | recall | precision | fp_rate |\n|---|---:|---:|---:|---:|\n");
            for (lang, m) in &s.eval.per_language {
                let _ = writeln!(
                    out,
                    "| {lang} | {} | {:.3} | {:.3} | {:.3} |",
                    m.total, m.recall, m.precision, m.fp_rate
                );
            }
        }
        if !s.gates.is_empty() {
            out.push_str("\n**Gates**\n\n| gate | scope | observed | threshold | status |\n|---|---|---:|---:|---|\n");
            for gr in &s.gates {
                let scope = gr.gate.category.as_ref().map_or_else(
                    || gr.gate.suite.clone(),
                    |c| format!("{}/{}", gr.gate.suite, c),
                );
                let tag = if gr.passed { "✅ pass" } else { "❌ fail" };
                let _ = writeln!(
                    out,
                    "| `{}` | {scope} | {:.3} | {:.3} | {tag} |",
                    gr.gate.name, gr.value, gr.gate.threshold
                );
            }
        }
        out.push('\n');
    }
    out
}

/// Baseline comparison: what changed between `baseline` and `current`.
#[derive(Debug, Clone, Serialize)]
pub struct Comparison {
    pub suite_deltas: Vec<SuiteDelta>,
    pub gate_def_changed: Vec<String>,
    pub regressed: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuiteDelta {
    pub suite: String,
    pub mode: String,
    pub recall_delta: f64,
    pub fp_rate_delta: f64,
    pub f1_delta: f64,
}

/// Compare two reports; a metric movement below `noise` is ignored.
pub fn compare(baseline: &FullReport, current: &FullReport, noise: f64) -> Comparison {
    let mut suite_deltas = Vec::new();
    let mut regressed = Vec::new();

    // Index baseline by (suite, mode) for quick lookup.
    let mut baseline_suites: BTreeMap<(String, String), &SuiteReport> = BTreeMap::new();
    for s in &baseline.suites {
        baseline_suites.insert((s.suite.clone(), s.mode.clone()), s);
    }

    for cur in &current.suites {
        let key = (cur.suite.clone(), cur.mode.clone());
        if let Some(base) = baseline_suites.get(&key) {
            let rd = cur.eval.overall.recall - base.eval.overall.recall;
            let fpd = cur.eval.overall.fp_rate - base.eval.overall.fp_rate;
            let f1d = cur.eval.overall.f1 - base.eval.overall.f1;
            suite_deltas.push(SuiteDelta {
                suite: cur.suite.clone(),
                mode: cur.mode.clone(),
                recall_delta: rd,
                fp_rate_delta: fpd,
                f1_delta: f1d,
            });
            // Regression = recall decreased by more than noise, or fp_rate
            // increased by more than noise.
            if rd < -noise {
                regressed.push(format!(
                    "{}[{}] recall: {:.3} -> {:.3} ({:+.3})",
                    cur.suite, cur.mode, base.eval.overall.recall, cur.eval.overall.recall, rd
                ));
            }
            if fpd > noise {
                regressed.push(format!(
                    "{}[{}] fp_rate: {:.3} -> {:.3} ({:+.3})",
                    cur.suite, cur.mode, base.eval.overall.fp_rate, cur.eval.overall.fp_rate, fpd
                ));
            }
        }
    }

    // Detect changed gate definitions (threshold / scope / direction).
    let mut gate_def_changed = Vec::new();
    let base_map: BTreeMap<String, &GateInfo> = baseline
        .gates_def
        .iter()
        .map(|g| (g.name.clone(), g))
        .collect();
    for cg in &current.gates_def {
        if let Some(bg) = base_map.get(&cg.name) {
            if *bg != cg {
                gate_def_changed.push(cg.name.clone());
            }
        } else {
            gate_def_changed.push(format!("{} (new)", cg.name));
        }
    }

    Comparison {
        suite_deltas,
        gate_def_changed,
        regressed,
    }
}

pub fn print_comparison(cmp: &Comparison) {
    println!("\n── baseline comparison ──");
    for d in &cmp.suite_deltas {
        println!(
            "  {}[{}] Δrecall={:+.3}  Δfp_rate={:+.3}  Δf1={:+.3}",
            d.suite, d.mode, d.recall_delta, d.fp_rate_delta, d.f1_delta
        );
    }
    if !cmp.gate_def_changed.is_empty() {
        println!(
            "  gate definitions changed: {}",
            cmp.gate_def_changed.join(", ")
        );
    }
    if cmp.regressed.is_empty() {
        println!("  no regressions beyond noise threshold");
    } else {
        println!("  REGRESSIONS:");
        for r in &cmp.regressed {
            println!("    - {r}");
        }
    }
}

/// Pick the metric value a gate cares about from an EvalResult.
pub fn metric_value_for_gate(eval: &EvalResult, gate: &Gate) -> f64 {
    let metrics = match gate.category {
        Some(cat) => eval.per_category.get(cat).unwrap_or_else(|| {
            panic!("gate '{}' references missing category '{}'", gate.name, cat)
        }),
        None => &eval.overall,
    };
    match gate.metric {
        "precision" => metrics.precision,
        "recall" => metrics.recall,
        "f1" => metrics.f1,
        "fp_rate" => metrics.fp_rate,
        "accuracy" => metrics.accuracy,
        other => panic!("unknown metric name in gate: {other}"),
    }
}

/// Write failure JSONL: one line per (case, outcome) where outcome is FP or FN.
pub fn write_failures(
    path: &Path,
    suite: &str,
    mode: &str,
    cases: &[Case],
    outcomes: &[Outcome],
) -> Result<()> {
    use std::io::Write as _;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let mut file = std::io::BufWriter::new(file);
    for (c, o) in cases.iter().zip(outcomes.iter()) {
        if !o.is_failure() {
            continue;
        }
        let rec = serde_json::json!({
            "suite": suite,
            "mode": mode,
            "outcome": o,
            "text": c.text,
            "label": c.label,
            "category": c.category,
            "lang": c.lang,
            "note": c.note,
        });
        writeln!(file, "{}", serde_json::to_string(&rec)?)?;
    }
    Ok(())
}
