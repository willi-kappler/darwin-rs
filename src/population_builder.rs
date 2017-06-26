//! This module defines helper functions (builder pattern) to create a valid population.
//!
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

use std;

use individual::{Individual, IndividualWrapper};
use population::Population;

/// This is a helper struct in order to build (configure) a valid population.
/// See builder pattern: https://en.wikipedia.org/wiki/Builder_pattern
///
/// Maybe use phantom types, see https://github.com/willi-kappler/darwin-rs/issues/9
pub struct PopulationBuilder<T: Individual> {
    /// The actual simulation
    population: Population<T>,
}

error_chain! {
    errors {
        IndividualsTooLow
        LimitEndTooLow
    }
}

/// This implementation contains all the helper method to build (configure) a valid population.
impl<T: Individual + Clone> PopulationBuilder<T> {
    /// Start with this method, it must always be called as the first one.
    /// It creates a default population with some dummy (but invalid) values.
    pub fn new() -> PopulationBuilder<T> {
        PopulationBuilder {
            population: Population {
                num_of_individuals: 0,
                population: Vec::new(),
                reset_limit: 0,
                reset_limit_start: 1000,
                reset_limit_end: 10000,
                reset_limit_increment: 1000,
                reset_counter: 0,
                id: 1,
                fitness_counter: 0
            }
        }
    }

    /// Sets the initial population provided inside a vector, length must be >= 3
    pub fn initial_population(mut self, individuals: &[T]) -> PopulationBuilder<T> {
        self.population.num_of_individuals = individuals.len() as u32;

        for individual in individuals {
            self.population.population.push(IndividualWrapper {
                individual: (*individual).clone(),
                fitness: std::f64::MAX,
                num_of_mutations: 1,
                id: self.population.id,
            });
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals
    /// in the population: The first individual will mutate once, the second will mutate twice,
    /// the nth individual will Mutate n-times per iteration.
    pub fn increasing_mutation_rate(mut self) -> PopulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in &mut self.population.population {
            wrapper.num_of_mutations = mutation_rate;
            mutation_rate += 1;
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the
    /// population: Instead of a linear growing mutation rate like in the
    /// `increasing_mutation_rate` function above this sets an exponention mutation rate for
    /// all the individuals. The first individual will mutate base^1 times, the second will
    /// mutate base^2 times, and nth will mutate base^n times per iteration.
    pub fn increasing_exp_mutation_rate(mut self, base: f64) -> PopulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in &mut self.population.population {
            wrapper.num_of_mutations = base.powi(mutation_rate).floor() as u32;
            mutation_rate += 1;
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the
    /// population: This allows to specify an arbitrary mutation scheme for each individual.
    /// The number of rates must be equal to the number of individuals.
    pub fn mutation_rate(mut self, mutation_rate: Vec<u32>) -> PopulationBuilder<T> {
        // TODO: better error handling
        assert!(self.population.population.len() == mutation_rate.len());

        for (individual, mutation_rate) in self.population
            .population
            .iter_mut()
            .zip(mutation_rate.into_iter()) {
            individual.num_of_mutations = mutation_rate;
        }

        self
    }

    /// Configures the reset limit for the population. If reset_limit_end is greater than zero
    /// then a reset counter is increased each iteration. If that counter is greater than the
    /// limit, all individuals will be resetted, the limit will be increased by 1000 and the
    /// counter is set back to zero. Default value for reset_limit_start is 1000.
    pub fn reset_limit_start(mut self, reset_limit_start: u32) -> PopulationBuilder<T> {
        self.population.reset_limit_start = reset_limit_start;
        self.population.reset_limit = reset_limit_start;
        self
    }

    /// Configures the end value for the reset_limit. If the reset_limit >= reset_limit_end
    /// then the reset_limit will be resetted to the start value reset_limit_start.
    /// Default value for reset_limit_end is 100000.
    /// If reset_limit_end == 0 then the reset limit feature will be disabled.
    pub fn reset_limit_end(mut self, reset_limit_end: u32) -> PopulationBuilder<T> {
        self.population.reset_limit_end = reset_limit_end;
        self
    }

    /// Configure the increment for the reset_limit. If the reset_limit is reached, its value
    /// is incrementet by the amount of reset_limit_increment.
    pub fn reset_limit_increment(mut self, reset_limit_increment: u32) -> PopulationBuilder<T> {
        self.population.reset_limit_increment = reset_limit_increment;
        self
    }

    /// Set the population id. Currently this is only used for statistics.
    pub fn set_id(mut self, id: u32) -> PopulationBuilder<T> {
        for individual in &mut self.population.population {
            individual.id = id;
        }

        self.population.id = id;
        self
    }

    /// This checks the configuration of the simulation and returns an PopError or Ok if no PopErrors
    /// where found.
    pub fn finalize(self) -> Result<Population<T>> {
        match self.population {
            Population { num_of_individuals: 0...2, ..} => {
                Err(ErrorKind::IndividualsTooLow.into())
            }
            Population { reset_limit_start: start,
                         reset_limit_end: end, ..} if (end > 0) && (start >= end) => {
                Err(ErrorKind::LimitEndTooLow.into())
            }
            _ => Ok(self.population)
        }
    }
}
