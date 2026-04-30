//! Classification metrics for a binary "profane vs benign" decision.
//!
//! This module keeps per-case outcomes so callers can both compute
//! aggregate metrics AND dump the specific cases that failed.

use crate::corpus::{Case, Label};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How a single case was classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    TruePositive,  // profane, correctly flagged
    TrueNegative,  // benign, correctly not flagged
    FalsePositive, // benign, incorrectly flagged
    FalseNegative, // profane, incorrectly not flagged
}

impl Outcome {
    pub fn new(truth_profane: bool, predicted_profane: bool) -> Self {
        match (truth_profane, predicted_profane) {
            (true, true) => Outcome::TruePositive,
            (true, false) => Outcome::FalseNegative,
            (false, true) => Outcome::FalsePositive,
            (false, false) => Outcome::TrueNegative,
        }
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, Outcome::FalsePositive | Outcome::FalseNegative)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub total: usize,
    pub tp: usize,
    pub fn_: usize,
    pub fp: usize,
    pub tn: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fp_rate: f64,
    pub accuracy: f64,
}

impl Metrics {
    pub fn from_counts(tp: usize, fn_: usize, fp: usize, tn: usize) -> Self {
        let total = tp + fn_ + fp + tn;
        // When a slice contains no positives, recall is conventionally 1.0
        // (nothing to miss). Same for precision when no predicted positives.
        let precision = if tp + fp == 0 {
            1.0
        } else {
            tp as f64 / (tp + fp) as f64
        };
        let recall = if tp + fn_ == 0 {
            1.0
        } else {
            tp as f64 / (tp + fn_) as f64
        };
        let f1 = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        let fp_rate = if fp + tn == 0 {
            0.0
        } else {
            fp as f64 / (fp + tn) as f64
        };
        let accuracy = if total == 0 {
            1.0
        } else {
            (tp + tn) as f64 / total as f64
        };
        Self {
            total,
            tp,
            fn_,
            tn,
            fp,
            precision,
            recall,
            f1,
            fp_rate,
            accuracy,
        }
    }

    pub fn from_outcomes<'a, I: IntoIterator<Item = &'a Outcome>>(it: I) -> Self {
        let mut tp = 0;
        let mut fn_ = 0;
        let mut fp = 0;
        let mut tn = 0;
        for o in it {
            match o {
                Outcome::TruePositive => tp += 1,
                Outcome::FalseNegative => fn_ += 1,
                Outcome::FalsePositive => fp += 1,
                Outcome::TrueNegative => tn += 1,
            }
        }
        Metrics::from_counts(tp, fn_, fp, tn)
    }
}

/// Produce per-case outcomes from truth labels + predictions.
pub fn classify(cases: &[Case], predicted_profane: &[bool]) -> Vec<Outcome> {
    assert_eq!(
        cases.len(),
        predicted_profane.len(),
        "predictions and cases must have the same length"
    );
    cases
        .iter()
        .zip(predicted_profane.iter())
        .map(|(c, &p)| Outcome::new(matches!(c.label, Label::Profane), p))
        .collect()
}

/// Overall + per-category + per-language breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub overall: Metrics,
    pub per_category: BTreeMap<String, Metrics>,
    pub per_language: BTreeMap<String, Metrics>,
}

pub fn evaluate(cases: &[Case], outcomes: &[Outcome]) -> EvalResult {
    assert_eq!(cases.len(), outcomes.len());

    let overall = Metrics::from_outcomes(outcomes.iter());

    let mut per_category: BTreeMap<&str, Vec<Outcome>> = BTreeMap::new();
    let mut per_language: BTreeMap<&str, Vec<Outcome>> = BTreeMap::new();
    for (c, o) in cases.iter().zip(outcomes.iter()) {
        if let Some(cat) = c.category.as_deref() {
            per_category.entry(cat).or_default().push(*o);
        }
        let lang = c.lang.as_deref().unwrap_or("en");
        per_language.entry(lang).or_default().push(*o);
    }

    let per_category = per_category
        .into_iter()
        .map(|(k, v)| (k.to_string(), Metrics::from_outcomes(v.iter())))
        .collect();
    let per_language = per_language
        .into_iter()
        .map(|(k, v)| (k.to_string(), Metrics::from_outcomes(v.iter())))
        .collect();

    EvalResult {
        overall,
        per_category,
        per_language,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(label: Label, category: &str, lang: &str) -> Case {
        Case {
            text: "x".into(),
            label,
            category: Some(category.into()),
            lang: Some(lang.into()),
            note: None,
        }
    }

    #[test]
    fn from_counts_perfect_score() {
        let m = Metrics::from_counts(10, 0, 0, 10);
        assert_eq!(m.precision, 1.0);
        assert_eq!(m.recall, 1.0);
        assert_eq!(m.f1, 1.0);
        assert_eq!(m.fp_rate, 0.0);
        assert_eq!(m.accuracy, 1.0);
    }

    #[test]
    fn from_counts_all_wrong() {
        // 0 tp, 10 fn, 10 fp, 0 tn: precision 0, recall 0 -> f1 = 0
        let m = Metrics::from_counts(0, 10, 10, 0);
        assert_eq!(m.precision, 0.0);
        assert_eq!(m.recall, 0.0);
        assert_eq!(m.f1, 0.0);
        assert_eq!(m.fp_rate, 1.0);
        assert_eq!(m.accuracy, 0.0);
    }

    #[test]
    fn from_counts_no_positives_defaults_recall_to_1() {
        // Only benign cases in this slice.
        let m = Metrics::from_counts(0, 0, 0, 10);
        assert_eq!(m.recall, 1.0);
    }

    #[test]
    fn outcome_classification() {
        assert_eq!(Outcome::new(true, true), Outcome::TruePositive);
        assert_eq!(Outcome::new(true, false), Outcome::FalseNegative);
        assert_eq!(Outcome::new(false, true), Outcome::FalsePositive);
        assert_eq!(Outcome::new(false, false), Outcome::TrueNegative);
    }

    #[test]
    fn evaluate_populates_per_category_and_per_language() {
        let cases = vec![
            case(Label::Profane, "leet", "en"),
            case(Label::Profane, "leet", "es"),
            case(Label::Benign, "other", "en"),
        ];
        let outcomes = vec![
            Outcome::TruePositive,
            Outcome::FalseNegative,
            Outcome::TrueNegative,
        ];
        let e = evaluate(&cases, &outcomes);

        assert_eq!(e.overall.tp, 1);
        assert_eq!(e.overall.fn_, 1);
        assert_eq!(e.overall.tn, 1);

        assert_eq!(e.per_category["leet"].total, 2);
        assert_eq!(e.per_category["leet"].recall, 0.5);
        assert_eq!(e.per_language["en"].total, 2);
        assert_eq!(e.per_language["es"].total, 1);
        assert_eq!(e.per_language["es"].recall, 0.0);
    }
}
