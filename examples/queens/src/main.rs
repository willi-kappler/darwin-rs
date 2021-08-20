

use darwin_rs::{DWNode, DWServer, DWIndividual, DWMutateMethod, NCConfiguration, DWConfiguration};

use rand::{thread_rng, Rng};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, ConfigBuilder};
use serde::{Serialize, Deserialize};

use std::fs;

#[derive(StructOpt, Debug)]
#[structopt(name = "queens")]
pub struct QueensOpt {
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
    #[structopt(long = "method", default_value = "only_best")]
    mutate_method: DWMutateMethod,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Queens {
    board: Vec<u8>,
}

impl Queens {
    pub fn new() -> Self {
        Self {
            board: vec![
                1,1,1,1,1,1,1,1,
                0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,
            ],
        }
    }
    fn one_trace(&self, row: usize, col: usize, dx: i8, dy: i8) -> u8 {
        let mut num_of_collisions = 0;
        let mut x: i16 = col as i16;
        let mut y: i16 = row as i16;

        loop {
            x += dx as i16;
            if (x < 0) || (x > 7) {
                break;
            }

            y += dy as i16;
            if (y < 0) || (y > 7) {
                break;
            }

            if self.board[((y * 8) + x) as usize] == 1 {
                num_of_collisions += 1;
            }
        }

        num_of_collisions
    }
    fn find_collisions(&self, row: usize, col: usize) -> u8 {
        let mut num_of_collisions = 0;

        // up
        num_of_collisions += self.one_trace(row, col, -1, 0);

        // up right
        num_of_collisions += self.one_trace(row, col, -1, 1);

        // right
        num_of_collisions += self.one_trace(row, col, 0, 1);

        // right down
        num_of_collisions += self.one_trace(row, col, 1, 1);

        // down
        num_of_collisions += self.one_trace(row, col, 1, 0);

        // down left
        num_of_collisions += self.one_trace(row, col, 1, -1);

        // left
        num_of_collisions += self.one_trace(row, col, 0, -1);

        // left top
        num_of_collisions += self.one_trace(row, col, -1, -1);

        num_of_collisions
    }
}

impl DWIndividual for Queens {
    fn mutate(&mut self, _other: &Self) {
        let mut rng = thread_rng();
        let last = self.board.len();
        let mut index1 = rng.gen_range(1_usize..last);
        let mut index2 = rng.gen_range(1_usize..last);

        // Pick a position where a queen is placed.
        // Try random position until it finds a queen
        while self.board[index1] != 1 {
            index1 = rng.gen_range(1_usize..last);
        }

        // Pick a position where no queen is placed and this index is different from the other
        while (index1 == index2) || (self.board[index2] != 0) {
            index2 = rng.gen_range(1_usize..last);
        }

        // Move queen onto an empty position
        self.board.swap(index1, index2);
    }
    fn calculate_fitness(&self) -> f64 {
        let mut num_of_collisions = 0;

        for row in 0..8 {
            for col in 0..8 {
                // Found a queen, so check for collisions
                if self.board[(row * 8) + col] == 1 {
                    num_of_collisions += self.find_collisions(row, col);
                }
            }
        }

        num_of_collisions as f64
    }
}

fn main() {
    let options = QueensOpt::from_args();
    let queens = Queens::new();

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

        let server = DWServer::new(queens, dw_configuration, nc_configuration);
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

        let node = DWNode::new(queens, dw_configuration, nc_configuration);
        node.run();
    }
}
