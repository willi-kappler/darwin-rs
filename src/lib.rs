//! darwin-rs: evolutionary algorithms with Rust
//!
//! Written by Willi Kappler, Version 0.1 (2016.06.05)
//!
//! Repository: https://github.com/willi-kappler/darwin-rs
//!
//! License: MIT
//!
//! This library allows you to write evolutionary algorithms (EA) in Rust. Examples provieded: TSP, Sudoku, Queens Problem
//!
//!


extern crate time;
extern crate jobsteal;

// external modules
use time::precise_time_ns;
use jobsteal::{make_pool, Pool, IntoSplitIterator, SplitIterator};
use std::cmp::Ordering;

/// The `SimulationType` type. Speficies the criteria on how a simulation should stop.
#[derive(Debug,Clone)]
pub enum SimulationType {
    /// Finish the simulation when a number of iteration has been reached.
    EndIteration(u32),
    /// Finish the simulation when a specific fittness is rached.
    /// That means if at least one of the individuals has this fittness.
    /// The fittness is calculated using the implemented `calculate_fittness` functions
    /// of the `Individual` trait
    EndFittness(f64),
    /// Finish the simulation when a specific improvement factor is reached.
    /// That means the relation between the very first fittness and the current fittness of the fittest individual
    EndFactor(f64)
}

/// The `Simulation` type. Contains all the information / configuration for the simulation to run.
pub struct Simulation<T: Individual + Send + Sync> {
    /// How should the simulation stop ?
    pub type_of_simulation: SimulationType,
    /// The number of individuals for the whole simulation.
    pub num_of_individuals: u32,
    /// The number of threads to use to speed up calculation.
    pub num_of_threads: usize,
    /// The current improvement factor, that means the ration between the first and the current fittness.
    pub improvement_factor: f64,
    /// The very first calculated fittness, when the simulation just started.
    pub original_fittness: f64,
    /// The current fittest individual. This will change during the simulation as soon as a new more fittest individual is found
    pub fittest: IndividualWrapper<T>,
    /// The population for the simulation. Contains all individuals for the simulation.
    pub population: Vec<IndividualWrapper<T>>,
    /// The total run time for the simulation. This will be calculated once the stimulation has finished.
    pub total_time_in_ms: f64,
    /// The number of current iteration. This changes with every iteration and is used by the `EndIteration` enum.
    pub iteration_counter: u32,
    /// A flag that specifies if the sumulation should write a message every time a new most fittest individual is found.
    pub output_new_fittest: bool,
    /// The thread pool used by the `jobsteal` crate
    pub pool: Pool,
}

/// Mutates the population and calculates the new fittness for each individual.
/// Every individual has its own mutation counter `num_of_mutations`.
/// This counter specifies how often an individual should be mutated during one iteration.
/// The function iterates through all the individual in the population of the simulation and
/// spawns threads to do the mutation and calculation in parallel using the `jobsteal` crate.
fn mutate_population<T: Individual + Send + Sync>(simulation: &mut Simulation<T>) {
    (&mut simulation.population).into_split_iter().for_each(
        &simulation.pool.spawner(), |wrapper|
        {
            for _ in 0..wrapper.num_of_mutations {
                wrapper.individual.mutate();
            }
            wrapper.fittness = wrapper.individual.calculate_fittness();
        }
    );
}

/// This is the body that gets called for every iteration.
/// This function does the following:
/// 1. Clone the current population.
/// 2. Mutate the current population using the `mutate_population` function.
/// 3. Merge the newly mutated population and the original cloned population into one big population twice the size.
/// 4. Sort this new big population by fittness. So the fittest individual is at position 0.
/// 5. Truncated the big population to its original size and thus gets rid of all the less fittest individuals (they "die").
/// 6. Check if the fittest individual (at index 0) in the current sorted population is better (= fitter) than the global
///    fittest individual of the whole simulation. If yes, the global fittest individual is replaced.
/// 7. Calculate the new improvement factor and prepare for the next iteration.
fn run_body_sorting_fittest<T: Individual + Send + Sync + Clone>(simulation: &mut Simulation<T>) {
    // Keep original population
    let orig_population = simulation.population.clone();

    // Mutate population
    mutate_population(simulation);

    // Append original (unmutated) population to new (mutated) population
    simulation.population.extend(orig_population.iter().cloned());

    // Sort by fittness
    simulation.population.sort();

    // Copy last individual (= unfittest) in order to avoid local minimum
    simulation.population[orig_population.len() - 1] = simulation.population[simulation.population.len() - 1].clone();

    // Reduce population to original length
    simulation.population.truncate(orig_population.len());

    // Restore original number of mutation rate, since these get overwritten by .clone()
    for i in 0..orig_population.len() {
        simulation.population[i].num_of_mutations = orig_population[i].num_of_mutations;
    }

    // Check if we have new fittest individual and store it globally
    if simulation.population[0].fittness < simulation.fittest.fittness {
        simulation.fittest = simulation.population[0].clone();
        if simulation.output_new_fittest {
            println!("{}: new fittest: {}", simulation.iteration_counter, simulation.fittest.fittness);
        }
    }

    simulation.improvement_factor = simulation.fittest.fittness / simulation.original_fittness;
}

/// This implements the two functions `run` and `print_fittness` for the struct `Simulation`.
impl<T: Individual + Send + Sync + Clone> Simulation<T> {
    /// This actually runs the simulation.
    /// Depending on the type of simulation (EndIteration, EndFactor or EndFittness) the iteration loop will check for The
    /// stop condition accordingly.
    pub fn run(&mut self) {
        // Initialize timer
        let start_time = precise_time_ns();

        // Calculate the fittness for all individuals at the beginning.
        for wrapper in self.population.iter_mut() {
            wrapper.fittness = wrapper.individual.calculate_fittness();
        }

        // Select one global fittest individual.
        self.original_fittness = self.population[0].fittness;

        println!("original_fittness: {}", self.original_fittness);

        self.iteration_counter = 0;
        self.pool = make_pool(self.num_of_threads).unwrap();

        // Check which type of simulation to run.
        match self.type_of_simulation {
            SimulationType::EndIteration(end_iteration) => {
                for i in 0..end_iteration {
                    run_body_sorting_fittest(self);
                    self.iteration_counter = i
                }
            },
            SimulationType::EndFactor(end_factor) => {
                loop {
                    if self.improvement_factor <= end_factor { break }
                    run_body_sorting_fittest (self);
                    self.iteration_counter = self.iteration_counter + 1;
                }
            },
            SimulationType::EndFittness(end_fittness) => {
                loop {
                    if self.fittest.fittness <= end_fittness { break }
                    run_body_sorting_fittest(self);
                    self.iteration_counter = self.iteration_counter + 1;
                }
            }
        }

        let end_time = precise_time_ns();

        self.total_time_in_ms = ((end_time - start_time) as f64) / (1000.0 * 1000.0);
    }

    /// This is a helper function that the user can call after the simulation stops in order to see
    /// all the fittness values for all the individuals.
    pub fn print_fittness(&self) {
        for wrapper in self.population.iter() {
            println!("fittness: {}, num_of_mutations: {}", wrapper.fittness, wrapper.num_of_mutations);
        }
    }
}

/// A wrapper helper struct for the individuals.
/// It does the book keeping of the fittness and the number of mutations this individual
/// has to run in one iteration.
#[derive(Debug,Clone)]
pub struct IndividualWrapper<T: Individual> {
    /// The actual individual, user defined struct.
    pub individual: T,
    /// the current calculated fittness for this individual.
    fittness: f64,
    /// The number of mutation this individual is doing in one iteration.
    num_of_mutations: u32
}

/// Implement this for sorting
impl<T: Individual> PartialEq for IndividualWrapper<T> {
    fn eq(&self, other: &IndividualWrapper<T>) -> bool {
        self.fittness == other.fittness
    }
}

/// Implement this for sorting
impl<T: Individual> Eq for IndividualWrapper<T>{}

/// Implement this for sorting
impl<T: Individual> Ord for IndividualWrapper<T> {
    fn cmp(&self, other: &IndividualWrapper<T>) -> Ordering {
        if self.fittness < other.fittness { Ordering::Less }
        else if self.fittness > other.fittness { Ordering::Greater }
        else { Ordering::Equal }
    }
}

/// Implement this for sorting
impl<T: Individual> PartialOrd for IndividualWrapper<T> {
    fn partial_cmp(&self, other: &IndividualWrapper<T>) -> Option<Ordering> {
        if self.fittness < other.fittness { Some(Ordering::Less) }
        else if self.fittness > other.fittness { Some(Ordering::Greater) }
        else { Some(Ordering::Equal) }
    }
}

/// This trait has to be implemented for the user defined struct.
pub trait Individual {
    /// This method creates a new individual.
    fn new() -> Self;
    /// This method mutates the individual. Usually this is a cheap and easy to implement function.
    /// In order to improve the simulation, the user can make this function a bit smarter.
    fn mutate(&mut self);
    /// This method calculates the fittness for the individual. Usually this is an expensive operation Append
    /// a bite mor difficult to implement, compared to the mutation method above.
    /// The lower the fittness value, the better (healthier) the individual is and the closer the individual is to the
    /// perfect solution. This can also correspont to the number of errors for example in the sudoku or queens problem case.
    fn calculate_fittness(&self) -> f64;
}

/// This is a helper struct in order to build (configure) a valid simulation.
/// See builder pattern: https://en.wikipedia.org/wiki/Builder_pattern
pub struct SimulationBuilder<T: Individual + Send + Sync> {
    /// The actual simulation
    simulation: Simulation<T>
}

/// This enum describes the possible return error that the simulation builder can return.
pub enum BuilderResult<T: Individual + Send + Sync> {
    /// The number of iteration is too low, should be >= 10
    TooLowEndIterration,
    /// The number of individuals is too loe, should be >= 3
    TooLowIndividuals,
    /// Everything is fine, the simulation is properly configured and ready to run.
    Ok(Simulation<T>)
}

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
                original_fittness: std::f64::MAX,
                fittest: IndividualWrapper {
                    individual: Individual::new(),
                    fittness: std::f64::MAX,
                    num_of_mutations: 1
                },
                population: Vec::new(),
                total_time_in_ms: 0.0,
                iteration_counter: 0,
                output_new_fittest: true,
                pool: make_pool(4).unwrap(),
            }
        }
    }

    /// Set the total number of iterations for the simulation and thus sets the simulation type to `EndIteration`.
    /// (Only usefull in combination with `EndIteration`).
    pub fn iterations(mut self, iterations: u32) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndIteration(iterations);
        self
    }

    /// Set the improvement factor stop criteria for the simulation and thus sets the simulation type to `EndFactor`.
    /// (Only usefull in combination with `EndFactor`).
    pub fn factor(mut self, factor: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFactor(factor);
        self
    }

    /// Set the minimum fittness stop criteria for the simulation and thus sets the simulation type to `EndFittness`.
    /// (Only usefull in combination with `EndFactor`).
    pub fn fittness(mut self, fittness: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFittness(fittness);
        self
    }

    /// Sets the number of individuals and creates the population, must be >= 3
    pub fn individuals(mut self, individuals: u32) -> SimulationBuilder<T> {
        self.simulation.num_of_individuals = individuals;

        for _ in 0..individuals {
            self.simulation.population.push(
                IndividualWrapper {
                    individual: Individual::new(),
                    fittness: std::f64::MAX,
                    num_of_mutations: 1
                }
            );
        }

        self
    }

    /// Sets the number of threads in order to speed up the simulation.
    pub fn threads(mut self, threads: usize) -> SimulationBuilder<T> {
        self.simulation.num_of_threads = threads;
        self
    }

    /// Sets a flag if the simulation should write a message whenever a new fittest individual is found.
    pub fn output_new_fittest(mut self, output_new_fittest: bool) -> SimulationBuilder<T> {
        self.simulation.output_new_fittest = output_new_fittest;
        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the sumulation:
    /// The first individual will mutate once, the second will mutate twice, the nth individual will Mutate
    /// n-times per iteration.
    pub fn increasing_mutation_rate(mut self) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in self.simulation.population.iter_mut() {
            wrapper.num_of_mutations = mutation_rate;
            mutation_rate = mutation_rate + 1;
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the sumulation:
    /// Instead of a linear growing mutation rate like in the `increasing_mutation_rate` function above
    /// this sets an exponention mutation rate for all the individuals.
    /// The first individual will mutate base^1 times, the second will mutate base^2 times, and nth
    /// will mutate base^n times per iteration.
    pub fn increasing_exp_mutation_rate(mut self, base: f64) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in self.simulation.population.iter_mut() {
            wrapper.num_of_mutations = base.powi(mutation_rate).floor() as u32;
            mutation_rate = mutation_rate + 1;
        }

        self
    }

    /// Configures the mutation rates (number of mutation runs) for all the individuals in the sumulation:
    /// This allows to specify an arbitrary mutation scheme for each individual.
    /// The number of rates must be equal to the number of individuals.
    pub fn mutation_rate(mut self, mutation_rate: Vec<u32>) -> SimulationBuilder<T> {
        // TODO: better error handling
        assert!(self.simulation.population.len() == mutation_rate.len());

        for i in 0..self.simulation.population.len() {
            self.simulation.population[i].num_of_mutations = mutation_rate[i];
        }

        self
    }

    /// This checks the configuration of the simulation and returns an error or Ok if no errors where found.
    pub fn finalize(self) -> BuilderResult<T> {
        let result = Simulation {
            type_of_simulation: self.simulation.type_of_simulation.clone(),
            num_of_individuals: self.simulation.num_of_individuals,
            num_of_threads: self.simulation.num_of_threads,
            improvement_factor: self.simulation.improvement_factor,
            original_fittness: self.simulation.original_fittness,
            fittest: self.simulation.fittest,
            population: self.simulation.population,
            total_time_in_ms: self.simulation.total_time_in_ms,
            iteration_counter: self.simulation.iteration_counter,
            output_new_fittest: self.simulation.output_new_fittest,
            pool: self.simulation.pool,
        };

        if self.simulation.num_of_individuals < 3 { return BuilderResult::TooLowIndividuals }

        if let SimulationType::EndIteration(end_iteration) = self.simulation.type_of_simulation {
            if end_iteration < 10 { return BuilderResult::TooLowEndIterration }
        }

        BuilderResult::Ok(result)
    }
}
