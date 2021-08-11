
use thiserror::Error;
use node_crunch::NCError;
use serde_json;

use std::io;

#[derive(Error, Debug)]
pub enum DWError {
    /// Node Crunch error
    #[error("Node Crunch error: {0}")]
    NCError(#[from] NCError),
    /// Common IO error
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    /// Serde JSON error
    #[error("Serde JSON error: {0}")]
    JSONError(#[from] serde_json::Error)
}