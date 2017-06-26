// This example implements the queens problem:
// https://en.wikipedia.org/wiki/Eight_queens_puzzle
// using an evolutionary algorithm.

extern crate rand;
extern crate simplelog;

// internal crates
extern crate darwin_rs;

use rand::Rng;
use simplelog::{SimpleLogger, LogLevelFilter, Config};

// internal modules
use darwin_rs::{Individual, SimulationBuilder, Population, PopulationBuilder, simulation_builder};

#[derive(Debug, Clone)]
struct Queens {
    board: Vec<u8>,
}

// Chech one straight line in one specific direction
fn one_trace(board: &[u8], row: usize, col: usize, dy: i8, dx: i8) -> u8 {
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

        if board[((y * 8) + x) as usize] == 1 {
            num_of_collisions += 1;
        }
    }

    num_of_collisions
}

// Check all eight directions:
fn find_collisions(board: &[u8], row: usize, col: usize) -> u8 {
    let mut num_of_collisions = 0;

    // up
    num_of_collisions += one_trace(board, row, col, -1, 0);

    // up right
    num_of_collisions += one_trace(board, row, col, -1, 1);

    // right
    num_of_collisions += one_trace(board, row, col, 0, 1);

    // right down
    num_of_collisions += one_trace(board, row, col, 1, 1);

    // down
    num_of_collisions += one_trace(board, row, col, 1, 0);

    // down left
    num_of_collisions += one_trace(board, row, col, 1, -1);

    // left
    num_of_collisions += one_trace(board, row, col, 0, -1);

    // left top
    num_of_collisions += one_trace(board, row, col, -1, -1);

    num_of_collisions
}

fn make_population(count: u32) -> Vec<Queens> {
    let mut result = Vec::new();

    for _ in 0..count {
        result.push(
            Queens {
                // Start with all queens in one row
                board: vec![
                    1,1,1,1,1,1,1,1,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                ]
            }
        );
    }

    result
}

fn make_all_populations(individuals: u32, populations: u32) -> Vec<Population<Queens>> {
    let mut result = Vec::new();

    let initial_population = make_population(individuals);

    for i in 1..(populations + 1) {
        let pop = PopulationBuilder::<Queens>::new()
            .set_id(i)
            .initial_population(&initial_population)
            .mutation_rate((1..10).cycle().take(individuals as usize).collect())
            .reset_limit_end(0) // disable the resetting of all individuals
            .finalize().unwrap();

        result.push(pop);
    }

    result
}

// implement trait functions mutate and calculate_fitness:
impl Individual for Queens {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        let mut index1: usize = rng.gen_range(0, self.board.len());
        let mut index2: usize = rng.gen_range(0, self.board.len());

        // Pick a position where a queen is placed.
        // Try random position until it finds a queen
        while self.board[index1] != 1 {
            index1 = rng.gen_range(0, self.board.len());
        }

        // Pick a position where no queen is placed and this index is different from the other
        while (index1 == index2) && (self.board[index2] != 0) {
            index2 = rng.gen_range(0, self.board.len());
        }

        // Move queen onto an empty position
        self.board.swap(index1, index2);
    }

    // fitness means here: how many queens are colliding
    fn calculate_fitness(&mut self) -> f64 {
        let mut num_of_collisions = 0;

        for row in 0..8 {
            for col in 0..8 {
                // Found a queen, so check for collisions
                if self.board[(row * 8) + col] == 1 {
                    num_of_collisions += find_collisions(&self.board, row, col);
                }
            }
        }

        num_of_collisions as f64
    }

    fn reset(&mut self) {
        self.board = vec![
            1,1,1,1,1,1,1,1,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
        ];
    }
}

fn main() {
    println!("Darwin test: queens problem");

    let _ = SimpleLogger::init(LogLevelFilter::Info, Config::default());

    let queens = SimulationBuilder::<Queens>::new()
        .fitness(0.0)
        .threads(2)
        .add_multiple_populations(make_all_populations(100, 8))
        .finalize();

    match queens {
        Err(simulation_builder::Error(simulation_builder::ErrorKind::EndIterationTooLow, _)) => println!("more than 10 iteratons needed"),
        Err(e) => println!("unexpected error: {}", e),
        Ok(mut queens_simulation) => {
            queens_simulation.run();

            // A fitness of zero means a solution was found.
            queens_simulation.print_fitness();

            // print solution
            for row in 0..8 {
                for col in 0..8 {
                    print!("{} | ",
                           queens_simulation.simulation_result.fittest[0].individual.board[(row * 8) + col]);
                }
                println!("\n");
            }

            println!("total run time: {} ms", queens_simulation.total_time_in_ms);
            println!("improvement factor: {}",
                     queens_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}",
                     queens_simulation.simulation_result.iteration_counter);

        }
    }
}
