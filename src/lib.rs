//! darwin-rs: evolutionary algorithms with Rust
//!
//! Repository: https://github.com/willi-kappler/darwin-rs
//!
//! License: MIT
//!
//! This library allows you to write evolutionary algorithms (EA) in Rust.
//! Examples provided: TSP, Sudoku, Queens Problem, OCR
//!
//!

pub mod dw_individual;
pub mod dw_server;
pub mod dw_node;
pub mod dw_error;
pub mod dw_config;
pub mod dw_population;

pub use dw_individual::DWIndividual;
pub use dw_server::{DWServer, DWFileFormat};
pub use dw_node::{DWNode, DWMutateMethod};
pub use dw_population::DWDeleteMethod;
pub use dw_config::DWConfiguration;

pub use node_crunch::NCConfiguration;
