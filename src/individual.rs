//! This module defines the trait and the data structure needed for specifying each individual in a population.
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

// external modules
use std::cmp::Ordering;

/// A wrapper helper struct for the individuals.
/// It does the book keeping of the fitness and the number of mutations this individual
/// has to run in one iteration.
#[derive(Debug,Clone)]
pub struct IndividualWrapper<T: Individual> {
    /// The actual individual, user defined struct.
    pub individual: T,
    /// The current calculated fitness for this individual.
    pub fitness: f64,
    /// The number of mutation this individual is doing in one iteration.
    pub num_of_mutations: u32,
    /// The id of the population that this individual belongs to. Just for statistics.
    pub id: u32,
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
/// In order to share common data between all individuals use Arc. See TSP and OCR exmaples.
///
/// TODO: add serialization, see https://github.com/willi-kappler/darwin-rs/issues/11
pub trait Individual {
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
    fn calculate_fitness(&mut self) -> f64;
    /// This method resets each individual to an initial state.
    /// For example in the "queens" case it would reset the queens position randomly
    /// (or all in the first row).
    fn reset(&mut self);
    /// This method is called whenever a new fittest individual is found. It is usefull when you
    /// want to provide some additional information or do some statistics.
    /// It is optional and the default implementation does nothing.
    fn new_fittest_found(&mut self) {

    }
}

#[cfg(test)]
mod test {
    use super::{IndividualWrapper, Individual};

    struct IndividualTest1;

    impl Individual for IndividualTest1 {
        fn mutate(&mut self) {
        }

        fn calculate_fitness(&mut self) -> f64 {
            0.0
        }

        fn reset(&mut self) {

        }
    }

    #[test]
    fn compare1() {
        let individual1 = IndividualWrapper{individual: IndividualTest1, fitness: 1.2, num_of_mutations: 21, id: 1};
        let individual2 = IndividualWrapper{individual: IndividualTest1, fitness: 5.93, num_of_mutations: 7, id: 1};

        assert!(individual2 > individual1);
    }

    #[test]
    fn compare2() {
        let individual1 = IndividualWrapper{individual: IndividualTest1, fitness: 3.78, num_of_mutations: 21, id: 1};
        let individual2 = IndividualWrapper{individual: IndividualTest1, fitness: 7.12, num_of_mutations: 7, id: 1};

        assert!(individual1 < individual2);
    }

    #[test]
    fn compare3() {
        let individual1 = IndividualWrapper{individual: IndividualTest1, fitness: 21.996, num_of_mutations: 11, id: 1};
        let individual2 = IndividualWrapper{individual: IndividualTest1, fitness: 21.996, num_of_mutations: 34, id: 1};

        assert!(individual1 == individual2);
    }
}
