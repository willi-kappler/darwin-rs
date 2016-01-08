extern crate rand;

// internal crates
extern crate darwin_rs;

use rand::Rng;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, BuilderResult};

fn initialize_sudoku() -> Vec<(Sudoku, u32)> {
    let mut result = Vec::new();

    for i in 0..20 {
        let mut sudoku = Sudoku {
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
            solved: Vec::new()
        };

        sudoku.solved = sudoku.original.clone();

        result.push((
            sudoku,
            if i < 5 { 1 } else { 20 }
        ))}

    result
}

// A cell is a 3x3 sub field inside the 9x9 sudoku field
fn fittness_of_one_cell(sudoku: &Vec<u8>, row: usize, col: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for i in 0..3 {
        for j in 0..3 {
            let data = sudoku[(((row + i) * 9) + col + j) as usize];

            if data >= 1 && data <= 9 {
                number_occurence[(data - 1) as usize] = number_occurence[(data - 1) as usize] + 1;
            } else {
                error = error + 1.0;
            }
        }
    }

    for number in number_occurence {
        if number != 1 {
            error = error + 1.0;
        }
    }

    error
}

fn fittness_of_one_row(sudoku: &Vec<u8>, row: usize) -> f64 {
    let mut number_occurence = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut error = 0.0;

    for col in 0..8 {
        let number = sudoku[(row * 9) + col];
        if number > 0 && number < 10 {
            number_occurence[(number - 1) as usize] = number_occurence[(number - 1) as usize] + 1;
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
            number_occurence[(number - 1) as usize] = number_occurence[(number - 1) as usize] + 1;
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

        let index: usize = rng.gen_range(0, self.original.len());

        if self.original[index] == 0 {
            self.solved[index] = rng.gen_range(1, 10);
        }
    }

    // fittness means here: how many errors
    fn calculate_fittness(&self) -> f64 {
        let mut result = 0.0;

        for i in 0..9 {
            result = result + fittness_of_one_row(&self.solved, i);
            result = result + fittness_of_one_col(&self.solved, i);
        }

        let mut i = 0;
        let mut j = 0;

        loop {
            result = result + fittness_of_one_cell(&self.solved, i, j);

            i = i + 3;
            if i >= 9 {
                i = 0;
                j = j + 3;
                if j >= 9 {
                    break;
                }
            }
        }

        result
    }
}

fn main() {
    println!("Darwin test: sudoku solver");

    let sudoku_builder = SimulationBuilder::<Sudoku>::new()
        .iterations(1000000)
        .threads(1)
        .global_fittest(true)
        .initial_population_num_mut(initialize_sudoku())
        .increasing_mutation_rate()
        .finalize();

    match sudoku_builder {
        BuilderResult::LowIterration => { println!("more than 10 iteratons needed") },
        BuilderResult::LowIndividuals => { println!("more than 2 individuals needed") },
        BuilderResult::Ok(mut sudoku_simulation) => {
            sudoku_simulation.run();

            println!("total run time: {} ms", sudoku_simulation.total_time_in_ms);
            println!("improvement factor: {}", sudoku_simulation.improvement_factor);

            sudoku_simulation.print_fittness();
        }
    }

}
