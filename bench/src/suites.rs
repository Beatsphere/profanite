//! Suite definitions + orchestration.
//!
//! Each suite knows where to find its corpus file, what bundles of
//! languages to load in the filter, which normalization modes to run,
//! and which gates apply.

use crate::corpus;
use crate::gates::{self, Gate, GateInfo, GateResult};
use crate::metrics::{self, Outcome};
use crate::report::{self, FullReport, Metadata, SuiteReport};
use anyhow::{bail, Context, Result};
use profanite_core::{Lang, NormalizationLevel, Profanite, SemanticDetector, SemanticScorer};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const DATA_DIR: &str = "bench/data";

/// Options that change how a run is orchestrated.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub json_out: Option<PathBuf>,
    pub markdown_out: Option<PathBuf>,
    pub failures_out: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub baseline_noise: f64, // e.g. 0.005 = half a point
    pub mode_sweep: bool,
    /// Run each suite a second time with the ONNX-backed scorer
    /// attached and print the delta. Requires the `semantic` build
    /// feature.
    pub semantic: bool,
    /// Suppression threshold (low → encoder almost never kills keyword hits).
    pub suppression_threshold: f32,
    /// Detector threshold (recall recovery on keyword misses).
    pub detector_threshold: f32,
}

/// All suites we know how to run, in declared order.
fn registry() -> Vec<SuiteDef> {
    vec![
        SuiteDef {
            name: "synthetic",
            file: "bypass_corpus.jsonl",
            langs: &[Lang::En],
            gates: &[gates::BYPASS_CATCH],
        },
        SuiteDef {
            name: "hatecheck",
            file: "hatecheck.jsonl",
            langs: &[Lang::En, Lang::Es, Lang::Fr, Lang::De],
            gates: &[gates::HATECHECK_PROFANITY, gates::HATECHECK_LEET],
        },
        SuiteDef {
            name: "jigsaw",
            file: "jigsaw.jsonl",
            langs: &[Lang::En],
            gates: &[gates::JIGSAW_RECALL, gates::JIGSAW_FP_RATE],
        },
    ]
}

struct SuiteDef {
    name: &'static str,
    file: &'static str,
    langs: &'static [Lang],
    gates: &'static [Gate],
}

pub fn run_fast(opts: &RunOptions) -> Result<()> {
    run_selected(&["synthetic", "hatecheck"], opts)
}

pub fn run_full(opts: &RunOptions) -> Result<()> {
    let all: Vec<&str> = registry().iter().map(|s| s.name).collect();
    run_selected(&all, opts)
}

pub fn run_one(name: &str, opts: &RunOptions) -> Result<()> {
    run_selected(&[name], opts)
}

fn run_selected(names: &[&str], opts: &RunOptions) -> Result<()> {
    let reg = registry();
    let mut full = FullReport {
        metadata: Metadata::current(),
        gates_def: gates::ALL.iter().map(GateInfo::from).collect(),
        ..Default::default()
    };

    // Clear any previous failures file before we append to it.
    if let Some(path) = &opts.failures_out {
        let _ = std::fs::remove_file(path);
    }

    let modes: Vec<NormalizationLevel> = if opts.mode_sweep {
        vec![NormalizationLevel::Basic, NormalizationLevel::Aggressive]
    } else {
        vec![NormalizationLevel::Basic]
    };

    let mut any = false;

    let semantic: Option<SemanticHandles> = if opts.semantic {
        Some(load_semantic_handles()?)
    } else {
        None
    };

    for name in names {
        let def = reg
            .iter()
            .find(|s| s.name == *name)
            .with_context(|| format!("unknown suite: {name}"))?;

        let path = corpus_path(def.file);
        if !path.exists() {
            eprintln!(
                "[skip] suite `{}`: {} not found (see bench/scripts/ for fetchers)",
                def.name,
                path.display()
            );
            continue;
        }

        for mode in &modes {
            let baseline_suite = run_suite(def, &path, *mode, opts, None)?;
            report::print_suite(&baseline_suite);

            if let Some(handles) = &semantic {
                let semantic_suite = run_suite(
                    def,
                    &path,
                    *mode,
                    opts,
                    Some(SemanticAttach {
                        scorer: handles.scorer.clone(),
                        detector: handles.detector.clone(),
                        suppression_threshold: opts.suppression_threshold,
                        detector_threshold: opts.detector_threshold,
                    }),
                )?;
                report::print_semantic_delta(&baseline_suite, &semantic_suite);
                full.suites.push(semantic_suite);
            }

            full.suites.push(baseline_suite);
            any = true;
        }
    }

    if !any {
        bail!("no suites had data available. Did you run the fetch scripts?");
    }

    full.green = full
        .suites
        .iter()
        .flat_map(|s| s.gates.iter())
        .all(|g: &GateResult| g.passed);

    report::print_summary(&full);

    // Baseline comparison.
    let baseline_regressed = if let Some(bp) = &opts.baseline {
        match load_baseline(bp) {
            Ok(baseline) => {
                let cmp = report::compare(&baseline, &full, opts.baseline_noise);
                report::print_comparison(&cmp);
                !cmp.regressed.is_empty()
            }
            Err(e) => {
                eprintln!("[warn] could not load baseline {}: {e}", bp.display());
                false
            }
        }
    } else {
        false
    };

    if let Some(path) = &opts.json_out {
        report::write_json(path, &full)?;
    }
    if let Some(path) = &opts.markdown_out {
        report::write_markdown(path, &full)?;
    }

    if !full.green || baseline_regressed {
        std::process::exit(1);
    }
    Ok(())
}

struct SemanticHandles {
    scorer: Arc<dyn SemanticScorer>,
    detector: Arc<dyn SemanticDetector>,
}

#[derive(Clone)]
struct SemanticAttach {
    scorer: Arc<dyn SemanticScorer>,
    detector: Arc<dyn SemanticDetector>,
    suppression_threshold: f32,
    detector_threshold: f32,
}

fn run_suite(
    def: &SuiteDef,
    path: &Path,
    mode: NormalizationLevel,
    opts: &RunOptions,
    semantic: Option<SemanticAttach>,
) -> Result<SuiteReport> {
    let cases = corpus::load_jsonl(path)?;
    let attached = semantic.is_some();
    let filter = build_filter(def.langs, mode, semantic)?;

    let predictions: Vec<bool> = cases
        .iter()
        .map(|c| filter.contains_profanity(&c.text))
        .collect();

    let outcomes: Vec<Outcome> = metrics::classify(&cases, &predictions);
    let eval = metrics::evaluate(&cases, &outcomes);

    if let Some(failures_path) = &opts.failures_out {
        report::write_failures(failures_path, def.name, mode_name(mode), &cases, &outcomes)?;
    }

    // Gates are calibrated against the default (Basic) normalization.
    // When mode-sweeping, the Aggressive run is informational only —
    // we record the numbers but don't fail the build on them.
    let gates = if mode == NormalizationLevel::Basic {
        def.gates
            .iter()
            .map(|g| {
                let value = report::metric_value_for_gate(&eval, g);
                g.evaluate(value)
            })
            .collect()
    } else {
        Vec::new()
    };

    let corpus_sha256 =
        report::sha256_of(path).with_context(|| format!("hashing {}", path.display()))?;

    Ok(SuiteReport {
        suite: def.name.to_string(),
        mode: mode_name(mode).to_string(),
        cases: cases.len(),
        corpus_sha256,
        eval,
        gates,
        semantic_attached: attached,
    })
}

fn mode_name(m: NormalizationLevel) -> &'static str {
    match m {
        NormalizationLevel::None => "none",
        NormalizationLevel::Basic => "basic",
        NormalizationLevel::Aggressive => "aggressive",
    }
}

fn build_filter(
    langs: &[Lang],
    mode: NormalizationLevel,
    semantic: Option<SemanticAttach>,
) -> Result<Profanite> {
    let mut b = Profanite::builder().normalization(mode);
    for lang in langs {
        b = b.language(*lang);
    }
    if let Some(s) = semantic {
        b = b
            .scorer(s.scorer)
            .min_confidence(s.suppression_threshold)
            .detector(s.detector)
            .detector_threshold(s.detector_threshold);
    }
    Ok(b.build()?)
}

#[cfg(feature = "semantic")]
fn load_semantic_handles() -> Result<SemanticHandles> {
    eprintln!("[semantic] loading Xenova/toxic-bert int8 ONNX (first run downloads ~30 MB)…");
    let model = profanite_semantic::OnnxToxicScorer::from_pretrained()
        .with_context(|| "loading OnnxToxicScorer")?;
    let shared: Arc<profanite_semantic::OnnxToxicScorer> = Arc::new(model);
    eprintln!("[semantic] model ready.");
    Ok(SemanticHandles {
        scorer: shared.clone(),
        detector: shared,
    })
}

#[cfg(not(feature = "semantic"))]
fn load_semantic_handles() -> Result<SemanticHandles> {
    bail!(
        "this binary was built without the `semantic` feature; rebuild with \
         `cargo build --release -p profanite-bench --features semantic` to use --semantic"
    )
}

fn load_baseline(path: &Path) -> Result<FullReport> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

/// Run the full suite and write a compact Markdown stats block suitable
/// for splicing into the README. The block contains only stable,
/// human-readable summary rows — no timestamps or git revs — so
/// regenerating it only changes content when measurements change.
pub fn run_snapshot(out: &Path) -> Result<()> {
    let reg = registry();
    let mut lines = vec![
        "<!-- @generated by `cargo run -p profanite-bench -- snapshot`. Do not edit. -->"
            .to_string(),
        String::new(),
        "| Suite | Mode | n | recall | precision | fp_rate | f1 |".to_string(),
        "|---|---|---:|---:|---:|---:|---:|".to_string(),
    ];

    for def in &reg {
        let path = corpus_path(def.file);
        if !path.exists() {
            continue;
        }
        let suite = run_suite(
            def,
            &path,
            NormalizationLevel::Basic,
            &RunOptions::default(),
            None,
        )?;
        let m = &suite.eval.overall;
        lines.push(format!(
            "| {} | basic | {} | {:.3} | {:.3} | {:.3} | {:.3} |",
            suite.suite, suite.cases, m.recall, m.precision, m.fp_rate, m.f1
        ));
    }

    let ws_root = workspace_root();
    let final_path = if out.is_absolute() {
        out.to_path_buf()
    } else {
        ws_root.join(out)
    };
    std::fs::create_dir_all(final_path.parent().unwrap_or(&ws_root))?;
    std::fs::write(&final_path, lines.join("\n") + "\n")?;
    println!("wrote snapshot to {}", final_path.display());
    Ok(())
}

fn corpus_path(file: &str) -> PathBuf {
    let ws_root = workspace_root();
    ws_root.join(DATA_DIR).join(file)
}

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap_or(manifest).to_path_buf()
}
