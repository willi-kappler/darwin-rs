//! This module defines structure and methods for a population that is needed by a smulation.
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

use individual::{Individual, IndividualWrapper};

/// The `Population` type. Contains the actual individuals (through a wrapper) and informations
/// like the `reset_limit`. Use the `PopulationBuilder` in your main program to create populations.
#[derive(Clone)]
pub struct Population<T: Individual> {
    /// The number of individuals for this population.
    pub num_of_individuals: u32,
    /// The actual population (vector of individuals).
    pub population: Vec<IndividualWrapper<T>>,
    /// The amount of iteration to wait until all individuals will be resetted.
    /// This calls the `reset` method for each individual.
    pub reset_limit: u32,
    /// The start value of the reset limit.
    pub reset_limit_start: u32,
    /// The end value of the reset limit, if `reset_limit` >= `reset_limit_end`, then the `reset_limit`
    /// will be resettet to the start value `reset_limit_start`.
    /// If `reset_limit_end` == 0, this feature will be disabled.
    pub reset_limit_end: u32,
    /// The increment for the `reset_limit`. After the reset_limit value is reached, it will be
    /// increased by the value of `reset_limit_increment`.
    pub reset_limit_increment: u32,
    /// The reset counter, if `reset_counter` >= `reset_limit`, all the individuals are discarded and
    /// the simulation restarts anew with an increased `reset_limit`. This prevents local minima,
    /// but also discards the current fittest individual.
    pub reset_counter: u32,
    /// The ID of the population, only used for statistics. For example: which population does
    /// have the most fittest individuals ? This may help you to set the correct parameters for
    /// your simulations.
    pub id: u32,
    /// Count how often this population has created (found) the fittest individual. This may help
    /// you to fine tune the parameters for the population and the simulation in general.
    pub fitness_counter: u64
}

impl<T: Individual + Send + Sync + Clone> Population<T> {
    /// Just calculates the fitness for each individual.
    /// Usually this is the most computational expensive operation, so optimize the
    /// `calculate_fitness` method of your data structure ;-)
    pub fn calculate_fitness(&mut self) {
        for wrapper in &mut self.population {
            wrapper.fitness = wrapper.individual.calculate_fitness();
        }
    }

    /// This is the body that gets called for every iteration.
    /// This function does the following:
    ///
    /// 1. Check if the reset limit is reached. If it is, this whole population is
    /// discarded and re-initialized from the start. All the information about the
    /// current fittest individual is lost. This is done to avoid local minima.
    ///
    /// 2. Clone the current population.
    ///
    /// 3. Mutate the current population using the `mutate_population` function.
    ///
    /// 4. Merge the newly mutated population and the original cloned population into one big
    /// population twice the size.
    ///
    /// 5. Sort this new big population by fitness. So the fittest individual is at position 0.
    ///
    /// 6. Truncated the big population to its original size and thus gets rid of all the less fittest
    /// individuals (they "die").
    ///
    /// 7. Check if the fittest individual (at index 0) in the current sorted population is better
    /// (= fitter) than the global fittest individual of the whole simulation. If yes, the global
    /// fittest individual is replaced.
    ///
    /// 8. Calculate the new improvement factor and prepare for the next iteration.
    pub fn run_body(&mut self) {
        // Is reset limit enabled ?
        if self.reset_limit_end > 0 {
            self.reset_counter += 1;

            // Check if reset limit is reached
            if self.reset_counter > self.reset_limit {
                self.reset_limit += self.reset_limit_increment;
                if self.reset_limit >= self.reset_limit_end {
                    self.reset_limit = self.reset_limit_start;
                    info!("reset_limit reset to reset_limit_start: {}, id: {}", self.reset_limit_start, self.id);
                }
                self.reset_counter = 0;
                info!("new reset_limit: {}, id: {}, counter: {}", self.reset_limit, self.id, self.fitness_counter);

                // Kill all individuals since we are most likely stuck in a local minimum.
                // Why is it so ? Because the simulation is still running and the exit criteria
                // hasn't been reached yet!
                // Keep number of mutations.
                for wrapper in &mut self.population {
                    wrapper.individual.reset();
                    wrapper.fitness = wrapper.individual.calculate_fitness();
                }
            }
        }

        // Keep original population.
        let orig_population = self.population.clone();

        // Mutate population
        for wrapper in &mut self.population {
            for _ in 0..wrapper.num_of_mutations {
                // Maybe add super optimization ?
                // See https://github.com/willi-kappler/darwin-rs/issues/10
                wrapper.individual.mutate();
            }
            wrapper.fitness = wrapper.individual.calculate_fitness();
        }

        // Append original (unmutated) population to new (mutated) population.
        self.population.extend(orig_population.iter().cloned());

        // Sort by fitness
        // Use random choice, see https://github.com/willi-kappler/darwin-rs/issues/7
        self.population.sort();

        // Reduce population to original length.
        self.population.truncate(self.num_of_individuals as usize);

        // Restore original number of mutation rate, since these will be lost because of sorting.
        for (individual, orig_individual) in self.population
            .iter_mut()
            .zip(orig_population.iter()) {
            individual.num_of_mutations = orig_individual.num_of_mutations;
        }
    }
}
