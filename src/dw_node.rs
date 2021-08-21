

use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_config::DWConfiguration;
use crate::dw_error::DWError;
use crate::dw_population::DWPopulation;

use node_crunch::{NCNode, NCConfiguration, NCError, NodeID,
    NCNodeStarter, nc_decode_data, nc_encode_data};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};

use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq)]
pub enum DWMutateMethod {
    Simple,
    OnlyBest,
    LowMem,
}

impl FromStr for DWMutateMethod {
    type Err = DWError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => {
                Ok(DWMutateMethod::Simple)
            }
            "only_best" => {
                Ok(DWMutateMethod::OnlyBest)
            }
            "low_mem" => {
                Ok(DWMutateMethod::LowMem)
            }
            _ => {
                Err(DWError::ParseDWMethodError(s.to_string()))
            }
        }
    }
}

impl TryFrom<u8> for DWMutateMethod {
    type Error = DWError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => {
                Ok(DWMutateMethod::Simple)
            }
            1 => {
                Ok(DWMutateMethod::OnlyBest)
            }
            2 => {
                Ok(DWMutateMethod::LowMem)
            }
            _ => {
                Err(DWError::ConvertDWMutateMethodError(value))
            }
        }
    }
}

impl Display for DWMutateMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DWMutateMethod::Simple => {
                write!(f, "simple")
            }
            DWMutateMethod::OnlyBest => {
                write!(f, "only_best")
            }
            DWMutateMethod::LowMem => {
                write!(f, "low_mem")
            }
        }
    }
}

pub struct DWNode<T> {
    population: DWPopulation<T>,
    nc_configuration: NCConfiguration,
    num_of_iterations: u64,
    mutate_method: DWMutateMethod,
    best_counter: u64,
}

impl<T: DWIndividual + Clone + Serialize + DeserializeOwned> DWNode<T> {
    pub fn new(initial: T, dw_configuration: DWConfiguration, nc_configuration: NCConfiguration) -> Self {
        let initial = DWIndividualWrapper::new(initial);
        let population = DWPopulation::new(initial, &dw_configuration);

        debug!("DW Configuration: {}", dw_configuration);
        debug!("NC Configuration: {}", nc_configuration);
        debug!("Initial best fitness: {}", population.get_best_fitness());

        Self {
            population,
            nc_configuration,
            num_of_iterations: dw_configuration.num_of_iterations,
            mutate_method: dw_configuration.mutate_method,
            best_counter: 0,
        }
    }

    pub fn run(self) {

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
    fn set_initial_data(&mut self, node_id: NodeID, _initial_data: Option<Vec<u8>>) -> Result<(), NCError> {
        debug!("Got new node id: {}", node_id);
        Ok(())
    }

    fn process_data_from_server(&mut self, data: &[u8]) -> Result<Vec<u8>, NCError> {
        debug!("SimulationNode::process_data_from_server, new message received");

        let individual: DWIndividualWrapper<T> = nc_decode_data(&data)?;
        debug!("Individual from server, fitness: '{}'", individual.get_fitness());

        self.population.check_reset(individual);
        self.population.reseed_rng();

        match self.mutate_method {
            DWMutateMethod::Simple => {
                for _ in 0..self.num_of_iterations {
                    self.population.mutate_all_clone();
                    self.population.delete();

                    if self.population.is_job_done() {
                        break
                    }
                }
            }
            DWMutateMethod::OnlyBest => {
                for _ in 0..self.num_of_iterations {
                    self.population.mutate_all_only_best();
                    self.population.delete();

                    if self.population.is_job_done() {
                        break
                    }
                }
            }
            DWMutateMethod::LowMem => {
                for _ in 0..self.num_of_iterations {
                    self.population.mutate_random_single_clone();
                    self.population.delete();

                    if self.population.is_job_done() {
                        break
                    }
                }
            }
        }

        self.population.log_fitness();
        let (best_fitness, worst_fitness) = self.population.get_best_and_worst_fitness();
        debug!("Difference between best and worst fitness: '{}', ratio: '{}', median: '{}'",
            worst_fitness - best_fitness, best_fitness / worst_fitness, (best_fitness + worst_fitness) / 2.0);

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
