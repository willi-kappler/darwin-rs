//! This module defines helper functions (builder pattern) to create a valid simulation.
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

use simulation::{Simulation, SimulationType, SimulationResult};
use individual::{Individual};
use population::Population;

/// This is a helper struct in order to build (configure) a valid simulation.
/// See builder pattern: https://en.wikipedia.org/wiki/Builder_pattern
///
/// Maybe use phantom types, see https://github.com/willi-kappler/darwin-rs/issues/9
pub struct SimulationBuilder<T: Individual + Send + Sync> {
    /// The actual simulation.
    simulation: Simulation<T>,
}

error_chain! {
    errors {
        EndIterationTooLow
    }
}

/// This implementation contains all the helper method to build (configure) a valid simulation.
impl<T: Individual + Send + Sync> SimulationBuilder<T> {
    /// Start with this method, it must always be called as the first one.
    /// It creates a default simulation with some dummy (but invalid) values.
    pub fn new() -> SimulationBuilder<T> {
        SimulationBuilder {
            simulation: Simulation {
                type_of_simulation: SimulationType::EndIteration(10),
                num_of_threads: 2,
                habitat: Vec::new(),
                total_time_in_ms: 0.0,
                simulation_result: SimulationResult {
                    improvement_factor: std::f64::MAX,
                    original_fitness: std::f64::MAX,
                    fittest: Vec::new(),
                    iteration_counter: 0
                },
                share_fittest: false,
                num_of_global_fittest: 10,
                output_every: 10,
                output_every_counter: 0,
                share_every: 10,
                share_counter: 0
            },
        }
    }

    /// Set the total number of iterations for the simulation and thus sets the simulation
    /// type to `EndIteration`. (Only usefull in combination with `EndIteration`).
    pub fn iterations(mut self, iterations: u32) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndIteration(iterations);
        self
    }

    /// Set the improvement factor stop criteria for the simulation and thus sets the simulation
    /// type to `EndFactor`. (Only usefull in combination with `EndFactor`).
    pub fn factor(mut self, factor: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFactor(factor);
        self
    }

    /// Set the minimum fitness stop criteria for the simulation and thus sets the simulation
    /// type to `EndFitness`. (Only usefull in combination with `EndFactor`).
    pub fn fitness(mut self, fitness: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFitness(fitness);
        self
    }

    /// Sets the number of threads in order to speed up the simulation.
    pub fn threads(mut self, threads: usize) -> SimulationBuilder<T> {
        self.simulation.num_of_threads = threads;
        self
    }

    /// Add a population to the simulation.
    pub fn add_population(mut self, population: Population<T>) -> SimulationBuilder<T> {
        self.simulation.habitat.push(population);
        self
    }

    /// Add multiple populations to the simulation.
    pub fn add_multiple_populations(mut self, multiple_populations: Vec<Population<T>>) -> SimulationBuilder<T> {
        for population in multiple_populations {
            self.simulation.habitat.push(population);
        }
        self
    }

    /// If this option is enabled (default: off), then the fittest individual of all populations
    /// is shared between all populations.
    pub fn share_fittest(mut self) -> SimulationBuilder<T> {
        self.simulation.share_fittest = true;
        self
    }

    /// How many global fittest should be kept ? (The size of the "high score list")
    pub fn num_of_global_fittest(mut self, num_of_global_fittest: usize) -> SimulationBuilder<T> {
        self.simulation.num_of_global_fittest = num_of_global_fittest;
        self
    }

    /// Do not output every time a new individual is found, only every nth time.
    /// n == output_every
    pub fn output_every(mut self, output_every: u32) -> SimulationBuilder<T> {
        self.simulation.output_every = output_every;
        self
    }

    /// If share fittest is enabled and the number share_every of iteration has passed then
    /// the fittest individual is shared between all populations
    pub fn share_every(mut self, share_every: u32) -> SimulationBuilder<T> {
        self.simulation.share_every = share_every;
        self
    }

    /// This checks the configuration of the simulation and returns an error or Ok if no errors
    /// where found.
    pub fn finalize(self) -> Result<Simulation<T>> {
        match self.simulation {
            Simulation { type_of_simulation: SimulationType::EndIteration(0...9), .. } => {
                Err(ErrorKind::EndIterationTooLow.into())
            }
            _ => Ok(self.simulation),
        }
    }
}
