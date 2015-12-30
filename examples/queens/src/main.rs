extern crate rand;

// internal crates
extern crate darwin_rs;

use rand::Rng;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, BuilderResult};

fn initialize_queens() -> Queens {
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
}

struct Queens {
    board: Vec<u8>
}

// implement trait functions mutate and calculate_fittness:
impl Individual for Queens {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        // Keep stating position always the same:
        let mut index1: usize = rng.gen_range(1, self.board.len());
        let mut index2: usize = rng.gen_range(1, self.board.len());

        // Pick a position where a qeen is placed.
        // Try random position until it finds a queen
        while self.board[index1] != 1 {
            index1 = rng.gen_range(1, self.board.len());
        }

        // Pick a position where no queen is placed and this index is different from the other
        while (index1 == index2) && (self.board[index2] != 0) {
            index2 = rng.gen_range(1, self.board.len());
        }

        // Place queen onto an empty position
        self.path.swap(index1, index2);
    }

    // fittness means here: how many queens are colliding
    fn calculate_fittness(&self) -> f64 {
        let mut num_of_collisions = 0;
        let mut num_of_queens = 0;

        let sx = 0;
        let sy = 0;

        for row in 0..8 {
            num_of_queens = 0;
            for col in 0..8 {
                if self.borad[(row * 8) + col] == 1 {
                    num_of_queens = num_of_queens + 1;
                }

                if num_of_queens > 1 {
                    num_of_collisions = num_of_collisions + 1;
                }
            }
        }

        for col in 0..8 {
            num_of_queens = 0;
            for row in 0..8 {
                if self.borad[(row * 8) + col] == 1 {
                    num_of_queens = num_of_queens + 1;
                }

                if num_of_queens > 1 {
                    num_of_collisions = num_of_collisions + 1;
                }
            }
        }

        // TODO: check diagonals

        num_of_collisions as f64
    }
}


fn main() {
    println!("Darwin test: queens problem");

    let queens_builder = SimulationBuilder::<Queens>::new()
        .iterations(10000)
        .one_individual(initialize_queens())
        .finalize();

    match queens_builder {
        BuilderResult::LowIterration => { println!("more than 10 iteratons needed") },
        BuilderResult::LowIndividuals => { println!("more than 2 individuals needed") },
        BuilderResult::Ok(mut queens_simulation) => {
            let total_run_time = queens_simulation.run();

            println!("total run time: {} ms", total_run_time);
            println!("improvement factor: {}", queens_simulation.improvement_factor);

            queens_simulation.print_fittness();
        }
    }

}
