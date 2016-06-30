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

use itertools::Itertools;

use individual::{Individual, IndividualWrapper};

struct Population<T: Individual + Send + Sync> {
    population: Vec<IndividualWrapper<T>>
}

impl<T: Individual + Send + Sync + Clone> Population<T> {
    fn mutate(&mut self) {
        self.population.iter_mut().foreach(|wrapper| {
            for _ in 0..wrapper.num_of_mutations {
                wrapper.individual.mutate();
            }
            wrapper.fitness = wrapper.individual.calculate_fitness();
        });
    }
}
