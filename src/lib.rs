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
pub mod dw_simulation_server;
pub mod dw_simulation_node;
pub mod dw_error;

pub use dw_individual::DWIndividual;
pub use dw_simulation_server::{DWSimulationServer, DWFileFormat};
pub use dw_simulation_node::{DWSimulationNode, DWMethod};

pub use node_crunch::NCConfiguration;
