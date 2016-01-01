extern crate rand;

// internal crates
extern crate darwin_rs;

use rand::Rng;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, BuilderResult};

fn initialize_sudoku() -> Sudoku {
    Sudoku {
        original: vec![
            5,3,0,   0,7,0,   0,0,0,
            6,0,0,   1,9,5,   0,0,0,
            0,9,8,   0,0,0,   0,6,0,

            8,0,0,   0,6,0,   0,0,3,
            4,0,0,   8,0,3,   0,0,1,
            7,0,0,   0,2,0,   0,0,6,

            0,6,0,   0,0,0,   2,8,0,
            0,0,0,   4,1,9,   0,0,5,
            0,0,0,   0,8,0,   0,7,9
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

// A cell is a 3x3 sub field inside the 9x9 sudoku field
fn fitness_of_one_cell(sudoku: &Vec<u8>, row: usize, col: usize) -> f64 {
    let number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    error
}

fn fittness_of_one_row(sudoku: &Vec<u8>, row: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for col in 0..8 {
        let number = sudoku[(row * 9) + col];
        if number > 0 && number < 10 {
            number_occurence[number as usize] = number_occurence[number as usize] + 1;
        } else {
            error = error + 1.0;
        }
    }

    // Each number must be unique, otherwise increase error
    for number in number_occurence {
        if number != 1 { error = error + 1.0; }
    }

    error
}

fn fittness_of_one_col(sudoku: &Vec<u8>, col: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for row in 0..8 {
        let number = sudoku[(row * 9) + col];
        if number > 0 && number < 10 {
            number_occurence[number as usize] = number_occurence[number as usize] + 1;
        } else {
            error = error + 1.0;
        }
    }

    // Each number must be unique, otherwise increase error
    for number in number_occurence {
        if number != 1 { error = error + 1.0; }
    }

    error
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
