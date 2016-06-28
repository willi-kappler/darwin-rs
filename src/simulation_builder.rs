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

use std;
use jobsteal::{make_pool};

use simulation::{Simulation, SimulationType};
use individual::{Individual, IndividualWrapper};

/// This is a helper struct in order to build (configure) a valid simulation.
/// See builder pattern: https://en.wikipedia.org/wiki/Builder_pattern
pub struct SimulationBuilder<T: Individual + Send + Sync> {
    /// The actual simulation
    simulation: Simulation<T>,
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        /// The number of iteration is too low, should be >= 10
        TooLowEndIteration {}
        /// The number of individuals is too low, should be >= 3
        TooLowIndividuals {}
    }
}

pub type Result<T> = std::result::Result<Simulation<T>, Error>;

/// This implementation contains all the helper method to build (configure) a valid simulation
impl<T: Individual + Send + Sync> SimulationBuilder<T> {
    /// Start with this method, it must always be called as the first one.
    /// It creates a default simulation with some dummy (but invalid) values.
    pub fn new() -> SimulationBuilder<T> {
        SimulationBuilder {
            simulation: Simulation {
                type_of_simulation: SimulationType::EndIteration(10),
                num_of_individuals: 0,
                num_of_threads: 2,
                improvement_factor: std::f64::MAX,
                original_fitness: std::f64::MAX,
                fittest: IndividualWrapper {
                    individual: Individual::new(),
                    fitness: std::f64::MAX,
                    num_of_mutations: 1,
                },
                population: Vec::new(),
                total_time_in_ms: 0.0,
                iteration_counter: 0,
                reset_limit: 1000,
                reset_limit_start: 1000,
                reset_limit_end: 100000,
                reset_counter: 0,
                output_new_fittest: true,
                pool: make_pool(4).unwrap(),
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
    /// type to `Endfitness`. (Only usefull in combination with `EndFactor`).
    pub fn fitness(mut self, fitness: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFitness(fitness);
        self
    }

    /// Sets the number of individuals and creates the population, must be >= 3
    pub fn individuals(mut self, individuals: u32) -> SimulationBuilder<T> {
        self.simulation.num_of_individuals = individuals;

        for _ in 0..individuals {
            self.simulation.population.push(IndividualWrapper {
                individual: Individual::new(),
                fitness: std::f64::MAX,
                num_of_mutations: 1,
            });
        }

        self
    }

    /// Sets the number of threads in order to speed up the simulation.
    pub fn threads(mut self, threads: usize) -> SimulationBuilder<T> {
        self.simulation.num_of_threads = threads;
        self
    }

    /// Sets a flag if the simulation should write a message whenever a new fittest
    ///  individual is found.
    pub fn output_new_fittest(mut self, output_new_fittest: bool) -> SimulationBuilder<T> {
        self.simulation.output_new_fittest = output_new_fittest;
        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals
    /// in the sumulation: The first individual will mutate once, the second will mutate twice,
    /// the nth individual will Mutate n-times per iteration.
    pub fn increasing_mutation_rate(mut self) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in &mut self.simulation.population {
            wrapper.num_of_mutations = mutation_rate;
            mutation_rate += 1;
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the
    /// sumulation: Instead of a linear growing mutation rate like in the
    /// `increasing_mutation_rate` function above this sets an exponention mutation rate for
    /// all the individuals. The first individual will mutate base^1 times, the second will
    /// mutate base^2 times, and nth will mutate base^n times per iteration.
    pub fn increasing_exp_mutation_rate(mut self, base: f64) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in &mut self.simulation.population {
            wrapper.num_of_mutations = base.powi(mutation_rate).floor() as u32;
            mutation_rate += 1;
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the
    /// simulation: This allows to specify an arbitrary mutation scheme for each individual.
    /// The number of rates must be equal to the number of individuals.
    pub fn mutation_rate(mut self, mutation_rate: Vec<u32>) -> SimulationBuilder<T> {
        // TODO: better error handling
        assert!(self.simulation.population.len() == mutation_rate.len());

        for (individual, mutation_rate) in self.simulation
            .population
            .iter_mut()
            .zip(mutation_rate.into_iter()) {
            individual.num_of_mutations = mutation_rate;
        }

        self
    }

    /// Configures the reset limit for the simulation. If reset_limit_end is greater than zero
    /// then a reset counter is increased each iteration. If that counter is greater than the
    /// limit, all individuals will be resetted, the limit will be increased by 1000 and the
    /// counter is set back to zero. Default value for reset_limit_start is 1000.
    pub fn reset_limit_start(mut self, reset_limit_start: u32) -> SimulationBuilder<T> {
        self.simulation.reset_limit_start = reset_limit_start;

        self
    }

    /// Configures the end value for the reset_limit. If the reset_limit >= reset_limit_end
    /// then the reset_limit will be resetted to the start value reset_limit_start.
    /// Default value for reset_limit_end is 100000.
    /// If reset_limit_end == 0 then the reset limit feature will be disabled.
    pub fn reset_limit_end(mut self, reset_limit_end: u32) -> SimulationBuilder<T> {
        self.simulation.reset_limit_end = reset_limit_end;

        self
    }

    /// This checks the configuration of the simulation and returns an error or Ok if no errors
    /// where found.
    pub fn finalize(self) -> Result<T> {
        match self.simulation {
            Simulation { num_of_individuals: 0...2, .. } => Err(Error::TooLowIndividuals),
            Simulation { type_of_simulation: SimulationType::EndIteration(0...9), .. } => {
                Err(Error::TooLowEndIteration)
            }
            simulation => Ok(simulation),
        }
    }
}
