
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
    JSONError(#[from] serde_json::Error),
    /// Parse DWMutateMethod error
    #[error("Could not parse DWMutateMethod: {0}")]
    ParseDWMethodError(String),
    /// Convert DWMutateMethod error
    #[error("Could not convert integer to DWMutateMethod: {0}")]
    ConvertDWMutateMethodError(u8),
    /// Convert DWDeleteMethod error
    #[error("Could not parse DEDeleteMethod: {0}")]
    ParseDWDeleteMethodError(String),
    /// Convert DWDeleteMethod error
    #[error("Could not convert integer to DWDeleteMethod: {0}")]
    ConvertDWDeleteMethodError(u8),
}
