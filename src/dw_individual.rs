
use serde::{Serialize, Deserialize};

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
