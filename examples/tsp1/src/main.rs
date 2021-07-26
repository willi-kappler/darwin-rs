

use darwin_rs::{SimulationNode, SimulationServer, Individual, Method};

use nanorand::{Rng, WyRand};

// let mut rng = WyRand::new();
// rng.generate::<u64>();

pub struct TSP {
    cities: Vec<(f64, f64)>,
    rng: WyRand,
}

impl TSP {
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
            rng: WyRand::new(),
        }
    }
}

impl Individual for TSP {
    fn mutate(&mut self) {
        let index1 = self.rng.generate::<usize>();
        let mut index2 = self.rng.generate::<usize>();

        while index1 == index2 {
            index2 = self.rng.generate::<usize>();
        }

        self.cities.swap(index1, index2);
    }
    fn calculate_fitness(&self) -> f64 {
        let mut distance = 0.0;

        let (mut px, mut py) = self.cities.last().unwrap();

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

}
