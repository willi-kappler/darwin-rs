

use darwin_rs::{DWSimulationNode, DWSimulationServer, DWIndividual, DWMethod, NCConfiguration};

use nanorand::{Rng, WyRand};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, Config};
use serde::{Serialize, Deserialize};

use std::fs;

#[derive(StructOpt, Debug)]
#[structopt(name = "ocr1")]
pub struct OCR1Opt {
    #[structopt(short = "s", long = "server")]
    server: bool,
    #[structopt(long = "ip", default_value = "127.0.0.1")]
    ip: String,
    #[structopt(short = "p", long = "port", default_value = "2020")]
    port: u16,
    #[structopt(short = "o", long = "population", default_value = "20")]
    population: usize,
    #[structopt(short = "l", long = "limit", default_value = "0.0")]
    limit: f64,
    #[structopt(short = "i", long = "iter", default_value = "1000")]
    num_of_iterations: u64,
    #[structopt(short = "m", long = "mutate", default_value = "10")]
    num_of_mutations: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OCR1 {
}

impl OCR1 {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl DWIndividual for OCR1 {
    fn mutate(&mut self) {
        let mut rng = WyRand::new();
    }
    fn calculate_fitness(&self) -> f64 {
        0.0
    }
}

fn main() {
    let options = OCR1Opt::from_args();
    let ocr1 = OCR1::new();

    let nc_configuration = NCConfiguration {
        port: options.port,
        address: options.ip,
        ..Default::default()
    };

    let log_level = LevelFilter::Debug;
    let log_config = Config::default();

    if options.server {
        let log_file = fs::File::create("server.log").unwrap();
        WriteLogger::init(log_level, log_config, log_file).unwrap();

        let mut server = DWSimulationServer::new(ocr1, options.population, options.limit);
        server.set_configuration(nc_configuration);
        server.run();
    } else {
        let mut postfix: u64 = 1;
        let mut log_file_name = format!("nc_node_{:08}.log", postfix);

        loop {
            if fs::metadata(&log_file_name).is_ok() {
                // Filename for logging already exists, try another one...
               postfix += 1;
               log_file_name = format!("nc_node_{:08}.log", postfix);
            } else {
                break
            }
        }

        let log_file = fs::File::create(&log_file_name).unwrap();
        WriteLogger::init(log_level, log_config, log_file).unwrap();

        let mut node = DWSimulationNode::new(ocr1, options.population);
        node.set_configuration(nc_configuration);
        node.set_num_of_iteration(options.num_of_iterations);
        node.set_num_of_mutations(options.num_of_mutations);
        node.set_fitness_limit(options.limit);
        node.set_method(DWMethod::Simple);
        node.run();
    }
}
