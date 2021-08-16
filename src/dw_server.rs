
use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_error::DWError;
use crate::dw_config::DWConfiguration;

use node_crunch::{NCServer, NCJobStatus, NCConfiguration, NodeID,
    NCServerStarter, nc_decode_data, nc_encode_data, NCError};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use rand::{thread_rng, Rng};

use std::fs::File;
use std::io::{Write, Read};
// use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum DWFileFormat {
    Binary,
    JSON,
}

pub struct DWServer<T> {
    population: Vec<DWIndividualWrapper<T>>,
    fitness_limit: f64,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    export_file_name: String,
    save_new_best_individual: bool,
    individual_file_counter: u64,
    file_format: DWFileFormat,
    best_fitness: f64,
    // node_score: HashMap<NodeID, u64>,
}

impl<T: 'static + DWIndividual + Clone + Send + Serialize + DeserializeOwned> DWServer<T> {
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

        Self {
            population,
            fitness_limit: dw_configuration.fitness_limit,
            num_of_individuals,
            nc_configuration,
            export_file_name: dw_configuration.export_file_name,
            save_new_best_individual: dw_configuration.save_new_best_individual,
            individual_file_counter: 0,
            file_format: dw_configuration.file_format,
            best_fitness,
            // node_score: HashMap::new(),
        }
    }
    pub fn set_population(&mut self, population: Vec<DWIndividualWrapper<T>>) {
        self.population = population;
    }
    pub fn read_population(&mut self, file_name: &str) -> Result<(), DWError> {
        let mut file = File::open(file_name)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;

        match self.file_format {
            DWFileFormat::Binary => {
                self.population = nc_decode_data(&data)?;
            }
            DWFileFormat::JSON => {
                self.population = serde_json::from_slice(&data)?;
            }
        }

        Ok(())
    }
    pub fn read_individual(&mut self, file_name: &str) -> Result<(), DWError> {
        let mut file = File::open(file_name)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;

        let individual: DWIndividualWrapper<T> = match self.file_format {
            DWFileFormat::Binary => {
                nc_decode_data(&data)?
            }
            DWFileFormat::JSON => {
                serde_json::from_slice(&data)?
            }
        };

        self.add_individual(individual);

        Ok(())
    }
    pub fn add_individual(&mut self, individual: DWIndividualWrapper<T>) {
        self.population.push(individual);
        self.population.sort();
        self.population.truncate(self.num_of_individuals);
    }
    pub fn run(self) {
        debug!("Start server with fitness limit: '{}', population size: '{}'", self.fitness_limit, self.num_of_individuals);

        let mut server_starter = NCServerStarter::new(self.nc_configuration.clone());

        match server_starter.start(self) {
            Ok(_) => {
                info!("Simulation finished");
            }
            Err(e) => {
                error!("An error occurred: {}", e);
            }
        }
    }
    pub fn save_population(&self) -> Result<(), DWError> {
        debug!("SimulationServer::save_population, to file: '{}'", self.export_file_name);

        let data: Vec<u8> = match self.file_format {
            DWFileFormat::Binary => {
                nc_encode_data(&self.population)?
            }
            DWFileFormat::JSON => {
                serde_json::ser::to_vec(&self.population)?
            }
        };

        let mut file = File::create(&self.export_file_name)?;

        file.write_all(&data)?;

        Ok(())
    }
    fn is_job_done(&self) -> bool {
        self.population[0].fitness < self.fitness_limit
    }
    fn save_individual(&mut self, index: usize) -> Result<(), DWError> {
        let (data, ext): (Vec<u8>, &str) = match self.file_format {
            DWFileFormat::Binary => {
                (nc_encode_data(&self.population[index])?, "dat")
            }
            DWFileFormat::JSON => {
                (serde_json::ser::to_vec(&self.population[index])?, "json")
            }
        };

        let file_name = format!("individual_{}.{}", self.individual_file_counter, ext);
        let mut file = File::create(&file_name)?;

        file.write_all(&data)?;

        self.individual_file_counter += 1;
        Ok(())
    }
}

impl<T: 'static + DWIndividual + Clone + Send + Serialize + DeserializeOwned> NCServer for DWServer<T> {
    fn prepare_data_for_node(&mut self, node_id: NodeID) -> Result<NCJobStatus, NCError> {
        debug!("SimulationServer::prepare_data_for_node, node_id: {}", node_id);

        if self.is_job_done() {
            Ok(NCJobStatus::Finished)
        } else {
            let mut rng = thread_rng();
            let index = rng.gen_range(0..self.population.len());
            let individual = self.population[index].clone();

            match nc_encode_data(&individual) {
                Ok(data) => {
                    debug!("preparing data for node {}", node_id);
                    Ok(NCJobStatus::Unfinished(data))
                }
                Err(e) => {
                    error!("An error occurred while preparing the data for the node: {}, error: {}", node_id, e);
                    Err(e)
                }
            }
        }
    }
    fn process_data_from_node(&mut self, node_id: NodeID, node_data: &[u8]) -> Result<(), NCError> {
        debug!("SimulationServer::process_data_from_node, node_id: {}", node_id);
        // TODO: Use a sorted data structure
        // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

        match nc_decode_data::<DWIndividualWrapper<T>>(node_data) {
            Ok(individual) => {
                self.population.push(individual);
                self.population.sort();
                self.population.dedup();
                self.population.truncate(self.num_of_individuals);

                let best_individual = &self.population[0];

                if best_individual.get_fitness() < self.best_fitness {
                    self.best_fitness = best_individual.get_fitness();
                    debug!("New best individual found: '{}', node_id: '{}'", self.best_fitness, node_id);
                    best_individual.new_best_individual();
                    // let counter = self.node_score.entry(node_id).or_insert(0);
                    // *counter += 1;
                    if self.save_new_best_individual {
                        if let Err(e) = self.save_individual(0) {
                            error!("An error occurred while saving the new best individual: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!("An error occurred while processing the data from the node: {}, error: {}", node_id, e);
                Err(e)
            }
        }
    }
    fn heartbeat_timeout(&mut self, _nodes: Vec<NodeID>) {
        // Nothing to do
    }
    fn finish_job(&mut self) {
        self.save_population().unwrap();
    }
}
