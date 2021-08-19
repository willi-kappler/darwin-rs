

use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_config::DWConfiguration;
use crate::dw_error::DWError;
use crate::dw_population::DWPopulation;

use node_crunch::{NCNode, NCConfiguration, NCError,
    NCNodeStarter, nc_decode_data, nc_encode_data};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};

use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq)]
pub enum DWMethod {
    Simple,
    OnlyBest,
    LowMem,
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
        }
    }
}

pub struct DWNode<T> {
    population: DWPopulation<T>,
    nc_configuration: NCConfiguration,
    num_of_iterations: u64,
    mutate_method: DWMethod,
    best_counter: u64,
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> DWNode<T> {
    pub fn new(initial: T, dw_configuration: DWConfiguration, nc_configuration: NCConfiguration) -> Self {
        let initial = DWIndividualWrapper::new(initial);
        let max_population_size = dw_configuration.num_of_individuals;
        let fitness_limit = dw_configuration.fitness_limit;
        let num_of_mutations = dw_configuration.num_of_mutations;
        let population = DWPopulation::new(initial, &dw_configuration);

        debug!("Max population size: '{}' fitness limit: '{}', mutations: '{}'", max_population_size, fitness_limit, num_of_mutations);

        Self {
            population,
            nc_configuration,
            num_of_iterations: dw_configuration.num_of_iterations,
            mutate_method: dw_configuration.mutate_method,
            best_counter: 0,
        }
    }

    pub fn run(self) {
        debug!("Start node with config: iterations: '{}', method: '{}'", self.num_of_iterations, self.mutate_method);
        debug!("Starting with best fitness: {}", self.population.get_best_fitness());

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
        debug!("Individual from server, fitness: '{}'", individual.get_fitness());

        self.population.add_individual(individual);

        match self.mutate_method {
            DWMethod::Simple => {
                for _ in 0..self.num_of_iterations {
                    self.population.mutate_all_clone();
                    self.population.random_delete();

                    if self.population.is_job_done() {
                        break
                    }
                }
            }
            DWMethod::OnlyBest => {

                for _ in 0..self.num_of_iterations {
                    self.population.mutate_all_only_best();
                    self.population.random_delete();

                    if self.population.is_job_done() {
                        break
                    }
                }
            }
            DWMethod::LowMem => {
                for _ in 0..self.num_of_iterations {
                    self.population.mutate_random_single_clone();
                    self.population.random_delete();

                    if self.population.is_job_done() {
                        break
                    }
                }
            }
        }

        self.population.log_fitness();
        let (best_fitness, worst_fitness) = self.population.get_best_and_worst_fitness();
        debug!("Difference between best and worst fitness: '{}', ratio: '{}'", worst_fitness - best_fitness, best_fitness / worst_fitness);

        let has_new_best_individual = self.population.has_new_best_individual();
        let best_individual = self.population.get_best_individual();

        if has_new_best_individual {
            let new_best_fitness = self.population.get_new_best_fitness();
            self.best_counter += 1;
            debug!("New best individual found: '{}', counter: '{}'", new_best_fitness, self.best_counter);
            best_individual.new_best_individual();
        }

        Ok(nc_encode_data(best_individual)?)
    }
}
