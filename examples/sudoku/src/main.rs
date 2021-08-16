

use darwin_rs::{DWNode, DWServer, DWIndividual, DWMethod, NCConfiguration, DWConfiguration};

use rand::{thread_rng, Rng};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, ConfigBuilder};
use serde::{Serialize, Deserialize};

use std::fs;

#[derive(StructOpt, Debug)]
#[structopt(name = "queens")]
pub struct SudokuOpt {
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
    method: DWMethod,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sudoku {
    solved: Vec<u8>,
    unsolved: Vec<u8>,
}

impl Sudoku {
    pub fn new() -> Self {
        let initial = vec![
            5, 3, 4, 6, 7, 8, 9, 1, 2,
            6, 7, 2, 1, 9, 5, 3, 4, 8,
            1, 9, 8, 3, 4, 2, 5, 6, 7,
            8, 5, 9, 7, 6, 1, 4, 2, 3,
            4, 2, 6, 8, 5, 3, 7, 9, 1,
            7, 1, 3, 9, 2, 4, 8, 5, 6,
            0, 6, 0, 0, 0, 0, 2, 8, 0,
            0, 0, 0, 4, 1, 9, 0, 0, 5,
            0, 0, 0, 0, 8, 0, 0, 7, 9];

        Self {
            unsolved: initial.clone(),
            solved: initial,
        }
    }
    // A cell is a 3x3 sub field inside the 9x9 sudoku field
    fn fitness_of_one_cell(&self, row: usize, col: usize) -> f64 {
        let mut number_occurrence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut error = 0.0;

        for i in 0..3 {
            for j in 0..3 {
                let data = self.solved[(((row + i) * 9) + col + j) as usize];

                if data > 0 && data < 10 {
                    number_occurrence[(data - 1) as usize] += 1;
                } else {
                    error += 1.0;
                }
            }
        }

        for number in number_occurrence {
            if number != 1 {
                error += 1.0;
            }
        }

        error
    }
    fn fitness_of_one_row(&self, row: usize) -> f64 {
        let mut number_occurrence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut error = 0.0;

        for col in 0..9 {
            let number = self.solved[(row * 9) + col];
            if number > 0 && number < 10 {
                number_occurrence[(number - 1) as usize] += 1;
            } else {
                error += 1.0;
            }
        }

        // Each number must be unique, otherwise increase error
        for number in number_occurrence {
            if number != 1 {
                error += 1.0;
            }
        }

        error
    }
    fn fitness_of_one_col(&self, col: usize) -> f64 {
        let mut number_occurrence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut error = 0.0;

        for row in 0..9 {
            let number = self.solved[(row * 9) + col];
            if number > 0 && number < 10 {
                number_occurrence[(number - 1) as usize] += 1;
            } else {
                error += 1.0;
            }
        }

        // Each number must be unique, otherwise increase error
        for number in number_occurrence {
            if number != 1 {
                error += 1.0;
            }
        }

        error
    }
}

impl DWIndividual for Sudoku {
    fn mutate(&mut self, _other: &Self) {
        let mut rng = thread_rng();
        let last = self.solved.len();

        let mut index: usize = rng.gen_range(0..last);

        // pick free (= not pre set) position
        while self.unsolved[index] != 0 {
            index = rng.gen_range(0..last);
        }

        // and set it to a random value
        self.solved[index] = rng.gen_range(1..10);
    }
    fn calculate_fitness(&self) -> f64 {
        let mut result = 0.0;

        for i in 0..9 {
            result += self.fitness_of_one_row(i);
            result += self.fitness_of_one_col(i);
        }

        for i in (0..9).step_by(3) {
            for j in (0..9).step_by(3) {
                result += self.fitness_of_one_cell(i, j);
            }
        }

        result
    }
}

fn main() {
    let options = SudokuOpt::from_args();
    let sudoku = Sudoku::new();

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

        let server = DWServer::new(sudoku, dw_configuration, nc_configuration);
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

        let node = DWNode::new(sudoku, dw_configuration, nc_configuration);
        node.run();
    }
}
