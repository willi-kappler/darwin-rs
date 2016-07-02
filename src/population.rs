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

use std::sync::{Arc, Mutex};

use simulation::SimulationResult;
use individual::{Individual, IndividualWrapper};

// pub fn new_population() -> Population {
//
// }

/// The Population type. TODO
#[derive(Clone)]
pub struct Population<T: Individual + Send + Sync> {
    /// The number of individuals for this population.
    pub num_of_individuals: u32,
    /// The actual population (vector of individuals).
    pub population: Vec<IndividualWrapper<T>>,
    /// The amount of iteration to wait until all individuals will be resetted.
    pub reset_limit: u32,
    /// The start value of the reset limit
    pub reset_limit_start: u32,
    /// The end value of the reset limit, if reset_limit >= reset_limit_end, then the reset_limit
    /// will be resettet to the start value reset_limit_start.
    /// If reset_limit_end == 0, this feature will be disabled.
    pub reset_limit_end: u32,
    /// TODO
    pub reset_limit_increment: u32,
    /// The reset counter, if reset_counter >= reset_limit, all the individuals are discarded and
    /// the simulation restarts anew with an increased reset_limit. This prevents local minima,
    /// but also discards the current fittest individual.
    pub reset_counter: u32,
}

impl<T: Individual + Send + Sync + Clone> Population<T> {
    /// Just calculates the fitness for each individual.
    pub fn calculate_fitness(&mut self) {
        for wrapper in self.population.iter_mut() {
            wrapper.fitness = wrapper.individual.calculate_fitness();
        }
    }

    /// This is the body that gets called for every iteration.
    /// This function does the following:
    /// 1. Check if the reset limit is reached. If it is, this whole population is
    /// discarded and re-initialized from the start. All the information about the
    /// current fittest individual is lost. This is done to avoid local minima.
    /// 2. Clone the current population.
    /// 3. Mutate the current population using the `mutate_population` function.
    /// 4. Merge the newly mutated population and the original cloned population into one big
    /// population twice the size.
    /// 5. Sort this new big population by fitness. So the fittest individual is at position 0.
    /// 6. Truncated the big population to its original size and thus gets rid of all the less fittest
    /// individuals (they "die").
    /// 7. Check if the fittest individual (at index 0) in the current sorted population is better
    /// (= fitter) than the global
    /// fittest individual of the whole simulation. If yes, the global fittest individual is
    /// replaced.
    /// 8. Calculate the new improvement factor and prepare for the next iteration.
    pub fn run_body(&mut self, simulation_result: &Arc<Mutex<&mut SimulationResult<T>>>,
            iteration_counter: u32) {
        // First check if reset limit is reached
        if self.reset_limit_end > 0 {
            self.reset_counter += 1;

            if self.reset_counter > self.reset_limit {
                self.reset_limit += self.reset_limit_increment;
                if self.reset_limit >= self.reset_limit_end {
                    self.reset_limit = self.reset_limit_start;
                    println!("reset_limit reset to reset_limit_start");
                }
                self.reset_counter = 0;
                println!("new reset_limit: {}", self.reset_limit);

                // Kill all individuals since we are most likely stuck in a local minimum.
                // Why is it so ? Because the simulation is still running!
                // Keep number of mutations.
                for wrapper in &mut self.population {
                    wrapper.individual = Individual::new();
                    wrapper.fitness = wrapper.individual.calculate_fitness();
                }
            }
        }

        // Keep original population
        let orig_population = self.population.clone();

        // Mutate population
        for wrapper in self.population.iter_mut() {
            for _ in 0..wrapper.num_of_mutations {
                wrapper.individual.mutate();
            }
            wrapper.fitness = wrapper.individual.calculate_fitness();
        }

        // Append original (unmutated) population to new (mutated) population
        self.population.extend(orig_population.iter().cloned());

        // Sort by fitness
        self.population.sort();

        // Reduce population to original length
        self.population.truncate(self.num_of_individuals as usize);

        // Restore original number of mutation rate, since these will be lost because of sorting
        for (individual, orig_individual) in self.population
            .iter_mut()
            .zip(orig_population.iter()) {
            individual.num_of_mutations = orig_individual.num_of_mutations;
        }

        match simulation_result.lock() {
            Ok(mut simulation_result) => {
                // Check if we have new fittest individual and store it globally
                if self.population[0].fitness < simulation_result.fittest[0].fitness {
                    // Insert it to the first position (at index 0) so that the order of fitness
                    // is preserved (fittest at index 0, then decreasing fitness).
                    simulation_result.fittest.insert(0, self.population[0].clone());
                    println!("{}: new fittest: {}",
                             iteration_counter, simulation_result.fittest[0].fitness);
                }

                simulation_result.improvement_factor = simulation_result.fittest[0].fitness / simulation_result.original_fitness;
            },
            Err(e) => println!("Mutex (poison) error (simulation_result): {}", e)
        }
        // No need to unlock simulation_result, since it goes out of scope and then
        // drop() (= destructor) is called.
    }
}
