//! darwin-rs: evolutionary algorithms with Rust
//!
//! Written by Willi Kappler, Version 0.1 (2016.06.11)
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

extern crate jobsteal;
#[macro_use]
extern crate quick_error;

// external modules
use jobsteal::{make_pool, Pool, IntoSplitIterator, SplitIterator};
use std::cmp::Ordering;
use std::time::Instant;

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
        if self.reset_limit > 0 {
            self.reset_counter += 1;

            if self.reset_counter > self.reset_limit {
                self.reset_limit += 1000;
                self.reset_counter = 0;
                println!("new reset_limit: {}", self.reset_limit);

                // Kill all individuals since we are stuck in a local minimum.
                // Why is it so ? Because the simulation is still running!
                for population in &mut self.population {
                    population.individual = Individual::new();
                    population.fitness = population.individual.calculate_fitness();
                }
            }
        }
    }
}

/// A wrapper helper struct for the individuals.
/// It does the book keeping of the fitness and the number of mutations this individual
/// has to run in one iteration.
#[derive(Debug,Clone)]
pub struct IndividualWrapper<T: Individual> {
    /// The actual individual, user defined struct.
    pub individual: T,
    /// the current calculated fitness for this individual.
    fitness: f64,
    /// The number of mutation this individual is doing in one iteration.
    num_of_mutations: u32,
}

/// Implement this for sorting
impl<T: Individual> PartialEq for IndividualWrapper<T> {
    fn eq(&self, other: &IndividualWrapper<T>) -> bool {
        self.fitness == other.fitness
    }
}

/// Implement this for sorting
impl<T: Individual> Eq for IndividualWrapper<T> {}

/// Implement this for sorting
impl<T: Individual> Ord for IndividualWrapper<T> {
    fn cmp(&self, other: &IndividualWrapper<T>) -> Ordering {
        self.partial_cmp(other).expect("Fitness of Individual is NaN")
    }
}

/// Implement this for sorting
impl<T: Individual> PartialOrd for IndividualWrapper<T> {
    fn partial_cmp(&self, other: &IndividualWrapper<T>) -> Option<Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}

/// This trait has to be implemented for the user defined struct.
pub trait Individual {
    /// This method creates a new individual.
    fn new() -> Self;
    /// This method mutates the individual. Usually this is a cheap and easy to implement
    /// function. In order to improve the simulation, the user can make this function a bit
    /// "smarter". This is nicely shown in the tsp and tsp2 example. The tsp2 example contains
    /// two types of mutation, tsp just one:
    ///
    /// examples/tsp: 1. swap position
    ///
    /// examples/tsp2: 1. swap position, 2. rotate (shift) positions
    ///
    /// By just adding this one additional mutation type the simulation converges much faster
    /// to the optimum. Of course rotation can be "simulated" by a number of swaps, but re-doing
    /// all these steps takes time and the chances that these steps are taken in the correct
    /// order by just randomly swaping positions are very slim. So just start with one simple
    /// mutation function (one operation) and add more and more "smarter" mutation types to the
    /// mutate function.
    fn mutate(&mut self);
    /// This method calculates the fitness for the individual. Usually this is an expensive
    /// operation and a bit more difficult to implement, compared to the mutation method above.
    /// The lower the fitness value, the better (healthier) the individual is and the closer
    /// the individual is to the perfect solution. This can also correspont to the number of
    /// errors like for example in the sudoku or queens problem case.
    fn calculate_fitness(&self) -> f64;
}

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

    /// Configures the reset limit for the simulation. If the limit is greater than zero then a
    /// reset counter is increased each iteration. If that counter is greater than the limit,
    /// all individuals will be resetted, the limit will be increased by 1000 and the counter is
    /// set back to zero. Default value is 1000.
    pub fn reset_limit(mut self, reset_limit: u32) -> SimulationBuilder<T> {
        self.simulation.reset_limit = reset_limit;

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
