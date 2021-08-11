
use crate::dw_server::DWFileFormat;
use crate::dw_node::DWMethod;

#[derive(Debug, Clone)]
pub struct DWConfiguration {
    // Server config:
    pub num_of_individuals: usize,
    pub fitness_limit: f64,
    pub export_file_name: String,
    pub save_new_best_individual: bool,
    pub file_format: DWFileFormat,

    // Node config:
    pub num_of_iterations: u64,
    pub num_of_mutations: u64,
    pub mutate_method: DWMethod,
    pub additional_fitness_threshold: Option<f64>,
}

impl Default for DWConfiguration {
    fn default() -> Self {
        DWConfiguration {
            num_of_individuals: 20,
            fitness_limit: 1.0,
            export_file_name: "best_population".to_string(),
            save_new_best_individual: false,
            file_format: DWFileFormat::JSON,

            num_of_iterations: 1000,
            num_of_mutations: 10,
            mutate_method: DWMethod::Simple,
            additional_fitness_threshold: None,
        }
    }
}