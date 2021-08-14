

use darwin_rs::{DWNode, DWServer, DWIndividual, NCConfiguration, DWConfiguration};

use rand::{thread_rng, Rng};
use structopt::StructOpt;
use simplelog::{WriteLogger, LevelFilter, ConfigBuilder};
use serde::{Serialize, Deserialize};

use std::fs;

#[derive(StructOpt, Debug)]
#[structopt(name = "tsp1")]
pub struct TSP1Opt {
    #[structopt(short = "s", long = "server")]
    server: bool,
    #[structopt(long = "ip", default_value = "127.0.0.1")]
    ip: String,
    #[structopt(short = "p", long = "port", default_value = "2020")]
    port: u16,
    #[structopt(short = "o", long = "population", default_value = "20")]
    population: usize,
    #[structopt(short = "l", long = "limit", default_value = "1.0")]
    limit: f64,
    #[structopt(short = "i", long = "iter", default_value = "1000")]
    num_of_iterations: u64,
    #[structopt(short = "m", long = "mutate", default_value = "10")]
    num_of_mutations: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TSP1 {
    cities: Vec<(f64, f64)>,
}

impl TSP1 {
    pub fn new() -> Self {
        Self {
            cities: vec![(2.852197810188428, 90.31966506130796),
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
                        (55.760857794890796, 26.95947234362994)],
        }
    }
}

impl DWIndividual for TSP1 {
    fn mutate(&mut self) {
        let mut rng = thread_rng();
        let last = self.cities.len();
        let index1 = rng.gen_range(1_usize..last);
        let mut index2 = rng.gen_range(1_usize..last);

        while index1 == index2 {
            index2 = rng.gen_range(1_usize..last);
        }

        // This is just a very simple and dumb mutation...
        self.cities.swap(index1, index2);
    }
    fn calculate_fitness(&self) -> f64 {
        let mut distance = 0.0;
        let last = self.cities.len() - 1;

        let (mut px, mut py) = self.cities[last];

        for (x, y) in self.cities.iter() {
            let dx = *x - px;
            let dy = *y - py;

            distance += dx.hypot(dy);

            px = *x;
            py = *y;
        }

        distance
    }
}

fn main() {
    let options = TSP1Opt::from_args();
    let tsp1 = TSP1::new();

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
        ..Default::default()
    };

    let log_level = LevelFilter::Debug;
    let log_config = ConfigBuilder::new().set_time_format_str("%Y.%m.%d %H:%M:%S").build();

    if options.server {
        let log_file = fs::File::create("server.log").unwrap();
        WriteLogger::init(log_level, log_config, log_file).unwrap();

        let server = DWServer::new(tsp1, dw_configuration, nc_configuration);
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

        let node = DWNode::new(tsp1, dw_configuration, nc_configuration);
        node.run();
    }
}
