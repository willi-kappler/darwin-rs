
use std::fmt::{self, Display, Formatter};

use crate::dw_server::DWFileFormat;
use crate::dw_node::DWMutateMethod;
use crate::dw_population::DWDeleteMethod;

#[derive(Debug, Clone)]
pub struct DWConfiguration {
    // Common:
    pub max_population_size: usize,
    pub fitness_limit: f64,

    // Server config:
    pub export_file_name: String,
    pub save_new_best_individual: bool,
    pub file_format: DWFileFormat,

    // Node config:
    pub num_of_iterations: u64,
    pub num_of_mutations: u64,
    pub mutate_method: DWMutateMethod,
    pub delete_method: DWDeleteMethod,
    pub additional_fitness_threshold: Option<f64>,
    pub reset_limit: u64,
}

impl Default for DWConfiguration {
    fn default() -> Self {
        DWConfiguration {
            // Common:
            max_population_size: 20,
            fitness_limit: 1.0,

            // Server config:
            export_file_name: "best_population".to_string(),
            save_new_best_individual: false,
            file_format: DWFileFormat::JSON,

            // Node config:
            num_of_iterations: 1000,
            num_of_mutations: 10,
            mutate_method: DWMutateMethod::Simple,
            delete_method: DWDeleteMethod::SortUnique,
            additional_fitness_threshold: None,
            reset_limit: 100,
        }
    }
}

impl Display for DWConfiguration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Common: max population size: '{}', fitness limit: '{}'\n\
                   Server: export file name: '{}', save new best individual: '{}', file format: '{}'\n\
                   Node:num of iterations: '{}', num of mutations: '{}', reset limit: '{}',\n\
                   mutate method: '{}', delete method: '{}'",
           self.max_population_size, self.fitness_limit, self.export_file_name, self.save_new_best_individual,
           self.file_format, self.num_of_iterations, self.num_of_mutations, self.reset_limit,
           self.mutate_method, self.delete_method)
    }
}
