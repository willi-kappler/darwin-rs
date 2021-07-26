
use serde::{Serialize, Deserialize};

use std::cmp::Ordering;

pub trait Individual {
    fn mutate(&mut self);
    fn calculate_fitness(&self) -> f64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualWrapper<T> {
    pub individual: T,
    pub fitness: f64,
}

impl<T: Individual> IndividualWrapper<T> {
    pub fn new(individual: T) -> Self {
        Self {
            individual,
            fitness: f64::MAX,
        }
    }
    pub fn mutate(&mut self) {
        self.individual.mutate();
    }
    pub fn calculate_fitness(&mut self) {
        self.fitness = self.individual.calculate_fitness();
    }
    pub fn get_fitness(&self) -> f64 {
        self.fitness
    }
}

impl<T> PartialEq for IndividualWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.fitness == other.fitness
    }
}

impl<T> Eq for IndividualWrapper<T> {}

impl<T> PartialOrd for IndividualWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}

impl<T> Ord for IndividualWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Fitness of individual is NaN")
    }
}
