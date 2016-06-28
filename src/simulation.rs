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

use jobsteal::{make_pool, Pool, IntoSplitIterator, SplitIterator};

use individual::{Individual, IndividualWrapper};

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
    /// The number of individuals for the whole simulation.
    pub num_of_individuals: u32,
    /// The number of threads to use to speed up calculation.
    pub num_of_threads: usize,
    /// The current improvement factor, that means the ration between the first and the current
    /// fitness.
    pub improvement_factor: f64,
    /// The very first calculated fitness, when the simulation just started.
    pub original_fitness: f64,
    /// The current fittest individual. This will change during the simulation as soon as a new
    /// more fittest individual is found
    pub fittest: IndividualWrapper<T>,
    /// The population for the simulation. Contains all individuals for the simulation.
    pub population: Vec<IndividualWrapper<T>>,
    /// The total run time for the simulation. This will be calculated once the stimulation has
    /// finished.
    pub total_time_in_ms: f64,
    /// The number of current iteration. This changes with every iteration and is used by the
    /// `EndIteration` enum.
    pub iteration_counter: u32,
    /// The amount of iteration to wait until all individuals will be resetted.
    pub reset_limit: u32,
    /// The start value of the reset limit
    pub reset_limit_start: u32,
    /// The end value of the reset limit, if reset_limit >= reset_limit_end, then the reset_limit
    /// will be resettet to the start value reset_limit_start.
    /// If reset_limit_end == 0, this feature will be disabled.
    pub reset_limit_end: u32,
    /// The reset counter, if reset_counter >= reset_limit, all the individuals are discarded and
    /// the simulation restarts anew with an increased reset_limit. This prevents local minima,
    /// but also discards the current fittest individual.
    pub reset_counter: u32,
    /// A flag that specifies if the sumulation should write a message every time a new most
    /// fittest individual is found.
    pub output_new_fittest: bool,
    /// The thread pool used by the `jobsteal` crate
    pub pool: Pool,
}

/// Mutates the population and calculates the new fitness for each individual.
/// Every individual has its own mutation counter `num_of_mutations`.
/// This counter specifies how often an individual should be mutated during one iteration.
/// The function iterates through all the individual in the population of the simulation and
/// spawns threads to do the mutation and calculation in parallel using the `jobsteal` crate.
fn mutate_population<T: Individual + Send + Sync>(simulation: &mut Simulation<T>) {
    (&mut simulation.population).into_split_iter().for_each(&simulation.pool.spawner(),
                                                            |wrapper| {
        for _ in 0..wrapper.num_of_mutations {
            wrapper.individual.mutate();
        }
        wrapper.fitness = wrapper.individual.calculate_fitness();
    });
}

/// This is the body that gets called for every iteration.
/// This function does the following:
/// 1. Clone the current population.
/// 2. Mutate the current population using the `mutate_population` function.
/// 3. Merge the newly mutated population and the original cloned population into one big
/// population twice the size.
/// 4. Sort this new big population by fitness. So the fittest individual is at position 0.
/// 5. Truncated the big population to its original size and thus gets rid of all the less fittest
/// individuals (they "die").
/// 6. Check if the fittest individual (at index 0) in the current sorted population is better
/// (= fitter) than the global
/// fittest individual of the whole simulation. If yes, the global fittest individual is
/// replaced.
/// 7. Calculate the new improvement factor and prepare for the next iteration.
fn run_body_sorting_fittest<T: Individual + Send + Sync + Clone>(simulation: &mut Simulation<T>) {
    // Keep original population
    let orig_population = simulation.population.clone();
    let orig_population_len = orig_population.len();

    // Mutate population
    mutate_population(simulation);

    // Append original (unmutated) population to new (mutated) population
    simulation.population.extend(orig_population.iter().cloned());

    // Sort by fitness
    simulation.population.sort();

    // Reduce population to original length
    simulation.population.truncate(orig_population_len);

    // Restore original number of mutation rate, since these will be lost because of sorting
    for (individual, orig_individual) in simulation.population
        .iter_mut()
        .zip(orig_population.iter()) {
        individual.num_of_mutations = orig_individual.num_of_mutations;
    }

    // Check if we have new fittest individual and store it globally
    if simulation.population[0].fitness < simulation.fittest.fitness {
        simulation.fittest = simulation.population[0].clone();
        if simulation.output_new_fittest {
            println!("{}: new fittest: {}",
                     simulation.iteration_counter,
                     simulation.fittest.fitness);
        }
    }

    simulation.improvement_factor = simulation.fittest.fitness / simulation.original_fitness;
}

/// This implements the two functions `run` and `print_fitness` for the struct `Simulation`.
impl<T: Individual + Send + Sync + Clone> Simulation<T> {
    /// This actually runs the simulation.
    /// Depending on the type of simulation (EndIteration, EndFactor or Endfitness) the iteration
    /// loop will check for The
    /// stop condition accordingly.
    pub fn run(&mut self) {
        // Initialize timer
        let start_time = Instant::now();

        // Calculate the fitness for all individuals at the beginning.
        for wrapper in &mut self.population {
            wrapper.fitness = wrapper.individual.calculate_fitness();
        }

        // Select one global fittest individual.
        self.original_fitness = self.population[0].fitness;

        println!("original_fitness: {}", self.original_fitness);

        self.iteration_counter = 0;
        self.reset_limit = self.reset_limit_start;
        self.pool = make_pool(self.num_of_threads).unwrap();

        // Check which type of simulation to run.
        match self.type_of_simulation {
            SimulationType::EndIteration(end_iteration) => {
                for i in 0..end_iteration {
                    run_body_sorting_fittest(self);
                    self.iteration_counter = i;
                    self.check_iteration_limit();
                }
            }
            SimulationType::EndFactor(end_factor) => {
                loop {
                    if self.improvement_factor <= end_factor {
                        break;
                    }
                    run_body_sorting_fittest(self);
                    self.iteration_counter += 1;
                    self.check_iteration_limit();
                }
            }
            SimulationType::EndFitness(end_fitness) => {
                loop {
                    if self.fittest.fitness <= end_fitness {
                        break;
                    }
                    run_body_sorting_fittest(self);
                    self.iteration_counter += 1;
                    self.check_iteration_limit();
                }
            }
        }

        let elapsed = start_time.elapsed();

        self.total_time_in_ms = elapsed.as_secs() as f64 * 1000.0 + elapsed.subsec_nanos() as f64 / 1000_000.0;
    }

    /// This is a helper function that the user can call after the simulation stops in order to
    /// see all the fitness values for all the individuals.
    pub fn print_fitness(&self) {
        for wrapper in &self.population {
            println!("fitness: {}, num_of_mutations: {}",
                     wrapper.fitness,
                     wrapper.num_of_mutations);
        }
    }

    /// This function checks if the current iteration counter is above a certain threshold value.
    /// If yes, all the individuals will be replaced by new initial ones.
    /// Thus local minima are avoided. The limit threshold is increased every time after that.
    fn check_iteration_limit(&mut self) {
        if self.reset_limit_end > 0 {
            self.reset_counter += 1;

            if self.reset_counter > self.reset_limit {
                self.reset_limit += 1000;
                if self.reset_limit >= self.reset_limit_end {
                    self.reset_limit = self.reset_limit_start;
                }
                self.reset_counter = 0;
                println!("new reset_limit: {}", self.reset_limit);

                // Kill all individuals since we are most likely stuck in a local minimum.
                // Why is it so ? Because the simulation is still running!
                for population in &mut self.population {
                    population.individual = Individual::new();
                    population.fitness = population.individual.calculate_fitness();
                }
            }
        }
    }
}
