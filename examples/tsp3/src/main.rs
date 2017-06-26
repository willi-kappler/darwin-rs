// This example implements the tsp (travelling salesman problem) problem:
// https://en.wikipedia.org/wiki/Travelling_salesman_problem
// using an evolutionary algorithm.
//
// Note that evolutionary algorithms do no guarantee to always find the optimal solution.
// But they can get very close

extern crate rand;
extern crate simplelog;
#[macro_use] extern crate clap;

// Internal crates
extern crate darwin_rs;

use std::sync::Arc;
use rand::Rng;
use simplelog::{SimpleLogger, LogLevelFilter, Config};
use clap::{Arg, App};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

// Internal modules
use darwin_rs::{Individual, SimulationBuilder, Population, PopulationBuilder, simulation_builder};

fn city_distance(city: &[(f64, f64)], index1: usize, index2: usize) -> f64 {
    let (x1, y1) = city[index1];
    let (x2, y2) = city[index2];
    let x = x2 - x1;
    let y = y2 - y1;

    x.hypot(y)
}

fn make_population(count: u32, cities: &Vec<(f64, f64)>) -> Vec<CityItem> {
    let mut result = Vec::new();

    let shared = Arc::new(cities.clone());

    let mut path: Vec<usize> = (0..cities.len()).map(|x| x as usize).collect();
    path.push(0); // Add start position to end of path

    for _ in 0..count {
        result.push( CityItem {
                path: path.clone(),
                cities: shared.clone()
            }
        );
    }

    result
}

fn make_all_populations(individuals: u32, populations: u32, cities: &Vec<(f64, f64)>) -> Vec<Population<CityItem>> {
    let mut result = Vec::new();

    let initial_population = make_population(individuals, &cities);

    for i in 1..(populations + 1) {
        let reset_limit_end = if i == 1 { 0 } else { 1000 * i };

        let pop = PopulationBuilder::<CityItem>::new()
            .set_id(i)
            .initial_population(&initial_population)
            .increasing_exp_mutation_rate(1.1)
            .reset_limit_increment(100 * i)
            .reset_limit_start(100 * i)
            .reset_limit_end(reset_limit_end)
            .finalize().unwrap();

        result.push(pop)
    }

    result
}

#[derive(Debug, Clone)]
struct CityItem {
    path: Vec<usize>,
    cities: Arc<Vec<(f64, f64)>>
}

// Implement trait functions mutate and calculate_fitness:
impl Individual for CityItem {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        // Keep stating position always the same: (random numbers from 1, not 0)
        let index1: usize = rng.gen_range(1, self.cities.len());
        let mut index2: usize = rng.gen_range(1, self.cities.len());

        // Small optimisation
        while index1 == index2 {
            index2 = rng.gen_range(1, self.cities.len());
        }

        // Compared to examples/tsp/ here we add a second operation:
        // Additionaly to swaping indices we also rotate (shift) items around.
        // And just by adding this second mutation operation, the resulst converge
        // much faster to the optimum.
        // You can add a third operation her if you want (for ex. mirrorig), or
        // try to leave the swap opersion out, just to see if it runs better.

        // Choose mutate operation
        let operation: u8 = rng.gen_range(0, 2);

        match operation {
            0 => {
                // Just swap two positions
                self.path.swap(index1, index2);
            },
            1 => {
                // Rotate (shift) items
                let tmp = self.path.remove(index1);
                self.path.insert(index2, tmp);
            },
            2 => {
                // Add your new operation here, for ex. mirror between index1 and index2:

            },
            _ => println!("unknown operation: {}", operation),
        }
    }

    // fitness means here: the length of the route, the shorter the better
    fn calculate_fitness(&mut self) -> f64 {
        let mut prev_index = &(self.cities.len() - 1);
        let mut length: f64 = 0.0;

        for index in &self.path {
            length += city_distance(&self.cities, *prev_index, *index);

            prev_index = index;
        }

        // Seconds, Nanoseconds
        // sleep(Duration::new(0, 100000));

        length
    }

    fn reset(&mut self) {
        let mut path: Vec<usize> = (0..self.cities.len()).map(|x| x as usize).collect();
        path.push(0); // Add start position to end of path

        self.path = path;
    }
}

fn load_cities(file_name: Option<&str>) -> Vec<(f64, f64)> {
    if let Some(file_name) = file_name {
        // NODE_COORD_SECTION
        // 1 12183.3333 52233.3333
        // EOF

        let f = File::open(file_name).unwrap();
        let f = BufReader::new(f);

        let mut inside_data_section = false;
        let mut result = Vec::new();

        // Read in coordinates liny by line from a .tsp file
        // The file format is specified here:
        //
        for line in f.lines() {
            let line = line.unwrap();

            if line.starts_with("NODE_COORD_SECTION") { inside_data_section = true }
            else if line.starts_with("EOF") { inside_data_section = false }
            else {
                if inside_data_section {
                    let line_items: Vec<&str> = line.split(char::is_whitespace).collect();
                    result.push((line_items[1].parse().unwrap(), line_items[2].parse().unwrap()));
                }
            }
        }

        result
    } else {
        // Default coordinates of cities
        vec![(2.852197810188428, 90.31966506130796),
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
              (55.760857794890796, 26.95947234362994)]
    }
}

fn main() {
    println!("Darwin test: traveling salesman problem");

    let _ = SimpleLogger::init(LogLevelFilter::Info, Config::default());

    let matches = App::new("My Super Program")
        .version("0.1")
        .author("Willi Kappler")
        .about("Solves TSP (traveling salesman problem)")
        .arg(Arg::with_name("input_file")
            .short("f")
            .help("Sets the input file (default: built in coordinates)")
            .takes_value(true))
        .arg(Arg::with_name("end_fitness")
            .short("e")
            .help("Sets the end fitness criteria (default: 500.0)")
            .takes_value(true))
        .arg(Arg::with_name("num_of_threads")
            .short("t")
            .help("Sets the number of threads to use (default: 4)")
            .takes_value(true))
        .arg(Arg::with_name("num_of_populations")
            .short("p")
            .help("Sets the number of populations to use (default: 10)")
            .takes_value(true))
        .arg(Arg::with_name("num_of_individuals")
            .short("i")
            .help("Sets the number of individuals to use (default: 100)")
            .takes_value(true))
        .get_matches();

    let cities = load_cities(matches.value_of("input_file"));

    let end_fitness = value_t!(matches.value_of("end_fitness"), f64).unwrap_or(1000.0);
    let num_of_threads = value_t!(matches.value_of("num_of_threads"), usize).unwrap_or(8);
    let num_of_populations = value_t!(matches.value_of("num_of_populations"), u32).unwrap_or(32);
    let num_of_individuals = value_t!(matches.value_of("num_of_individuals"), u32).unwrap_or(20);


    let tsp = SimulationBuilder::<CityItem>::new()
          // .factor(0.34)
          .fitness(end_fitness)
          .threads(num_of_threads)
          .add_multiple_populations(make_all_populations(num_of_individuals, num_of_populations, &cities))
          .share_fittest()
          .output_every(100)
          .finalize();


    match tsp {
        Err(simulation_builder::Error(simulation_builder::ErrorKind::EndIterationTooLow, _)) => println!("more than 10 iteratons needed"),
        Err(e) => println!("unexpected error: {}", e),
        Ok(mut tsp_simulation) => {
            tsp_simulation.run();

            tsp_simulation.print_fitness();

            println!("Path and coordinates: ");

            for index in &tsp_simulation.simulation_result.fittest[0].individual.path {
                let (x, y) = cities[*index];
                println!("{} {}", x, y);
            }

            println!("total run time: {} ms", tsp_simulation.total_time_in_ms);
            println!("improvement factor: {}",
                tsp_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}",
                tsp_simulation.simulation_result.iteration_counter);
        }
    }
}
