extern crate rand;

// internal crates
extern crate darwin_rs;

use rand::Rng;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, BuilderResult};

fn initialize_sudoku() -> Sudoku {
    Sudoku {
        original: vec![
            1,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,

            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,

            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0
        ],
        solved: vec![
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,

            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,

            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0,
            0,0,0,   0,0,0,   0,0,0
        ]
    }
}

fn fitness_of_one_cell(sudoku: &Vec<u8>, cell: u8) -> f64 {
    0.0
}

fn fittness_of_one_row(sudoku: &Vec<u8>, row: u8) -> f64 {
    0.0
}

fn fittness_of_one_col(sudoku: &Vec<u8>, col: u8) -> f64 {
    0.0
}

#[derive(Debug, Clone)]
struct Sudoku {
    original: Vec<u8>,
    solved: Vec<u8>
}



// implement trait functions mutate and calculate_fittness:
impl Individual for Sudoku {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
    }

    // fittness means here: how many errors
    fn calculate_fittness(&self) -> f64 {
        0.0
    }
}





fn main() {
    println!("Darwin test: sudoku solver");

    let sudoku_builder = SimulationBuilder::<Sudoku>::new()
        .iterations(10000)
        .one_individual(initialize_sudoku())
        .finalize();

    match sudoku_builder {
        BuilderResult::LowIterration => { println!("more than 10 iteratons needed") },
        BuilderResult::LowIndividuals => { println!("more than 2 individuals needed") },
        BuilderResult::Ok(mut sudoku_simulation) => {
            let total_run_time = sudoku_simulation.run();

            println!("total run time: {} ms", total_run_time);
            println!("improvement factor: {}", sudoku_simulation.improvement_factor);

            sudoku_simulation.print_fittness();
        }
    }

}
