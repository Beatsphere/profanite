//! Profanite evaluation harness.

mod corpus;
mod gates;
mod metrics;
mod report;
mod suites;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use suites::RunOptions;

#[derive(Parser)]
#[command(name = "profanite-bench")]
#[command(about = "Evaluate profanite against labeled corpora", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Options shared by every run-style subcommand.
#[derive(Parser, Debug, Clone, Default)]
struct RunArgs {
    /// Write the full JSON report to this path.
    #[arg(long)]
    json: Option<PathBuf>,

    /// Write a GitHub-flavored Markdown report to this path.
    #[arg(long)]
    markdown: Option<PathBuf>,

    /// Append every FP/FN case to this JSONL file for inspection.
    #[arg(long)]
    dump_failures: Option<PathBuf>,

    /// Compare against a previously-saved JSON report and exit non-zero on
    /// regression.
    #[arg(long)]
    baseline: Option<PathBuf>,

    /// Metric movements below this value are treated as noise when comparing
    /// against a baseline. Default 0.005 (half a percentage point).
    #[arg(long, default_value_t = 0.005)]
    baseline_noise: f64,

    /// Run each suite under both Basic and Aggressive normalization modes.
    #[arg(long)]
    mode_sweep: bool,
}

impl From<RunArgs> for RunOptions {
    fn from(a: RunArgs) -> Self {
        RunOptions {
            json_out: a.json,
            markdown_out: a.markdown,
            failures_out: a.dump_failures,
            baseline: a.baseline,
            baseline_noise: a.baseline_noise,
            mode_sweep: a.mode_sweep,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Run the fast suite (synthetic + HateCheck). Meant for every PR.
    Fast(RunArgs),
    /// Run every available suite.
    Full(RunArgs),
    /// Run one specific suite by name.
    Suite {
        name: String,
        #[command(flatten)]
        args: RunArgs,
    },
    /// Print the list of available release gates and their thresholds.
    Gates,
    /// Run the full suite and write a compact README-ready stats block to
    /// `bench/STATS.md`. Used by scripts/sync-readme.py so the numbers
    /// printed in the README can't drift from the latest measured values.
    Snapshot {
        /// Output path. Defaults to bench/STATS.md at the workspace root.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Fast(args) => suites::run_fast(&args.into()),
        Command::Full(args) => suites::run_full(&args.into()),
        Command::Suite { name, args } => suites::run_one(&name, &args.into()),
        Command::Gates => {
            gates::print_all();
            Ok(())
        }
        Command::Snapshot { out } => {
            let out_path = out.unwrap_or_else(|| std::path::PathBuf::from("bench/STATS.md"));
            suites::run_snapshot(&out_path)
        }
    }
}
