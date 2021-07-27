
use crate::individual::{Individual, IndividualWrapper};

use node_crunch::{NCServer, NCJobStatus, NCConfiguration, NCError, NodeID,
    NCServerStarter, nc_decode_data, nc_encode_data};
use log::{debug, info, error};
use serde::{Serialize, de::DeserializeOwned};

use std::fs::File;
use std::io::{Write, Read};

pub struct SimulationServer<T> {
    population: Vec<IndividualWrapper<T>>,
    fitness_limit: f64,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    export_file_name: String,
}

impl<T: 'static + Individual + Clone + Send + Serialize + DeserializeOwned> SimulationServer<T> {
    pub fn new(initial: T, num_of_individuals: usize, fitness_limit: f64) -> Self {
        let mut population = Vec::with_capacity(num_of_individuals);

        for _ in 0..num_of_individuals {
            let mut individual = IndividualWrapper::new(initial.clone());
            individual.mutate();
            individual.calculate_fitness();
            population.push(individual);
        }

        Self {
            population,
            fitness_limit,
            num_of_individuals,
            nc_configuration: NCConfiguration::default(),
            export_file_name: "population_result.dat".to_string(),
        }
    }
    pub fn set_configuration(&mut self, nc_configuration: NCConfiguration) {
        self.nc_configuration = nc_configuration;
    }
    pub fn set_export_file_name(&mut self, export_file_name: &str) {
        self.export_file_name = export_file_name.to_string();
    }
    pub fn read_population(&mut self, file_name: &str) -> Result<(), NCError> {
        let mut file = File::open(file_name)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;

        let population = nc_decode_data::<Vec<IndividualWrapper<T>>>(&data)?;

        self.population = population;

        Ok(())
    }
    pub fn run(self) {
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
    pub fn save_population(&self) -> Result<(), NCError> {
        debug!("SimulationServer::save_population, to file: '{}'", self.export_file_name);

        let data = nc_encode_data(&self.population)?;
        let mut file = File::create(&self.export_file_name)?;

        file.write_all(&data)?;

        Ok(())
    }
    fn is_job_done(&self) -> bool {
        self.population[0].fitness < self.fitness_limit
    }
}

impl<T: 'static + Individual + Clone + Send + Serialize + DeserializeOwned> NCServer for SimulationServer<T> {
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

        match nc_decode_data::<IndividualWrapper<T>>(node_data) {
            Ok(individual) => {
                // TODO: Use a sorted data structure
                // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
                debug!("Fitness from node: {}", individual.get_fitness());

                self.population.push(individual);
                self.population.sort();
                self.population.dedup();
                self.population.truncate(self.num_of_individuals);

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
