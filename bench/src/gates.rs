//! Release quality gates.
//!
//! A gate is a named threshold on a metric of a specific suite (and
//! optionally a specific category within that suite). The fast suite
//! evaluates gates whose suite is "synthetic" or "hatecheck"; the full
//! suite evaluates all of them. A release is "green" when every
//! applicable gate passes.
//!
//! Gates are listed here by name so other code (CI, release scripts) can
//! treat them as the source of truth.

use serde::{Deserialize, Serialize};

/// A compile-time gate definition.
#[derive(Debug, Clone)]
pub struct Gate {
    pub name: &'static str,
    pub suite: &'static str,
    pub category: Option<&'static str>,
    pub metric: &'static str,
    pub threshold: f64,
    pub direction: Direction,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    AtLeast,
    AtMost,
}

/// Runtime representation of a gate with owned fields — used in JSON reports
/// where `&'static` is not deserializable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateInfo {
    pub name: String,
    pub suite: String,
    pub category: Option<String>,
    pub metric: String,
    pub threshold: f64,
    pub direction: Direction,
    pub description: String,
}

impl From<&Gate> for GateInfo {
    fn from(g: &Gate) -> Self {
        Self {
            name: g.name.into(),
            suite: g.suite.into(),
            category: g.category.map(str::to_string),
            metric: g.metric.into(),
            threshold: g.threshold,
            direction: g.direction,
            description: g.description.into(),
        }
    }
}

pub const BYPASS_CATCH: Gate = Gate {
    name: "bypass_catch_rate",
    suite: "synthetic",
    category: None,
    metric: "recall",
    threshold: 0.85,
    direction: Direction::AtLeast,
    description: "Synthetic bypass corpus: overall recall >= 0.85",
};

/// Jigsaw-slot recall (currently sourced from tdavidson/hate_speech_offensive
/// since the original jigsaw_toxicity_pred is no longer publicly
/// downloadable). Class 1 (offensive) -> profane.
///
/// 0.75 floor is calibrated to the current measured ~0.77. Lowering this
/// would silently allow regression; raising it demands real filter
/// improvements (better slang coverage, typo tolerance).
pub const JIGSAW_RECALL: Gate = Gate {
    name: "jigsaw_recall",
    suite: "jigsaw",
    category: None,
    metric: "recall",
    threshold: 0.75,
    direction: Direction::AtLeast,
    description: "Jigsaw proxy (offensive tweets): recall >= 0.75",
};

/// FP ceiling for the benign slice. Tweet data is noisier than curated
/// corpora — 5% is a realistic ceiling; allowlist + better overrides can
/// bring this down in v0.2.
pub const JIGSAW_FP_RATE: Gate = Gate {
    name: "jigsaw_fp_rate",
    suite: "jigsaw",
    category: None,
    metric: "fp_rate",
    threshold: 0.05,
    direction: Direction::AtMost,
    description: "Jigsaw proxy (benign tweets): false-positive rate <= 5%",
};

/// Multilingual profanity recall from HateCheck ES/FR/DE.
///
/// Scoped to the `profanity_h` category — sentences where HateCheck
/// deliberately used profane words. We don't gate on slurs (not bundled
/// at v0.1), char-deletion/swap (needs edit-distance matching), or
/// space-based bypass (needs Aggressive mode).
pub const HATECHECK_PROFANITY: Gate = Gate {
    name: "hatecheck_profanity_recall",
    suite: "hatecheck",
    category: Some("profanity_h"),
    metric: "recall",
    threshold: 0.35,
    direction: Direction::AtLeast,
    description: "HateCheck profanity_h: multilingual recall >= 0.35",
};

pub const HATECHECK_LEET: Gate = Gate {
    name: "hatecheck_leet_recall",
    suite: "hatecheck",
    category: Some("spell_leet_h"),
    metric: "recall",
    threshold: 0.10,
    direction: Direction::AtLeast,
    description: "HateCheck spell_leet_h: leet bypass recall >= 0.10",
};

pub const ALL: &[Gate] = &[
    BYPASS_CATCH,
    JIGSAW_RECALL,
    JIGSAW_FP_RATE,
    HATECHECK_PROFANITY,
    HATECHECK_LEET,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate: GateInfo,
    pub value: f64,
    pub passed: bool,
}

impl Gate {
    pub fn evaluate(&self, value: f64) -> GateResult {
        let passed = match self.direction {
            Direction::AtLeast => value >= self.threshold,
            Direction::AtMost => value <= self.threshold,
        };
        GateResult {
            gate: GateInfo::from(self),
            value,
            passed,
        }
    }
}

pub fn print_all() {
    println!("Release gates for profanite:\n");
    for g in ALL {
        let op = match g.direction {
            Direction::AtLeast => ">=",
            Direction::AtMost => "<=",
        };
        let scope = match g.category {
            Some(c) => format!("{}/{}", g.suite, c),
            None => g.suite.to_string(),
        };
        println!(
            "  [{}] {} :: {} {} {}",
            scope, g.name, g.metric, op, g.threshold
        );
        println!("        {}", g.description);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_passes_when_equal_or_greater() {
        assert!(BYPASS_CATCH.evaluate(0.85).passed);
        assert!(BYPASS_CATCH.evaluate(0.90).passed);
        assert!(!BYPASS_CATCH.evaluate(0.849).passed);
    }

    #[test]
    fn at_most_passes_when_equal_or_less() {
        assert!(JIGSAW_FP_RATE.evaluate(JIGSAW_FP_RATE.threshold).passed);
        assert!(
            JIGSAW_FP_RATE
                .evaluate(JIGSAW_FP_RATE.threshold / 2.0)
                .passed
        );
        assert!(
            !JIGSAW_FP_RATE
                .evaluate(JIGSAW_FP_RATE.threshold + 0.001)
                .passed
        );
    }

    #[test]
    fn gate_info_roundtrips_through_json() {
        let info: GateInfo = (&BYPASS_CATCH).into();
        let s = serde_json::to_string(&info).unwrap();
        let back: GateInfo = serde_json::from_str(&s).unwrap();
        assert_eq!(info, back);
    }

    #[test]
    fn every_gate_is_reachable_from_all() {
        // Sanity: ALL is the source of truth; each const must be in it.
        let names: Vec<&str> = ALL.iter().map(|g| g.name).collect();
        for g in [
            &BYPASS_CATCH,
            &JIGSAW_RECALL,
            &JIGSAW_FP_RATE,
            &HATECHECK_PROFANITY,
            &HATECHECK_LEET,
        ] {
            assert!(names.contains(&g.name), "{} missing from ALL", g.name);
        }
    }
}
