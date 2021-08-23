
use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_error::DWError;
use crate::dw_config::DWConfiguration;
use crate::dw_population::DWPopulation;

use node_crunch::{NCServer, NCJobStatus, NCConfiguration, NodeID,
    NCServerStarter, nc_decode_data, nc_encode_data, NCError};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};

use std::fs::File;
use std::io::{Write, Read};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum DWFileFormat {
    Binary,
    JSON,
}

impl Display for DWFileFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DWFileFormat::Binary => {
                write!(f, "binary")
            }
            DWFileFormat::JSON => {
                write!(f, "json")
            }
        }
    }
}

pub struct DWServer<T> {
    population: DWPopulation<T>,
    nc_configuration: NCConfiguration,
    export_file_name: String,
    save_new_best_individual: bool,
    individual_file_counter: u64,
    file_format: DWFileFormat,
    node_score: HashMap<NodeID, u64>,
}

impl<T: 'static + DWIndividual + Clone + Send + Serialize + DeserializeOwned> DWServer<T> {
    pub fn new(initial: T, dw_configuration: DWConfiguration, nc_configuration: NCConfiguration) -> Self {
        let initial = DWIndividualWrapper::new(initial);
        let population = DWPopulation::new(initial, &dw_configuration);

        debug!("DW Configuration:\n{}", dw_configuration);
        debug!("NC Configuration:\n{}", nc_configuration);
        debug!("Initial best fitness: '{}'", population.get_best_fitness());

        Self {
            population,
            nc_configuration,
            export_file_name: dw_configuration.export_file_name,
            save_new_best_individual: dw_configuration.save_new_best_individual,
            individual_file_counter: 0,
            file_format: dw_configuration.file_format,
            node_score: HashMap::new(),
        }
    }

    pub fn set_population(&mut self, population: &mut Vec<DWIndividualWrapper<T>>) {
        self.population.from_vec(population);
    }

    pub fn read_population(&mut self, file_name: &str) -> Result<(), DWError> {
        let mut file = File::open(file_name)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;

        let mut population: Vec<DWIndividualWrapper<T>> = match self.file_format {
            DWFileFormat::Binary => {
                nc_decode_data(&data)?
            }
            DWFileFormat::JSON => {
                serde_json::from_slice(&data)?
            }
        };

        self.population.from_vec(&mut population);

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
        self.population.add_individual(individual);
    }

    pub fn run(self) {
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
                nc_encode_data(self.population.to_vec())?
            }
            DWFileFormat::JSON => {
                serde_json::ser::to_vec(self.population.to_vec())?
            }
        };

        let mut file = File::create(&self.export_file_name)?;

        file.write_all(&data)?;

        Ok(())
    }

    fn is_job_done(&self) -> bool {
        self.population.is_job_done()
    }

    fn save_best_individual(&mut self) -> Result<(), DWError> {
        let individual = self.population.get_best_individual();

        let (data, ext): (Vec<u8>, &str) = match self.file_format {
            DWFileFormat::Binary => {
                (nc_encode_data(&individual)?, "dat")
            }
            DWFileFormat::JSON => {
                (serde_json::ser::to_vec(&individual)?, "json")
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
            let individual = self.population.get_random_individual();

            match nc_encode_data(individual) {
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

        match nc_decode_data::<DWIndividualWrapper<T>>(node_data) {
            Ok(individual) => {
                debug!("Fitness from node: '{}'", individual.get_fitness());

                self.population.add_individual(individual);
                self.population.delete();

                if self.population.has_new_best_individual() {
                    let new_best_fitness = self.population.get_new_best_fitness();
                    self.population.get_best_individual().new_best_individual();

                    let counter = self.node_score.entry(node_id).or_insert(0);
                    *counter += 1;

                    debug!("New best individual found: '{}', node id: '{}', counter: '{}'", new_best_fitness, node_id, counter);
                    self.population.log_fitness();

                    if self.save_new_best_individual {
                        if let Err(e) = self.save_best_individual() {
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
