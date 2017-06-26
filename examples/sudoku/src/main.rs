// This example implements a sudoku solver:
// https://en.wikipedia.org/wiki/Sudoku
// using an evolutionary algorithm.

extern crate rand;
extern crate simplelog;

// internal crates
extern crate darwin_rs;

use std::sync::Arc;
use rand::Rng;
use simplelog::{SimpleLogger, LogLevelFilter, Config};

// internal modules
use darwin_rs::{Individual, SimulationBuilder, Population, PopulationBuilder, simulation_builder};

// A cell is a 3x3 sub field inside the 9x9 sudoku field
fn fitness_of_one_cell(sudoku: &[u8], row: usize, col: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for i in 0..3 {
        for j in 0..3 {
            let data = sudoku[(((row + i) * 9) + col + j) as usize];

            if data > 0 && data < 10 {
                number_occurence[(data - 1) as usize] += 1;
            } else {
                error += 1.0;
            }
        }
    }

    for number in number_occurence {
        if number != 1 {
            error += 1.0;
        }
    }

    error
}

fn fitness_of_one_row(sudoku: &[u8], row: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for col in 0..9 {
        let number = sudoku[(row * 9) + col];
        if number > 0 && number < 10 {
            number_occurence[(number - 1) as usize] += 1;
        } else {
            error += 1.0;
        }
    }

    // Each number must be unique, otherwise increase error
    for number in number_occurence {
        if number != 1 {
            error += 1.0;
        }
    }

    error
}

fn fitness_of_one_col(sudoku: &[u8], col: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for row in 0..9 {
        let number = sudoku[(row * 9) + col];
        if number > 0 && number < 10 {
            number_occurence[(number - 1) as usize] += 1;
        } else {
            error += 1.0;
        }
    }

    // Each number must be unique, otherwise increase error
    for number in number_occurence {
        if number != 1 {
            error += 1.0;
        }
    }

    error
}

fn make_population(count: u32, unsolved: Vec<u8>) -> Vec<Sudoku> {
    let mut result = Vec::new();

    let shared = Arc::new(unsolved.clone());

    for _ in 0..count {
        result.push( Sudoku {
                solved: unsolved.clone(),
                unsolved: shared.clone()
            }
        );
    }

    result
}

fn make_all_populations(individuals: u32, populations: u32) -> Vec<Population<Sudoku>> {
    let mut result = Vec::new();

    let unsolved = vec![5, 3, 4, 6, 7, 8, 9, 1, 2,
                        6, 7, 2, 1, 9, 5, 3, 4, 8,
                        1, 9, 8, 3, 4, 2, 5, 6, 7,
                        8, 5, 9, 7, 6, 1, 4, 2, 3,
                        4, 2, 6, 8, 5, 3, 7, 9, 1,
                        7, 1, 3, 9, 2, 4, 8, 5, 6,
                        0, 6, 0, 0, 0, 0, 2, 8, 0,
                        0, 0, 0, 4, 1, 9, 0, 0, 5,
                        0, 0, 0, 0, 8, 0, 0, 7, 9];

    let initial_population = make_population(individuals, unsolved);

    for i in 1..(populations + 1) {
        let pop = PopulationBuilder::<Sudoku>::new()
            .set_id(i)
            .initial_population(&initial_population)
            .mutation_rate((1..10).cycle().take(individuals as usize).collect())
            .finalize().unwrap();

        result.push(pop);
    }

    result
}

#[derive(Debug, Clone)]
struct Sudoku {
    solved: Vec<u8>,
    unsolved: Arc<Vec<u8>>
}

// implement trait functions mutate and calculate_fitness:
impl Individual for Sudoku {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        let mut index: usize = rng.gen_range(0, self.solved.len());

        // pick free (= not pre set) position
        while self.unsolved[index] != 0 {
            index = rng.gen_range(0, self.solved.len());
        }

        // and set it to a random value
        self.solved[index] = rng.gen_range(1, 10);
    }

    // fitness means here: how many errors
    fn calculate_fitness(&mut self) -> f64 {
        let mut result = 0.0;

        for i in 0..9 {
            result += fitness_of_one_row(&self.solved, i);
            result += fitness_of_one_col(&self.solved, i);
        }


        let mut i = 0;
        let mut j = 0;

        loop {
            result += fitness_of_one_cell(&self.solved, i, j);

            i += 3;
            if i >= 9 {
                i = 0;
                j += 3;
                if j >= 9 {
                    break;
                }
            }
        }

        result
    }

    fn reset(&mut self) {
        self.solved = (*self.unsolved).clone();
    }
}

fn main() {
    println!("Darwin test: sudoku solver");

    let _ = SimpleLogger::init(LogLevelFilter::Info, Config::default());

    let sudoku = SimulationBuilder::<Sudoku>::new()
        .fitness(0.0)
        .threads(4)
        .add_multiple_populations(make_all_populations(100, 16))
        .finalize();

    match sudoku {
        Err(simulation_builder::Error(simulation_builder::ErrorKind::EndIterationTooLow, _)) => println!("more than 10 iteratons needed"),
        Err(e) => println!("unexpected error: {}", e),
        Ok(mut sudoku_simulation) => {
            sudoku_simulation.run();

            sudoku_simulation.print_fitness();

            // print solution
            for row in 0..9 {
                for col in 0..9 {
                    print!("{} | ",
                           sudoku_simulation.simulation_result.fittest[0].individual.solved[(row * 9) + col]);
                }
                println!("\n");
            }

            println!("total run time: {} ms", sudoku_simulation.total_time_in_ms);
            println!("improvement factor: {}",
                sudoku_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}",
                sudoku_simulation.simulation_result.iteration_counter);

        }
    }

}
