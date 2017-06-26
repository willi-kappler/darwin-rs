//! darwin-rs: evolutionary algorithms with Rust
//!
//! Written by Willi Kappler, Version 0.4 (2017.06.26)
//!
//! Repository: https://github.com/willi-kappler/darwin-rs
//!
//! License: MIT
//!
//! This library allows you to write evolutionary algorithms (EA) in Rust.
//! Examples provided: TSP, Sudoku, Queens Problem, OCR
//!
//!

// For clippy
// #![feature(plugin)]
//
// #![plugin(clippy)]

// For error-chain
#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
extern crate jobsteal;

pub mod individual;
pub mod simulation;
pub mod simulation_builder;
pub mod population;
pub mod population_builder;

pub use individual::Individual;
pub use simulation::Simulation;
pub use simulation_builder::{SimulationBuilder};
pub use population::Population;
pub use population_builder::{PopulationBuilder};
