

use darwin_rs::{DWNode, DWServer, DWIndividual, DWMethod, NCConfiguration, DWConfiguration};

use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, ConfigBuilder};
use serde::{Serialize, Deserialize};
use log::{error, debug};
use itertools::Itertools;

use std::{fs, io, io::BufRead};
use std::collections::HashMap;

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
    #[structopt(short = "l", long = "limit", default_value = "1.0")]
    limit: f64,
    #[structopt(short = "i", long = "iter", default_value = "1000")]
    num_of_iterations: u64,
    #[structopt(short = "m", long = "mutate", default_value = "10")]
    num_of_mutations: u64,
    #[structopt(short = "f", long = "file", default_value = "att532.txt")]
    input_file: String,
    #[structopt(long = "method", default_value = "only_best")]
    method: DWMethod,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TSP3 {
    cities: Vec<(f64, f64)>,
    mutation_counter: HashMap<u8, u64>,
}

impl TSP3 {
    pub fn new(cities: Vec<(f64, f64)>) -> Self {
        Self {
            cities,
            mutation_counter: HashMap::new(),
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

    fn calculate_length(&self, cities: &[(f64, f64)], len: usize) -> f64 {
        let mut length = 0.0;

        for i in 1..len {
            let (x1, y1) = cities[i - 1];
            let (x2, y2) = cities[i];
            let dx = x2 - x1;
            let dy = y2 - y1;
            length += dx.hypot(dy);
        }

        length
    }
}

impl DWIndividual for TSP3 {
    fn mutate(&mut self) {
        let mut rng = thread_rng();
        let last = self.cities.len();
        let index1 = rng.gen_range(1_usize..last);
        let mut index2 = rng.gen_range(1_usize..last);

        while index1 == index2 {
            index2 = rng.gen_range(1_usize..last);
        }

        let operation = rng.gen_range(0_u8..5);

        match operation {
            0 => {
                // Just swap two positions
                self.cities.swap(index1, index2);
                let counter = self.mutation_counter.entry(0).or_insert(0);
                *counter += 1;

            }
            1 => {
                // Rotate (shift) items
                let tmp = self.cities.remove(index1);
                self.cities.insert(index2, tmp);
                let counter = self.mutation_counter.entry(1).or_insert(0);
                *counter += 1;
            }
            2 => {
                // Reverse order of items
                let slice = if index1 < index2 {
                    &mut self.cities[index1..index2]
                } else {
                    &mut self.cities[index2..index1]
                };
                slice.reverse();
                let counter = self.mutation_counter.entry(2).or_insert(0);
                *counter += 1;
            }
            3 => {
                // Split and swap two parts
                let mut temp = vec![(0.0, 0.0); last];
                temp[0] = self.cities[0];
                let index3 = last - index1 + 1;

                for i in 1..index3 {
                    temp[i] = self.cities[index1 + i - 1];
                }
                for i in index3..last {
                    temp[i] = self.cities[i - index3 + 1];
                }
                self.cities = temp;
                let counter = self.mutation_counter.entry(3).or_insert(0);
                *counter += 1;
            }
            4 => {
                // Permutate a small slice and find best configuration
                let permut_len = rng.gen_range(3..8);
                let index = rng.gen_range(1_usize..(last - permut_len));
                let init = self.cities[index..(index + permut_len)].to_vec();
                let mut best = init.clone();
                let mut best_length = self.calculate_length(&best, permut_len);

                for permutation in init.into_iter().permutations(permut_len) {
                    let new_length = self.calculate_length(&permutation, permut_len);
                    if new_length < best_length {
                        best = permutation.clone();
                        best_length = new_length;
                    }
                }

                for i in index..(index + permut_len) {
                    self.cities[i] = best[i - index]
                }
                let counter = self.mutation_counter.entry(4).or_insert(0);
                *counter += 1;
            }
            _ => {
                error!("Unknown operation: '{}'", operation);
            }
        }
    }

    fn mutate_with_other(&mut self, other: &Self) {
        let mut rng = thread_rng();

        let mut result = Vec::new();
        result.push(self.cities[0]);

        let mut index1 = 1;
        let mut index2 = 1;

        while result.len() < self.cities.len() {
            if rng.gen::<bool>() {
                if index1 < self.cities.len() {
                    if !result.contains(&self.cities[index1]) {
                        result.push(self.cities[index1]);
                    }
                    index1 += 1;
                }
            } else {
                if index2 < other.cities.len() {
                    if !result.contains(&other.cities[index2]) {
                        result.push(other.cities[index2]);
                    }
                    index2 += 1;
                }
            }
        }

        self.cities = result;

        let counter = self.mutation_counter.entry(200).or_insert(0);
        *counter += 1;
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

    fn random_reset(&mut self) {
        let mut rng = thread_rng();
        self.cities[1..].shuffle(&mut rng);
        self.mutation_counter.clear();
    }

    fn new_best_individual(&self) {
        debug!("Mutations statistics:\nswap: {}, rotate: {}, reverse: {}, split: {}, permutation: {}, mutate with other: {}",
            self.mutation_counter.get(&0).unwrap_or(&0),
            self.mutation_counter.get(&1).unwrap_or(&0),
            self.mutation_counter.get(&2).unwrap_or(&0),
            self.mutation_counter.get(&3).unwrap_or(&0),
            self.mutation_counter.get(&4).unwrap_or(&0),
            self.mutation_counter.get(&200).unwrap_or(&0),
        );
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
        mutate_method: options.method,
        ..Default::default()
    };

    let log_level = LevelFilter::Debug;
    let log_config = ConfigBuilder::new()
        .set_time_format_str("%Y.%m.%d %H:%M:%S")
        .set_time_to_local(true)
        .add_filter_ignore_str("node_crunch")
        .build();

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
