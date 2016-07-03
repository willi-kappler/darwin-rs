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

use std::time::Instant;
use std::sync::{Arc, Mutex};

use jobsteal::{make_pool, IntoSplitIterator, SplitIterator};

use individual::{Individual, IndividualWrapper};
use population::Population;

/// The `SimulationType` type. Speficies the criteria on how a simulation should stop.
#[derive(Debug,Clone)]
pub enum SimulationType {
    /// Finish the simulation when a number of iteration has been reached.
    EndIteration(u32),
    /// Finish the simulation when a specific fitness is rached.
    /// That means if at least one of the individuals has this fitness.
    /// The fitness is calculated using the implemented `calculate_fitness` functions
    /// of the `Individual` trait
    EndFitness(f64),
    /// Finish the simulation when a specific improvement factor is reached.
    /// That means the relation between the very first fitness and the current fitness of the
    /// fittest individual
    EndFactor(f64),
}

/// The `Simulation` type. Contains all the information / configuration for the simulation to run.
pub struct Simulation<T: Individual + Send + Sync> {
    /// How should the simulation stop ?
    pub type_of_simulation: SimulationType,
    /// The number of threads to use to speed up calculation.
    pub num_of_threads: usize,
    /// All the populations for the simulation. Contains all individuals for the simulation.
    pub habitat: Vec<Population<T>>,
    /// The total run time for the simulation. This will be calculated once the stimulation has
    /// finished.
    pub total_time_in_ms: f64,
    /// The result of the simulation: improvement_factor, original_fitness and a vector of
    /// fittest individuals
    pub simulation_result: SimulationResult<T>
}

/// The SimulationResult Type. TODO
#[derive(Clone)]
pub struct SimulationResult<T: Individual + Send + Sync> {
    /// The current improvement factor, that means the ration between the very first and the
    /// current fitness.
    pub improvement_factor: f64,
    /// The very first calculated fitness, when the simulation just started.
    pub original_fitness: f64,
    /// Vector of fittest individuals. This will change during the simulation as soon as a new
    /// more fittest individual is found and pushed into the first position (index 0).
    pub fittest: Vec<IndividualWrapper<T>>
}

/// This implements the two functions `run` and `print_fitness` for the struct `Simulation`.
impl<T: Individual + Send + Sync + Clone> Simulation<T> {
    /// This actually runs the simulation.
    /// Depending on the type of simulation (EndIteration, EndFactor or Endfitness) the iteration
    /// loop will check for the stop condition accordingly.
    pub fn run(&mut self) {
        // Initialize timer
        let start_time = Instant::now();

        // Calculate the fitness for all individuals in all populations at the beginning.
        for population in self.habitat.iter_mut() {
            population.calculate_fitness();
        }

        let mut iteration_counter = 0;
        let mut pool = make_pool(self.num_of_threads).unwrap();

        // Initialize:
        // - The fittest individual.
        // - The fitness at the beginning of the simulation. This is uesed to calculate the
        //   overall improvement later on.
        let simulation_result = SimulationResult {
            improvement_factor: 0.0,
            original_fitness: self.habitat[0].population[0].fitness,
            fittest: vec![self.habitat[0].population[0].clone()]
        };

        println!("original_fitness: {}", simulation_result.original_fitness);

        let simulation_result = Arc::new(Mutex::new(Box::new(simulation_result)));

        // Check which type of simulation to run.
        match self.type_of_simulation {
            SimulationType::EndIteration(end_iteration) => {
                for iteration_counter in 0..end_iteration {
                    (&mut self.habitat).into_split_iter().for_each(
                        &pool.spawner(), |population| {
                            population.run_body(&simulation_result, iteration_counter);
                        });
                };
            }
            SimulationType::EndFactor(end_factor) => {
                loop {
                    match simulation_result.lock() {
                        Ok(simulation_result) => {
                            if simulation_result.improvement_factor <= end_factor {
                                break;
                            }
                        },
                        Err(e) => println!("Mutex (poison) error (simulation_result): {}", e)
                    }

                    iteration_counter += 1;
                    (&mut self.habitat).into_split_iter().for_each(
                        &pool.spawner(), |population| {
                            population.run_body(&simulation_result, iteration_counter);
                        });
                }
            }
            SimulationType::EndFitness(end_fitness) => {
                loop {
                    match simulation_result.lock() {
                        Ok(simulation_result) => {
                            if simulation_result.fittest[0].fitness <= end_fitness {
                                break;
                            }
                        },
                        Err(e) => println!("Mutex (poison) error (simulation_result): {}", e)
                    }

                    iteration_counter += 1;
                    (&mut self.habitat).into_split_iter().for_each(
                        &pool.spawner(), |population| {
                            population.run_body(&simulation_result, iteration_counter);
                        });
                }
            }
        }

        let elapsed = start_time.elapsed();

        self.total_time_in_ms = elapsed.as_secs() as f64 * 1000.0 + elapsed.subsec_nanos() as f64 / 1000_000.0;

        // TODO
        // match simulation_result.lock() {
        //     Ok(simulation_result) => {
        //         self.simulation_result = (*(*simulation_result)).clone();
        //     },
        //     Err(e) => println!("Mutex (poison) error (simulation_result): {}", e)
        // }

    }

    /// This is a helper function that the user can call after the simulation stops in order to
    /// see all the fitness values for all the individuals that participated to the overall
    /// improvement.
    pub fn print_fitness(&self) {
        for wrapper in self.simulation_result.fittest.iter() {
            println!("fitness: {}, num_of_mutations: {}",
                     wrapper.fitness,
                     wrapper.num_of_mutations);
        }
    }
}
