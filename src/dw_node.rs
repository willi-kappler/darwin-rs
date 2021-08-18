

use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_config::DWConfiguration;
use crate::dw_error::DWError;

use node_crunch::{NCNode, NCConfiguration, NCError,
    NCNodeStarter, nc_decode_data, nc_encode_data};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};
use rand::{thread_rng, Rng, rngs::ThreadRng};

use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq)]
pub enum DWMethod {
    Simple,
    OnlyBest,
    LowMem,
    Keep,
    RandomDelete,
}

impl FromStr for DWMethod {
    type Err = DWError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => {
                Ok(DWMethod::Simple)
            }
            "only_best" => {
                Ok(DWMethod::OnlyBest)
            }
            "low_mem" => {
                Ok(DWMethod::LowMem)
            }
            "keep" => {
                Ok(DWMethod::Keep)
            }
            "random_delete" => {
                Ok(DWMethod::RandomDelete)
            }
            _ => {
                Err(DWError::ParseDWMethodError(s.to_string()))
            }
        }
    }
}

impl TryFrom<u8> for DWMethod {
    type Error = DWError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => {
                Ok(DWMethod::Simple)
            }
            1 => {
                Ok(DWMethod::OnlyBest)
            }
            2 => {
                Ok(DWMethod::LowMem)
            }
            3 => {
                Ok(DWMethod::Keep)
            }
            4 => {
                Ok(DWMethod::RandomDelete)
            }
            _ => {
                Err(DWError::ConvertDWMethodError(value))
            }
        }
    }
}

impl Display for DWMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DWMethod::Simple => {
                write!(f, "simple")
            }
            DWMethod::OnlyBest => {
                write!(f, "only_best")
            }
            DWMethod::LowMem => {
                write!(f, "low_mem")
            }
            DWMethod::Keep => {
                write!(f, "keep")
            }
            DWMethod::RandomDelete => {
                write!(f, "random_delete")
            }
        }
    }
}

pub struct DWNode<T> {
    population: Vec<DWIndividualWrapper<T>>,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    num_of_iterations: u64,
    num_of_mutations: u64,
    mutate_method: DWMethod,
    best_fitness: f64,
    best_counter: u64,
    fitness_limit: f64,
    delete_limit: f64,
    additional_fitness_threshold: Option<f64>,
    reset_individual: DWIndividualWrapper<T>,
    reset_counter: u64,
    reset_limit: Option<u64>,
    rng: ThreadRng,
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> DWNode<T> {
    pub fn new(initial: T, dw_configuration: DWConfiguration, nc_configuration: NCConfiguration) -> Self {
        let num_of_individuals = dw_configuration.num_of_individuals;
        let mut population = Vec::with_capacity(num_of_individuals);
        let initial = DWIndividualWrapper::new(initial);

        for _ in 0..num_of_individuals {
            let mut individual = initial.clone();
            individual.mutate(&initial);
            individual.calculate_fitness();
            population.push(individual);
        }

        population.sort();

        let best_fitness = population[0].get_fitness();

        let reset_individual = initial;

        Self {
            population,
            num_of_individuals,
            nc_configuration,
            num_of_iterations: dw_configuration.num_of_iterations,
            num_of_mutations: dw_configuration.num_of_mutations,
            mutate_method: dw_configuration.mutate_method,
            best_fitness,
            best_counter: 0,
            fitness_limit: dw_configuration.fitness_limit,
            delete_limit: dw_configuration.delete_limit,
            additional_fitness_threshold: dw_configuration.additional_fitness_threshold,
            reset_individual,
            reset_counter: 0,
            reset_limit: dw_configuration.reset_limit,
            rng: thread_rng(),
        }
    }

    pub fn run(self) {
        debug!("Start node with config: population size: '{}', iterations: '{}', mutations: '{}', fitness limit: '{}', method: '{}', reset limit: '{}'",
            self.num_of_individuals, self.num_of_iterations, self.num_of_mutations, self.fitness_limit, self.mutate_method,
            if let Some(limit) = self.reset_limit { limit } else { 0 });
        debug!("Starting with best fitness: {}", self.best_fitness);

        let mut node_starter = NCNodeStarter::new(self.nc_configuration.clone());

        match node_starter.start(self) {
            Ok(_) => {
                info!("Simulation finished");
            }
            Err(e) => {
                error!("An error occurred: {}", e);
            }
        }
    }

    fn maybe_reset(&mut self) -> bool {
        match self.reset_limit {
            Some(limit) => {
                if self.population[0].get_fitness() == self.reset_individual.get_fitness() {
                    self.reset_counter += 1;
                    debug!("Reset counter increased: {}", self.reset_counter);

                    if self.reset_counter >= limit {
                        debug!("Reset counter max reached, resetting population with random_reset()");
                        self.reset_counter = 0;
                        self.best_counter = 0;

                        for individual in self.population.iter_mut() {
                            individual.random_reset();
                            individual.calculate_fitness();
                        }
                    }
                } else {
                    self.reset_counter = 0;
                    self.reset_individual = self.population[0].clone();
                }

                true
            }
            None => {
                false
            }
        }
    }

    fn clean(&mut self) {
        self.population.sort();

        let mut new_population = Vec::new();
        let first = self.population[0].clone();
        let mut limit = first.get_fitness();
        new_population.push(first);

        for i in 1..self.population.len() {
            let individual = self.population[i].clone();
            let fitness = individual.get_fitness();
            if fitness * self.delete_limit > limit {
                limit = fitness;
                new_population.push(individual);
            }
        }

        new_population.truncate(self.num_of_individuals);

        self.population = new_population;
    }

    fn job_done(&self) -> bool {
        self.population[0].get_fitness() < self.fitness_limit
    }

    fn random_index_from(&mut self, start: usize) -> usize {
        self.rng.gen_range(start..self.population.len())
    }

    fn random_index(&mut self) -> usize {
        self.random_index_from(0)
    }

    fn random_index_new(&mut self, index: usize) -> usize {
        let mut new_index = self.random_index();
        while index == new_index {
            new_index = self.random_index();
        }
        new_index
    }

    fn random_individual(&mut self) -> &DWIndividualWrapper<T> {
        let index = self.random_index();
        &self.population[index]
    }

    fn random_delete(&mut self) {
        while self.population.len() > self.num_of_individuals {
            // Keep the best, randomly remove the others
            let index = self.random_index_from(1);
            self.population.swap_remove(index);
        }
    }

    fn random_delete2(&mut self) {
        let mut worst_fitness = 0.0;

        for individual in self.population.iter() {
            let new_fitness = individual.get_fitness();
            if new_fitness > worst_fitness {
                worst_fitness = new_fitness;
            }
        }

        let mut index = 0;
        while self.population.len() > self.num_of_individuals {
            let dice = self.rng.gen::<f64>();
            let probability = (self.population[index].get_fitness() / worst_fitness) * 0.5;

            if dice < probability {
                self.population.swap_remove(index);
            }

            index += 1;
            if index >= self.population.len() {
                index = 0;
            }
        }
    }
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> NCNode for DWNode<T> {
    fn process_data_from_server(&mut self, data: &[u8]) -> Result<Vec<u8>, NCError> {
        debug!("SimulationNode::process_data_from_server, new message received");

        let individual: DWIndividualWrapper<T> = nc_decode_data(&data)?;
        debug!("Individual from server, fitness: '{}'", individual.get_fitness());

        if !self.maybe_reset() {
            self.population.push(individual);
        }

        match self.mutate_method {
            DWMethod::Simple => {
                for _ in 0..self.num_of_iterations {
                    let mut original1 = self.population.clone();
                    let other = self.random_individual().clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate(&other);
                        }
                        individual.calculate_fitness();
                    }

                    self.population.append(&mut original1);
                    self.clean();

                    if self.job_done() {
                        break
                    }
                }
            }
            DWMethod::OnlyBest => {
                let mut potential_population = Vec::new();

                for _ in 0..self.num_of_iterations {
                    let index = self.random_index();
                    let other = &self.population[index];

                    for individual in self.population.iter() {
                        let mut mutated = individual.clone();
                        let current_fitness = individual.get_fitness();

                        for _ in 0..self.num_of_mutations {
                            mutated.mutate(other);
                            mutated.calculate_fitness();
                            if mutated.get_fitness() < current_fitness {
                                potential_population.push(mutated.clone());
                            }
                        }
                    }

                    self.population.append(&mut potential_population);
                    self.clean();

                    if self.job_done() {
                        break
                    }
                }
            }
            DWMethod::LowMem => {
                for _ in 0..self.num_of_iterations {
                    let current_best = self.population[0].clone();
                    let other = self.random_individual().clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate(&other);
                        }
                        individual.calculate_fitness();
                    }

                    self.population.push(current_best);
                    self.clean();

                    if self.job_done() {
                        break
                    }
                }
            }
            DWMethod::Keep => {
                for _ in 0..self.num_of_iterations {
                    let other = self.random_individual().clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate(&other);
                        }
                        individual.calculate_fitness();
                    }

                    self.clean();

                    if self.job_done() {
                        break
                    }
                }
            }
            DWMethod::RandomDelete => {
                for _ in 0..self.num_of_iterations {
                    for _ in 0..self.num_of_mutations {
                        let index1 = self.random_index();
                        let index2 = self.random_index_new(index1);

                        let mut individual = self.population[index1].clone();
                        let other = &self.population[index2];
                        individual.mutate(other);
                        individual.calculate_fitness();

                        self.population.push(individual);
                    }

                    self.population.sort();
                    self.random_delete();

                    if self.job_done() {
                        break
                    }
                }
            }
        }

        if let Some(threshold) = self.additional_fitness_threshold {
            self.population.sort_by(|i1, i2|{
                let f1 = i1.get_fitness();
                let f2 = i2.get_fitness();

                if (f1 - f2).abs() < threshold {
                    let af1 = i1.get_additional_fitness();
                    let af2 = i2.get_additional_fitness();

                    af1.partial_cmp(&af2).unwrap()
                } else {
                    f1.partial_cmp(&f2).unwrap()
                }
            });
        }

        for individual in self.population.iter() {
            debug!("fitness: {}", individual.get_fitness());
        }

        let best_individual = &self.population[0];
        let fitness1 = best_individual.get_fitness();
        let fitness2 = self.population[self.population.len() - 1].get_fitness();

        debug!("Difference between best and worst fitness: {}", fitness2 - fitness1);

        if fitness1 < self.best_fitness {
            self.best_fitness = fitness1;
            self.best_counter += 1;
            debug!("New best individual found: '{}', counter: '{}'", self.best_fitness, self.best_counter);
            best_individual.new_best_individual();
        }

        Ok(nc_encode_data(best_individual)?)
    }
}
