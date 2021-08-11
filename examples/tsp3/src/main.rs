

use darwin_rs::{DWNode, DWServer, DWIndividual, NCConfiguration, DWConfiguration};

use nanorand::{Rng, WyRand};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, ConfigBuilder};
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

        let operation = rng.generate_range(0_u8..3);

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
            2 => {
                // Reverse order of items
                let slice = if index1 < index2 {
                    &mut self.cities[index1..index2]
                } else {
                    &mut self.cities[index2..index1]
                };
                slice.reverse();
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

    let dw_configuration = DWConfiguration {
        num_of_individuals: options.population,
        fitness_limit: options.limit,
        num_of_iterations: options.num_of_iterations,
        num_of_mutations: options.num_of_mutations,
        ..Default::default()
    };

    let log_level = LevelFilter::Debug;
    let log_config = ConfigBuilder::new().set_time_format_str("%Y.%m.%d %H:%M:%S").build();

    if options.server {
        let log_file = fs::File::create("server.log").unwrap();
        WriteLogger::init(log_level, log_config, log_file).unwrap();

        let server = DWServer::new(tsp3, dw_configuration, nc_configuration);
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

        let node = DWNode::new(tsp3, dw_configuration, nc_configuration);
        node.run();
    }
}
