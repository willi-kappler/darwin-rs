
use crate::dw_individual::{DWIndividual, DWIndividualWrapper};
use crate::dw_error::DWError;

use node_crunch::{NCServer, NCJobStatus, NCConfiguration, NodeID,
    NCServerStarter, nc_decode_data, nc_encode_data, NCError};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};
use serde_json;

use std::fs::File;
use std::io::{Write, Read};

pub enum DWFileFormat {
    Binary,
    JSON,
}

pub struct DWSimulationServer<T> {
    population: Vec<DWIndividualWrapper<T>>,
    fitness_limit: f64,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    export_file_name: String,
    save_new_best_individual: bool,
    individual_file_counter: u64,
    file_format: DWFileFormat,
}

impl<T: 'static + DWIndividual + Clone + Send + Serialize + DeserializeOwned> DWSimulationServer<T> {
    pub fn new(initial: T, num_of_individuals: usize, fitness_limit: f64) -> Self {
        let mut population = Vec::with_capacity(num_of_individuals);

        for _ in 0..num_of_individuals {
            let mut individual = DWIndividualWrapper::new(initial.clone());
            individual.mutate();
            individual.calculate_fitness();
            population.push(individual);
        }

        population.sort();

        Self {
            population,
            fitness_limit,
            num_of_individuals,
            nc_configuration: NCConfiguration::default(),
            export_file_name: "population_result.dat".to_string(),
            save_new_best_individual: false,
            individual_file_counter: 0,
            file_format: DWFileFormat::Binary,
        }
    }
    pub fn set_configuration(&mut self, nc_configuration: NCConfiguration) {
        self.nc_configuration = nc_configuration;
    }
    pub fn set_export_file_name(&mut self, export_file_name: &str) {
        self.export_file_name = export_file_name.to_string();
    }
    pub fn set_save_new_best_individual(&mut self, save_new_best_individual: bool) {
        self.save_new_best_individual = save_new_best_individual;
    }
    pub fn set_population(&mut self, population: Vec<DWIndividualWrapper<T>>) {
        self.population = population;
    }
    pub fn set_file_format(&mut self, file_format: DWFileFormat) {
        self.file_format = file_format;
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
                info!("Calculation finished");
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

impl<T: 'static + DWIndividual + Clone + Send + Serialize + DeserializeOwned> NCServer for DWSimulationServer<T> {
    fn prepare_data_for_node(&mut self, node_id: NodeID) -> Result<NCJobStatus, NCError> {
        debug!("SimulationServer::prepare_data_for_node, node_id: {}", node_id);

        if self.is_job_done() {
            Ok(NCJobStatus::Finished)
        } else {
            let individual = self.population[0].clone();

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

        match nc_decode_data::<Option<DWIndividualWrapper<T>>>(node_data) {
            Ok(Some(individual)) => {
                // TODO: Use a sorted data structure
                // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
                let fitness = individual.get_fitness();
                let best_fitness = self.population[0].get_fitness();

                if fitness < best_fitness {
                    debug!("New best individual found: '{}', node_id: '{}'", fitness, node_id);

                    self.population.insert(0, individual);
                    self.population.truncate(self.num_of_individuals);

                    if self.save_new_best_individual {
                        match self.save_individual(0) {
                            Ok(_) => {

                            }
                            Err(e) => {
                                error!("An error occurred while saving the new best individual: {}", e);
                            }
                        }
                    }
                } else {
                    debug!("No new best individual found, fitness: '{}' >= best fitness: '{}'", fitness, best_fitness);
                }

                Ok(())
            }
            Ok(None) => {
                debug!("No new best individual found by node");
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
