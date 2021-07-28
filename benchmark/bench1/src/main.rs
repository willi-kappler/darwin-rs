

use darwin_rs::{SimulationNode, SimulationServer, Individual, Method, NCConfiguration};

use nanorand::{Rng, WyRand};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, Config};
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
    #[structopt(short = "l", long = "limit", default_value = "459.0")]
    limit: f64,
    #[structopt(short = "i", long = "iter", default_value = "1000")]
    num_of_iterations: u64,
    #[structopt(short = "m", long = "mutate", default_value = "10")]
    num_of_mutations: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Queens {
    board: Vec<u8>,
    seed: u64,
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
            seed: 0,
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

impl Individual for Queens {
    fn mutate(&mut self) {
        let mut rng = WyRand::new();
        // Make rng reproducible for benchmark
        rng.reseed(&self.seed.to_ne_bytes());
        self.seed += 10;

        let last = self.board.len();
        let mut index1 = rng.generate_range(1_usize..last);
        let mut index2 = rng.generate_range(1_usize..last);

        // Pick a position where a queen is placed.
        // Try random position until it finds a queen
        while self.board[index1] != 1 {
            index1 = rng.generate_range(1_usize..last);
        }

        // Pick a position where no queen is placed and this index is different from the other
        while (index1 == index2) || (self.board[index2] != 0) {
            index2 = rng.generate_range(1_usize..last);
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

impl Clone for Queens {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            seed: self.seed + 100,
        }
    }
}

fn main() {
    let options = QueensOpt::from_args();
    let tsp = Queens::new();

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

        let mut server = SimulationServer::new(tsp, options.population, options.limit);
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

        let mut node = SimulationNode::new(tsp, options.population);
        node.set_configuration(nc_configuration);
        node.set_num_of_iteration(options.num_of_iterations);
        node.set_num_of_mutations(options.num_of_mutations);
        node.set_method(Method::Simple);
        node.run();
    }
}

// Time taken: 62.008323057 s, 1.03347205095 min, 0.0172245341825 h
// bd46a9294927f683421868786e64046ff683519388197d3b3b29786ac9320e6da85bf465c619edab5a0e4a95fd890e1412043503bb7d35071fd369aaba901295  population_result.dat

// Time taken: 62.018271139 s, 1.0336378523166667 min, 0.01722729753861111 h
// bd46a9294927f683421868786e64046ff683519388197d3b3b29786ac9320e6da85bf465c619edab5a0e4a95fd890e1412043503bb7d35071fd369aaba901295  population_result.dat

// Time taken: 62.008734951 s, 1.03347891585 min, 0.0172246485975 h
// bd46a9294927f683421868786e64046ff683519388197d3b3b29786ac9320e6da85bf465c619edab5a0e4a95fd890e1412043503bb7d35071fd369aaba901295  population_result.dat

// Time taken: 62.003685788 s, 1.0333947631333333 min, 0.01722324605222222 h
// bd46a9294927f683421868786e64046ff683519388197d3b3b29786ac9320e6da85bf465c619edab5a0e4a95fd890e1412043503bb7d35071fd369aaba901295  population_result.dat

