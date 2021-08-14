

use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_config::DWConfiguration;
use crate::dw_error::DWError;

use node_crunch::{NCNode, NCConfiguration, NCError,
    NCNodeStarter, nc_decode_data, nc_encode_data};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};
use rand::{thread_rng, Rng};

use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq)]
pub enum DWMethod {
    Simple,
    OnlyBest,
    LowMem,
    Keep,
    Reset,
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
            "reset" => {
                Ok(DWMethod::Reset)
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
                Ok(DWMethod::Reset)
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
            DWMethod::Reset => {
                write!(f, "reset")
            }
        }
    }
}

pub struct DWNode<T> {
    population: Vec<DWIndividualWrapper<T>>,
    slow_population: Vec<DWIndividualWrapper<T>>,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    num_of_iterations: u64,
    num_of_mutations: u64,
    mutate_method: DWMethod,
    best_fitness: f64,
    best_counter: u64,
    fitness_limit: f64,
    additional_fitness_threshold: Option<f64>,
    reset_initial: Option<DWIndividualWrapper<T>>,
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> DWNode<T> {
    pub fn new(initial: T, dw_configuration: DWConfiguration, nc_configuration: NCConfiguration) -> Self {
        let num_of_individuals = dw_configuration.num_of_individuals;
        let mut population = Vec::with_capacity(num_of_individuals);

        for _ in 0..num_of_individuals {
            let mut individual = DWIndividualWrapper::new(initial.clone());
            individual.mutate();
            individual.calculate_fitness();
            population.push(individual);
        }

        population.sort();

        let best_fitness = population[0].get_fitness();

        let mut slow_population = Vec::new();
        let mut reset_initial = None;

        if dw_configuration.mutate_method == DWMethod::LowMem {
            let individual = population[0].clone();
            slow_population.push(individual);
        } else {
            slow_population = population.clone();

            if dw_configuration.mutate_method == DWMethod::Reset {
                reset_initial = Some(DWIndividualWrapper::new(initial.clone()));
            }
        }


        Self {
            population,
            slow_population,
            num_of_individuals,
            nc_configuration,
            num_of_iterations: dw_configuration.num_of_iterations,
            num_of_mutations: dw_configuration.num_of_mutations,
            mutate_method: dw_configuration.mutate_method,
            best_fitness,
            best_counter: 0,
            fitness_limit: dw_configuration.fitness_limit,
            additional_fitness_threshold: dw_configuration.additional_fitness_threshold,
            reset_initial,
        }
    }
    pub fn run(self) {
        debug!("Start node with config: population size: '{}', iterations: '{}', mutations: '{}', fitness limit: '{}', method: '{}'",
            self.num_of_individuals, self.num_of_iterations, self.num_of_mutations, self.fitness_limit, self.mutate_method);
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

    fn clean(&mut self) {
        self.population.sort();
        self.population.dedup();
        self.population.truncate(self.num_of_individuals);
    }

    fn mutate_with_other(&mut self) {
        let mut rng = thread_rng();

        let len = self.population.len();
        let index1 = rng.gen_range(0..len);
        let mut index2 = rng.gen_range(0..len);

        while index1 == index2 {
            index2 = rng.gen_range(0..len);
        }

        let mut individual = self.population[index1].clone();
        individual.mutate_with_other(&self.population[index2]);
        individual.calculate_fitness();
        self.population.push(individual);
    }
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> NCNode for DWNode<T> {
    fn process_data_from_server(&mut self, data: &[u8]) -> Result<Vec<u8>, NCError> {
        debug!("SimulationNode::process_data_from_server, new message received");

        let individual: DWIndividualWrapper<T> = nc_decode_data(&data)?;
        debug!("Individual from server, fitness: '{}'", individual.get_fitness());
        self.population.push(individual);
        self.population.sort();
        self.population.truncate(self.num_of_individuals);

        let mut rng = thread_rng();
        let index = rng.gen_range(0..self.slow_population.len());
        let new_individual = self.slow_population[index].clone();
        self.slow_population.push(new_individual);


        // TODO: Maybe use a sorted data structure
        // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

        match self.mutate_method {
            DWMethod::Simple => {
                for _ in 0..self.num_of_iterations {
                    let mut original1 = self.population.clone();
                    let mut original2 = self.slow_population.clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate();
                        }
                        individual.calculate_fitness();
                    }

                    self.population.append(&mut original1);
                    self.population.append(&mut original2);
                    self.mutate_with_other();
                    self.clean();

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    for individual in self.slow_population.iter_mut() {
                        individual.mutate();
                        individual.calculate_fitness();
                    }
                }
            }
            DWMethod::OnlyBest => {
                let mut potential_population = Vec::new();

                for _ in 0..self.num_of_iterations {
                    let mut original2 = self.slow_population.clone();

                    for individual in self.population.iter() {
                        let mut mutated = individual.clone();
                        let current_fitness = individual.get_fitness();

                        for _ in 0..self.num_of_mutations {
                            mutated.mutate();
                            mutated.calculate_fitness();
                            if mutated.get_fitness() < current_fitness {
                                potential_population.push(mutated.clone());
                            }
                        }
                    }

                    self.population.append(&mut potential_population);
                    self.population.append(&mut original2);
                    self.mutate_with_other();
                    self.clean();

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    for individual in self.slow_population.iter_mut() {
                        individual.mutate();
                        individual.calculate_fitness();
                    }
                }
            }
            DWMethod::LowMem => {
                for _ in 0..self.num_of_iterations {
                    let current_best = self.population[0].clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate();
                        }
                        individual.calculate_fitness();
                    }

                    self.population.push(current_best);
                    self.population.push(self.slow_population[0].clone());
                    self.mutate_with_other();
                    self.clean();

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    self.slow_population[0].mutate();
                    self.slow_population[0].calculate_fitness();
                }
            }
            DWMethod::Keep => {
                for _ in 0..self.num_of_iterations {
                    let mut original = self.slow_population.clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate();
                        }
                        individual.calculate_fitness();
                    }

                    self.population.append(&mut original);
                    self.mutate_with_other();
                    self.clean();

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    for individual in self.slow_population.iter_mut() {
                        individual.mutate();
                        individual.calculate_fitness();
                    }
                }
            }
            DWMethod::Reset => {
                if let Some(initial) = self.reset_initial.clone() {
                    for individual in self.population.iter_mut() {
                        *individual = initial.clone();
                        individual.mutate();
                        individual.calculate_fitness();
                    }
                }
                for _ in 0..self.num_of_iterations {
                    let mut original1 = self.population.clone();
                    let mut original2 = self.slow_population.clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate();
                        }
                        individual.calculate_fitness();
                    }

                    self.population.append(&mut original1);
                    self.population.append(&mut original2);
                    self.mutate_with_other();
                    self.clean();

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    for individual in self.slow_population.iter_mut() {
                        individual.mutate();
                        individual.calculate_fitness();
                    }
                }
            }
        }

        self.slow_population.sort();
        self.slow_population.dedup();
        self.slow_population.truncate(self.num_of_individuals);

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
        let fitness2 = self.population[self.num_of_individuals - 1].get_fitness();

        debug!("Difference between best and worst fitness: {}", fitness2 - fitness1);
        debug!("Slow population best fitness: '{}', worst fitness: '{}'",
            self.slow_population[0].get_fitness(), self.slow_population[self.slow_population.len() - 1].get_fitness());

        if fitness1 < self.best_fitness {
            self.best_fitness = fitness1;
            self.best_counter += 1;
            debug!("New best individual found: '{}', counter: '{}'", self.best_fitness, self.best_counter);
        }

        Ok(nc_encode_data(&best_individual)?)
    }
}
