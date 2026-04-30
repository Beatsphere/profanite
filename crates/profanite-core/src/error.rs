use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no words configured; enable a language feature or call add_words")]
    EmptyWordlist,

    #[error("failed to build matcher automaton: {0}")]
    AutomatonBuild(String),
}
