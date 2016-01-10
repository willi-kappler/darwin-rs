extern crate rand;

// internal crates
extern crate darwin_rs;

use rand::Rng;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, BuilderResult};

fn initialize_tsp() -> Vec<(CityItem, u32)> {
    let city_positions = Box::new(vec![
        (2.852197810188428, 90.31966506130796),
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
        (55.760857794890796, 26.95947234362994)
    ]);

    let mut path : Vec<usize> = (0..city_positions.len()).map(|x| x as usize).collect();
    path.push(0); // Add start position to end of path

    let mut result = Vec::new();

    for i in 0..20 {
        result.push((
            CityItem {
                city_positions: city_positions.clone(),
                path: path.clone()
            },
            (i % 3) + 1
        ));
    }

    result
}

fn city_distance(city: &Vec<(f64, f64)>, index1: usize, index2: usize) -> f64 {
    let (x1, y1) = city[index1];
    let (x2, y2) = city[index2];
    let x = x2 - x1;
    let y = y2 - y1;

    x.hypot(y)
}

#[derive(Debug, Clone)]
struct CityItem {
    city_positions: Box<Vec<(f64, f64)>>,
    path: Vec<usize>
}

// implement trait functions mutate and calculate_fittness:
impl Individual for CityItem {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        // Keep stating position always the same:
        let index1: usize = rng.gen_range(1, self.city_positions.len());
        let mut index2: usize = rng.gen_range(1, self.city_positions.len());

        while index1 == index2 {
            index2 = rng.gen_range(1, self.city_positions.len());
        }

        self.path.swap(index1, index2);
    }

    // fittness means here: the length of the route
    fn calculate_fittness(&self) -> f64 {
        let mut prev_index = &(self.city_positions.len() - 1);
        let mut length : f64 = 0.0;

        for index in &self.path {
            length = length + city_distance(&self.city_positions, *prev_index, *index);

            prev_index = index;
        }

        length
    }
}

fn main() {
    println!("Darwin test: traveling salesman problem");

    let tsp_builder = SimulationBuilder::<CityItem>::new()
        //.iterations(1000000)
        .factor(0.35)
        .threads(2)
        .global_fittest()
        .initial_population_num_mut(initialize_tsp())
        .finalize();

    match tsp_builder {
        BuilderResult::LowIterration => { println!("more than 10 iteratons needed") },
        BuilderResult::LowIndividuals => { println!("more than 2 individuals needed") },
        BuilderResult::Ok(mut tsp_simulation) => {
            tsp_simulation.run();

            println!("total run time: {} ms", tsp_simulation.total_time_in_ms);
            println!("improvement factor: {}", tsp_simulation.improvement_factor);
            println!("number of iterations: {}", tsp_simulation.iteration_counter);

            tsp_simulation.print_fittness();

            println!("Path and coordinates: ");

            let cities = &tsp_simulation.population[0].individual.city_positions;
            for index in tsp_simulation.population[0].individual.path.iter() {
                let (x, y) = cities[*index];
                println!("{} {}", x, y);
            }
        }
    }
}
