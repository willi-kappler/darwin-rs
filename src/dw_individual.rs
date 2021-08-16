
use serde::{Serialize, Deserialize};

use std::cmp::Ordering;

pub trait DWIndividual {
    fn mutate(&mut self, other: &Self);

    fn calculate_fitness(&self) -> f64;

    fn get_additional_fitness(&self) -> f64 {
        0.0
    }

    fn random_reset(&mut self) {
    }

    fn new_best_individual(&self) {
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DWIndividualWrapper<T> {
    pub individual: T,
    pub fitness: f64,
}

impl<T: DWIndividual> DWIndividualWrapper<T> {
    pub fn new(individual: T) -> Self {
        Self {
            individual,
            fitness: f64::MAX,
        }
    }

    pub fn mutate(&mut self, other: &Self) {
        self.individual.mutate(&other.individual);
    }

    pub fn calculate_fitness(&mut self) {
        self.fitness = self.individual.calculate_fitness();
    }

    pub fn get_fitness(&self) -> f64 {
        self.fitness
    }

    pub fn get_additional_fitness(&self) -> f64 {
        self.individual.get_additional_fitness()
    }

    pub fn random_reset(&mut self) {
        self.individual.random_reset();
    }

    pub fn new_best_individual(&self) {
        self.individual.new_best_individual();
    }
}

impl<T> PartialEq for DWIndividualWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.fitness == other.fitness
    }
}

impl<T> Eq for DWIndividualWrapper<T> {}

impl<T> PartialOrd for DWIndividualWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}

impl<T> Ord for DWIndividualWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Fitness of individual is NaN")
    }
}
