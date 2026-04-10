// EVOLVE-BLOCK-START
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
// EVOLVE-BLOCK-END
