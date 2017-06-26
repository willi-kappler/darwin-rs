//! This module defines structures and methods for an EA simulation.
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

use std::time::Instant;

use jobsteal::make_pool;

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
    /// of the `Individual` trait.
    EndFitness(f64),
    /// Finish the simulation when a specific improvement factor is reached.
    /// That means the relation between the very first fitness and the current fitness of the
    /// fittest individual.
    EndFactor(f64),
}

/// The `Simulation` type. Contains all the information / configuration for the simulation to run.
/// Use the `SimulationBuilder` in order to create a simulation.
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
    /// The result of the simulation: `improvement_factor`, `original_fitness` and a vector of
    /// fittest individuals.
    pub simulation_result: SimulationResult<T>,
    /// If this feature is enabled, then the most fittest individual of all populations is
    /// shared between all the populations.
    pub share_fittest: bool,
    /// The total number of global fittest individual to keep, default: 10
    /// After each interation the most fittest individual of all populations is determinded.
    /// And this individual is copied into a global "high score list" of the whole simulation,
    /// if it is better then the highest entry.
    /// This number specifies how many of these global individuals should be kept.
    /// (i.e. the size of the "high score list")
    pub num_of_global_fittest: usize,
    /// Do not output every time a new fittest individual is found, only every nth times.
    /// n == output_every
    pub output_every: u32,
    /// Counter that will be incremented every iteration. If output_every_counter > output_every then
    /// the new fittest individual will be written to the log.
    pub output_every_counter: u32,
    /// Only share the most fittest individual between the populations if the counter reaches
    /// this value: share_counter >= share_every.
    pub share_every: u32,
    /// Counter that will be incremented every iteration. If share_counter >= share_every then the
    /// most fittest individual is shared between all the populations.
    pub share_counter: u32
}

/// The `SimulationResult` Type. Holds the simulation results:
/// All the fittest individuals, the `improvement_factor`, the `iteration_counter` and the
/// `original_fitness`.
#[derive(Clone)]
pub struct SimulationResult<T: Individual + Send + Sync> {
    /// The current improvement factor, that means the ration between the very first and the
    /// current fitness.
    pub improvement_factor: f64,
    /// The very first calculated fitness, when the simulation just started.
    pub original_fitness: f64,
    /// Vector of fittest individuals. This will change during the simulation as soon as a new
    /// more fittest individual is found and pushed into the first position (index 0).
    pub fittest: Vec<IndividualWrapper<T>>,
    /// How many iteration did the simulation run.
    pub iteration_counter: u32
}

/// This implements the the functions `run`, `print_fitness` and `update_results` (private)
/// for the struct `Simulation`.
impl<T: Individual + Send + Sync + Clone> Simulation<T> {
    /// This actually runs the simulation.
    /// Depending on the type of simulation (`EndIteration`, `EndFactor` or `EndFitness`)
    /// the iteration loop will check for the stop condition accordingly.
    pub fn run(&mut self) {
        // Initialize timer
        let start_time = Instant::now();

        // Calculate the fitness for all individuals in all populations at the beginning.
        for population in &mut self.habitat {
            population.calculate_fitness();
        }

        let mut iteration_counter = 0;
        let mut pool = make_pool(self.num_of_threads).unwrap();

        // Initialize:
        // - The fittest individual.
        // - The fitness at the beginning of the simulation. This is uesed to calculate the
        //   overall improvement later on.
        self.simulation_result = SimulationResult {
            improvement_factor: 0.0,
            original_fitness: self.habitat[0].population[0].fitness,
            fittest: vec![self.habitat[0].population[0].clone()],
            iteration_counter: 0
        };

        info!("original_fitness: {}", self.simulation_result.original_fitness);

        // Check which type of simulation to run.
        match self.type_of_simulation {
            SimulationType::EndIteration(end_iteration) => {
                for _ in 0..end_iteration {
                    pool.scope(|scope|
                        for population in &mut self.habitat {
                            scope.submit(move || { population.run_body() });
                        });

                    self.update_results();
                };
                self.simulation_result.iteration_counter = end_iteration;
            }

            SimulationType::EndFactor(end_factor) => {
                loop {
                    iteration_counter += 1;
                    pool.scope(|scope|
                        for population in &mut self.habitat {
                            scope.submit(move || { population.run_body() });
                        });

                    self.update_results();

                    if self.simulation_result.improvement_factor <= end_factor {
                        break;
                    }
                };
                self.simulation_result.iteration_counter = iteration_counter;
            }

            SimulationType::EndFitness(end_fitness) => {
                loop {
                    iteration_counter += 1;
                    pool.scope(|scope|
                        for population in &mut self.habitat {
                            scope.submit(move || { population.run_body() });
                        });

                    self.update_results();

                    if self.simulation_result.fittest[0].fitness <= end_fitness {
                        break;
                    }
                };
                self.simulation_result.iteration_counter = iteration_counter;
            }
        } // End of match

        let elapsed = start_time.elapsed();

        self.total_time_in_ms = elapsed.as_secs() as f64 * 1000.0 + elapsed.subsec_nanos() as f64 / 1000_000.0;
    }

    /// This is a helper function that the user can call after the simulation stops in order to
    /// see all the fitness values for all the individuals that participated to the overall
    /// improvement.
    pub fn print_fitness(&self) {
        for wrapper in &self.simulation_result.fittest {
            info!("fitness: {}, num_of_mutations: {}, population: {}",
                     wrapper.fitness, wrapper.num_of_mutations, wrapper.id);
        }
    }

    /// Update the internal state of the simulation: Has a new fittest individual been found ?
    /// Do we want to share it across all the other populations ?
    /// Also calculates the improvement factor.
    fn update_results(&mut self) {
        // Determine the fittest individual of all populations.
        let mut new_fittest_found = false;

        // Increment the output counter
        // Only write an output if the max value output_every is reached
        self.output_every_counter += 1;

        for population in &mut self.habitat {
            if population.population[0].fitness < self.simulation_result.fittest[0].fitness {
                new_fittest_found = true;
                self.simulation_result.fittest.insert(0, population.population[0].clone());
                // See https://github.com/willi-kappler/darwin-rs/issues/12
                self.simulation_result.fittest.truncate(self.num_of_global_fittest);
                population.fitness_counter += 1;
                if self.output_every_counter >= self.output_every {
                    info!("new fittest: fitness: {}, population id: {}, counter: {}", population.population[0].fitness,population.id,
                        population.fitness_counter);
                    self.output_every_counter = 0
                }
                // Call methond `new_fittest_found` of the newly found fittest individual.
                // The default implementation for this method does nothing.
                population.population[0].individual.new_fittest_found();
            }
        }

        // Now copy the most fittest individual back to each population
        // if the user has specified it and the share_every count is reached
        self.share_counter += 1;
        if self.share_fittest && new_fittest_found && (self.share_counter >= self.share_every) {
            for population in &mut self.habitat {
                population.population[0] = self.simulation_result.fittest[0].clone();
            }
            self.share_counter = 0;
        }

        self.simulation_result.improvement_factor =
            self.simulation_result.fittest[0].fitness /
            self.simulation_result.original_fitness;

    }
}
