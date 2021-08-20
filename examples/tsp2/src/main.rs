

use darwin_rs::{DWNode, DWServer, DWIndividual, DWMutateMethod, NCConfiguration, DWConfiguration};

use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, ConfigBuilder};
use serde::{Serialize, Deserialize};
use log::{error, debug};
use itertools::Itertools;

use std::fs;
use std::collections::HashMap;

#[derive(StructOpt, Debug)]
#[structopt(name = "tsp2")]
pub struct TSP2Opt {
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
    #[structopt(long = "method", default_value = "only_best")]
    mutate_method: DWMutateMethod,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TSP2 {
    cities: Vec<(f64, f64)>,
    mutation_counter: HashMap<u8, u64>,
}

impl TSP2 {
    pub fn new() -> Self {
        Self {
            cities: vec![(2.852197810188428, 90.31966506130796),
                        (33.62874999956513, 44.9790462485413),
                        (22.064901432163996, 83.9172876840628),
                        (20.595912954825923, 12.798762916676043),
                        (42.2234133639806, 88.41646877787616),
                        (94.18533963242542, 21.151217108254627),
                        (25.84671166792939, 63.707153428189514),
                        (13.051898250315553, 89.61945656056766),
                        (76.41370000896038, 97.20491253636689),
                        (18.832993288649792, 6.006559110093601),
                        (96.98045791932294, 72.23019966333018),
                        (71.93203564171793, 93.03998204972012),
                        (33.39161715459793, 5.13372283892819),
                        (25.23072873231501, 67.1123015383591),
                        (84.38812085016241, 90.80055533944926),
                        (29.20345964254656, 21.17642854392676),
                        (58.11390834674495, 66.93322778502613),
                        (22.070195932187254, 59.73489434853766),
                        (86.29060211377086, 83.14129496517567),
                        (55.760857794890796, 26.95947234362994)],
	     mutation_counter: HashMap::new(),
        }
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

impl DWIndividual for TSP2 {
    fn mutate(&mut self, other: &Self) {
        let mut rng = thread_rng();
        let last = self.cities.len();
        let index1 = rng.gen_range(1_usize..last);
        let mut index2 = rng.gen_range(1_usize..last);

        while index1 == index2 {
            index2 = rng.gen_range(1_usize..last);
        }

        let operation = rng.gen_range(0_u8..6);

        match operation {
            0 => {
                // Just swap two positions
                self.cities.swap(index1, index2);
                let counter = self.mutation_counter.entry(operation).or_insert(0);
                *counter += 1;

            }
            1 => {
                // Rotate (shift) items
                let tmp = self.cities.remove(index1);
                self.cities.insert(index2, tmp);
                let counter = self.mutation_counter.entry(operation).or_insert(0);
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
                let counter = self.mutation_counter.entry(operation).or_insert(0);
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
                let counter = self.mutation_counter.entry(operation).or_insert(0);
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
                let counter = self.mutation_counter.entry(operation).or_insert(0);
                *counter += 1;
            }
            5 => {
                // Take "genes" from other individual and mix them into self
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

                let counter = self.mutation_counter.entry(operation).or_insert(0);
                *counter += 1;
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
            self.mutation_counter.get(&5).unwrap_or(&0),
        );
    }
}

fn main() {
    let options = TSP2Opt::from_args();
    let tsp2 = TSP2::new();

    let nc_configuration = NCConfiguration {
        port: options.port,
        address: options.ip,
        ..Default::default()
    };

    let dw_configuration = DWConfiguration {
        max_population_size: options.population,
        fitness_limit: options.limit,
        num_of_iterations: options.num_of_iterations,
        num_of_mutations: options.num_of_mutations,
        mutate_method: options.mutate_method,
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

        let server = DWServer::new(tsp2, dw_configuration, nc_configuration);
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

        let node = DWNode::new(tsp2, dw_configuration, nc_configuration);
        node.run();
    }
}
