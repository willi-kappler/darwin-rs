
use crate::individual::{Individual, IndividualWrapper};

use node_crunch::{NCNode, NCError, NCConfiguration,
    NCNodeStarter, nc_decode_data, nc_encode_data};
use log::{info, error};
use serde::{Serialize, de::DeserializeOwned};

pub struct SimulationNode<T> {
    population: Vec<IndividualWrapper<T>>,
    unsorted_population: Vec<IndividualWrapper<T>>,
    num_of_individuals: usize,
    nc_configuration: NCConfiguration,
    num_of_iterations: u64,
    num_of_mutations: u64,
}

impl<T: Individual + Clone + Serialize + DeserializeOwned> SimulationNode<T> {
    pub fn new(initial: T, num_of_individuals: usize) -> Self {
        let mut population = Vec::with_capacity(num_of_individuals);

        for _ in 0..num_of_individuals {
            let individual = IndividualWrapper::new(initial.clone());
            population.push(individual);
        }

        let unsorted_population = population.clone();

        Self {
            population,
            unsorted_population,
            num_of_individuals,
            nc_configuration: NCConfiguration::default(),
            num_of_iterations: 1000,
            num_of_mutations: 10,
        }
    }
    pub fn set_configuration(&mut self, nc_configuration: NCConfiguration) {
        self.nc_configuration = nc_configuration;
    }
    pub fn set_num_of_iteration(&mut self, num_of_iterations: u64) {
        self.num_of_iterations = num_of_iterations;
    }
    pub fn set_num_of_mutations(&mut self, num_of_mutations: u64) {
        self.num_of_mutations = num_of_mutations;
    }
    pub fn run(self) {
        let mut node_starter = NCNodeStarter::new(self.nc_configuration.clone());

        match node_starter.start(self) {
            Ok(_) => {
                info!("Calculation finished");
            }
            Err(e) => {
                error!("An error occurred: {}", e);
            }
        }    
    }
}

impl<T: Individual + Clone + Serialize + DeserializeOwned> NCNode for SimulationNode<T> {
    fn process_data_from_server(&mut self, data: &[u8]) -> Result<Vec<u8>, NCError> {
        let individual: IndividualWrapper<T> = nc_decode_data(&data)?;
        self.population.push(individual);

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
            self.population.append(&mut original1);
            self.population.append(&mut original2);
            self.population.sort();
            self.population.dedup();
            self.population.truncate(self.num_of_individuals);

            for individual in self.unsorted_population.iter_mut() {
                individual.mutate();
                individual.calculate_fitness();
            }
        }

        let result = nc_encode_data(&self.population[0])?;
        Ok(result)
    }
}
