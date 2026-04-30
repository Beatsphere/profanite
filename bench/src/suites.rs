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
use profanite_core::{Lang, NormalizationLevel, Profanite};
use std::path::{Path, PathBuf};

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
            let suite = run_suite(def, &path, *mode, opts)?;
            report::print_suite(&suite);
            full.suites.push(suite);
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

fn run_suite(
    def: &SuiteDef,
    path: &Path,
    mode: NormalizationLevel,
    opts: &RunOptions,
) -> Result<SuiteReport> {
    let cases = corpus::load_jsonl(path)?;
    let filter = build_filter(def.langs, mode)?;

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
    })
}

fn mode_name(m: NormalizationLevel) -> &'static str {
    match m {
        NormalizationLevel::None => "none",
        NormalizationLevel::Basic => "basic",
        NormalizationLevel::Aggressive => "aggressive",
    }
}

fn build_filter(langs: &[Lang], mode: NormalizationLevel) -> Result<Profanite> {
    let mut b = Profanite::builder().normalization(mode);
    for lang in langs {
        b = b.language(*lang);
    }
    Ok(b.build()?)
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
