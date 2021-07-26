
use crate::individual::IndividualWrapper;

pub struct SimulationServer<T> {
    population: Vec<IndividualWrapper<T>>,
    fitness_limit: f64,
    num_of_individuals: usize,
}

impl SimulationServer {
    pub fn new<T: Individual + Clone>(initial: T, num_of_individuals: usize, fitness_limit: f64) -> Self {
        let mut population = Vec::with_capacity(num_of_individuals);

        for i in 0..num_of_individuals {
            population.push(initial.clone());
        }

        Self {
            population,
            fitness_limit,
            num_of_individuals,
        }
    }
    pub fn run(self) {
        // TODO:
    }
}

pub struct SimulationNode<T> {
    population: Vec<IndividualWrapper<T>>,
    unsorted_population: Vec<IndividualWrapper<T>>,
    num_of_individuals: usize,
}
