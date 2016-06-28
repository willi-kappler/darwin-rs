//! darwin-rs: evolutionary algorithms with Rust
//!
//! Written by Willi Kappler, Version 0.2 (2016.07.xx)
//!
//! Repository: https://github.com/willi-kappler/darwin-rs
//!
//! License: MIT
//!
//! This library allows you to write evolutionary algorithms (EA) in Rust.
//! Examples provided: TSP, Sudoku, Queens Problem
//!
//!

// For clippy
// #![feature(plugin)]
//
// #![plugin(clippy)]

#[macro_use] extern crate quick_error;
extern crate jobsteal;

pub mod individual;
pub mod simulation;
pub mod simulation_builder;
