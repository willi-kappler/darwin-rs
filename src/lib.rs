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

pub mod individual;
pub mod simulation_server;
pub mod simulation_node;

pub use individual::Individual;
pub use simulation_server::SimulationServer;
pub use simulation_node::SimulationNode;
