

use darwin_rs::{DWSimulationNode, DWSimulationServer, DWIndividual,
    DWFileFormat, NCConfiguration};

use nanorand::{Rng, WyRand};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, Config};
use serde::{Serialize, Deserialize};
use log::{error};

use std::{fs, io, io::BufRead};

#[derive(StructOpt, Debug)]
#[structopt(name = "tsp3")]
pub struct TSP3Opt {
    #[structopt(short = "s", long = "server")]
    server: bool,
    #[structopt(long = "ip", default_value = "127.0.0.1")]
    ip: String,
    #[structopt(short = "p", long = "port", default_value = "2020")]
    port: u16,
    #[structopt(short = "o", long = "population", default_value = "20")]
    population: usize,
    #[structopt(short = "l", long = "limit", default_value = "459.0")]
    limit: f64,
    #[structopt(short = "i", long = "iter", default_value = "1000")]
    num_of_iterations: u64,
    #[structopt(short = "m", long = "mutate", default_value = "10")]
    num_of_mutations: u64,
    #[structopt(short = "f", long = "file", default_value = "att532.txt")]
    input_file: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TSP3 {
    cities: Vec<(f64, f64)>,
}

impl TSP3 {
    pub fn new(cities: Vec<(f64, f64)>) -> Self {
        Self {
            cities,
        }
    }
    pub fn new_from_file(file_name: &str) -> Self {
        let mut cities: Vec<(f64, f64)> = Vec::new();

        // File structure:
        //
        // NODE_COORD_SECTION
        // 1 12183.3333 52233.3333
        // EOF

        let f = fs::File::open(file_name).unwrap();
        let f = io::BufReader::new(f);
        let mut inside_data_section = false;

        for line in f.lines() {
            let line = line.unwrap();

            if line.starts_with("NODE_COORD_SECTION") {
                inside_data_section = true;
            }
            else if line.starts_with("EOF") {
                inside_data_section = false;
            }
            else {
                if inside_data_section {
                    let mut line_items = line.split(char::is_whitespace).skip(1); // skip index
                    let x: f64 = line_items.next().unwrap().parse().unwrap();
                    let y: f64 = line_items.next().unwrap().parse().unwrap();
                    cities.push((x, y));
                }
            }
        }

        Self::new(cities)
    }
}

impl DWIndividual for TSP3 {
    fn mutate(&mut self) {
        let mut rng = WyRand::new();
        let last = self.cities.len();
        let index1 = rng.generate_range(1_usize..last);
        let mut index2 = rng.generate_range(1_usize..last);

        while index1 == index2 {
            index2 = rng.generate_range(1_usize..last);
        }

        let operation = rng.generate_range(0_u8..2);

        match operation {
            0 => {
                // Just swap two positions
                self.cities.swap(index1, index2);
            }
            1 => {
                // Rotate (shift) items
                let tmp = self.cities.remove(index1);
                self.cities.insert(index2, tmp);
            }
            _ => {
                error!("Unknown operation: '{}'", operation);
            }
        }
    }
    fn calculate_fitness(&self) -> f64 {
        let mut distance = 0.0;
        let last = self.cities.len() - 1;

        let (mut px, mut py) = self.cities[last];

        for (x, y) in self.cities.iter() {
            let dx = *x - px;
            let dy = *y - py;

            distance += dx.hypot(dy);

            px = *x;
            py = *y;
        }

        distance
    }
}

fn main() {
    let options = TSP3Opt::from_args();
    let tsp3 = TSP3::new_from_file(&options.input_file);

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

        let mut server = DWSimulationServer::new(tsp3, options.population, options.limit);
        server.set_configuration(nc_configuration);
        // server.set_file_format(DWFileFormat::JSON);
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

        let mut node = DWSimulationNode::new(tsp3, options.population);
        node.set_configuration(nc_configuration);
        node.set_num_of_iteration(options.num_of_iterations);
        node.set_num_of_mutations(options.num_of_mutations);
        node.set_fitness_limit(options.limit);
        node.run();
    }
}
