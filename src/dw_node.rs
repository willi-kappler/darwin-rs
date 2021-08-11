
use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_config::DWConfiguration;

use node_crunch::{NCNode, NCConfiguration, NCError,
    NCNodeStarter, nc_decode_data, nc_encode_data};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone)]
pub enum DWMethod {
    Simple,
    OnlyBest,
    LowMem,
}

pub struct DWNode<T> {
    population: Vec<DWIndividualWrapper<T>>,
    unsorted_population: Vec<DWIndividualWrapper<T>>,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    num_of_iterations: u64,
    num_of_mutations: u64,
    mutate_method: DWMethod,
    best_fitness: f64,
    best_counter: u64,
    fitness_limit: f64,
    additional_fitness_threshold: Option<f64>,
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

        Self {
            population,
            unsorted_population: Vec::new(),
            num_of_individuals,
            nc_configuration,
            num_of_iterations: dw_configuration.num_of_iterations,
            num_of_mutations: dw_configuration.num_of_mutations,
            mutate_method: dw_configuration.mutate_method,
            best_fitness,
            best_counter: 0,
            fitness_limit: dw_configuration.fitness_limit,
            additional_fitness_threshold: dw_configuration.additional_fitness_threshold,
        }
    }
    pub fn run(mut self) {
        debug!("Start node with config: population size: '{}', iterations: '{}', mutations: '{}', fitness limit: '{}'",
            self.num_of_individuals, self.num_of_iterations, self.num_of_mutations, self.fitness_limit);
        debug!("Starting with best fitness: {}", self.best_fitness);

        match self.mutate_method {
            DWMethod::LowMem => {
                let mut individual = self.population[0].clone();
                individual.mutate();
                individual.calculate_fitness();
                self.unsorted_population.push(individual);
            }
            _ => {
                self.unsorted_population = self.population.clone();
            }
        }

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
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> NCNode for DWNode<T> {
    fn process_data_from_server(&mut self, data: &[u8]) -> Result<Vec<u8>, NCError> {
        debug!("SimulationNode::process_data_from_server, new message received");

        let individual: DWIndividualWrapper<T> = nc_decode_data(&data)?;
        let fitness = individual.get_fitness();

        if fitness < self.best_fitness {
            debug!("New best individual from server with fitness: '{}'", fitness);
            self.population.push(individual);
            self.best_fitness = fitness;
        }

        match self.mutate_method {
            DWMethod::Simple => {
                for _ in 0..self.num_of_iterations {
                    let mut original1 = self.population.clone();
                    let mut original2 = self.unsorted_population.clone();

                    for individual in self.population.iter_mut() {
                        for _ in 0..self.num_of_mutations {
                            individual.mutate();
                        }
                        individual.calculate_fitness();
                    }

                    // TODO: use a sorted data structure
                    // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
                    self.population.append(&mut original1);
                    self.population.append(&mut original2);
                    self.population.sort();
                    self.population.dedup();
                    self.population.truncate(self.num_of_individuals);

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    for individual in self.unsorted_population.iter_mut() {
                        individual.mutate();
                        individual.calculate_fitness();
                    }
                }
            }
            DWMethod::OnlyBest => {
                let mut potential_population = Vec::new();

                for _ in 0..self.num_of_iterations {
                    let mut original2 = self.unsorted_population.clone();

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
                    self.population.sort();
                    self.population.dedup();
                    self.population.truncate(self.num_of_individuals);

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    for individual in self.unsorted_population.iter_mut() {
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
                    self.population.push(self.unsorted_population[0].clone());
                    self.population.sort();
                    self.population.dedup();
                    self.population.truncate(self.num_of_individuals);

                    if self.population[0].get_fitness() < self.fitness_limit {
                        break
                    }

                    self.unsorted_population[0].mutate();
                    self.unsorted_population[0].calculate_fitness();
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
        let fitness = best_individual.get_fitness();

        let individual = if fitness < self.best_fitness {
            self.best_counter += 1;
            debug!("Sending best individual to server, with fitness: '{}', counter: {}", fitness, self.best_counter);
            self.best_fitness = fitness;
            Some(best_individual)
        } else {
            debug!("No new best individual found, fitness: '{}' >= best fitness: '{}'", fitness, self.best_fitness);
            None
        };

        Ok(nc_encode_data(&individual)?)
    }
}
