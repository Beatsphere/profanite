//! Profanite — obfuscation-resistant profanity filter.
//!
//! See `Profanite::builder()` for the entry point.

mod allowlist;
mod censor;
mod config;
mod data;
mod error;
mod lang;
mod matcher;
mod normalize;

pub use censor::CensorStyle;
pub use config::{MatchMode, NormalizationLevel, Profanite, ProfaniteBuilder};
pub use data::{Category, WordEntry};
pub use error::Error;
pub use lang::Lang;
pub use matcher::Match;
