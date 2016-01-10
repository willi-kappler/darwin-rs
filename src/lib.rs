extern crate time;
extern crate simple_parallel;
extern crate rand;

// external modules
use time::precise_time_ns;
use simple_parallel::Pool;
use rand::Rng;

#[derive(Debug,Clone)]
pub enum SimulationType {
    EndIteration(u32),
    EndFittness(f64),
    EndFactor(f64)
}

#[derive(Debug,Clone,PartialEq)]
pub enum FittestType {
    GlobalFittest,
    LocalFittest,
    RandomFittest
}

pub struct Simulation<T: 'static + Individual + Send> {
    pub type_of_simulation: SimulationType,
    pub num_of_individuals: u32,
    pub num_of_threads: usize,
    pub improvement_factor: f64,
    pub original_fittness: f64,
    pub fittest: IndividualWrapper<T>,
    pub population: Vec<IndividualWrapper<T>>,
    pub total_time_in_ms: f64,
    pub iteration_counter: u32,
    pub output_new_fittest: bool,
    pub random_fittest: u32,
    pub type_of_fittest: FittestType,
    pub pool: Pool,
    pub run_body: Box<Fn(&mut Simulation<T>)>
}

fn find_fittest<T: Individual + Clone + Send>(simulation: &mut Simulation<T>) {
    for wrapper in simulation.population.iter() {
        if wrapper.fittness < simulation.fittest.fittness {
            simulation.fittest = wrapper.clone();
            if simulation.output_new_fittest {
                println!("new fittest: {}", simulation.fittest.fittness);
            }
        }
    }
}

fn mutate_population<T: Individual + Clone + Send>(simulation: &mut Simulation<T>) {
    simulation.pool.for_(simulation.population.iter_mut(), |wrapper|
        {
            for _ in 0..wrapper.num_of_mutations {
                wrapper.individual.mutate();
            }
            wrapper.fittness = wrapper.individual.calculate_fittness();
        }
    );
}

fn run_body_global_fittest<T: Individual + Clone + Send>(simulation: &mut Simulation<T>) {
    mutate_population(simulation);

    // Find fittest individual for whole simulation...
    find_fittest(simulation);

    simulation.improvement_factor = simulation.fittest.fittness / simulation.original_fittness;

    // ...  and copy it to the others (except the last one, to avoid local minimum or maximum)
    for i in 0..(simulation.population.len() - 1) {
        simulation.population[i].individual = simulation.fittest.individual.clone();
    }

    // Set fittness of first individual, since population vector will be sorted (by fittness) after the loop
    simulation.population[0].fittness = simulation.fittest.fittness;
}

fn run_body_local_fittest<T: Individual + Clone + Send>(simulation: &mut Simulation<T>) {
    simulation.fittest = simulation.population[0].clone();

    mutate_population(simulation);

    // Find fittest individual only for this function call...
    find_fittest(simulation);

    simulation.improvement_factor = simulation.fittest.fittness / simulation.original_fittness;

    // ...  and copy it to the others (except the last one, to avoid local minimum or maximum)
    for i in 0..(simulation.population.len() - 1) {
        simulation.population[i].individual = simulation.fittest.individual.clone();
    }

    // Set fittness of first individual, since population vector will be sorted (by fittness) after the loop
    simulation.population[0].fittness = simulation.fittest.fittness;
}

fn run_body_random_fittest<T: Individual + Clone + Send>(simulation: &mut Simulation<T>) {
    mutate_population(simulation);

    // Find fittest individual for whole simulation...
    find_fittest(simulation);

    simulation.improvement_factor = simulation.fittest.fittness / simulation.original_fittness;

    // ... and choose some random individual to set it back to the fittest
    let mut rng = rand::thread_rng();

    for _ in 0..simulation.random_fittest {
        let index: usize = rng.gen_range(0, simulation.population.len());
        simulation.population[index].individual = simulation.fittest.individual.clone();
    }
}

impl<T: Individual + Clone + Send> Simulation<T> {
    pub fn run(&mut self) {
        let start_time = precise_time_ns();

        self.original_fittness = self.population[0].individual.calculate_fittness();

        // Initialize
        let mut iteration_counter = 0;
        self.pool = simple_parallel::Pool::new(self.num_of_threads);

        match self.type_of_simulation {
            SimulationType::EndIteration(end_iteration) => {
                match self.type_of_fittest {
                    FittestType::GlobalFittest => {
                        for _ in 0..end_iteration {
                            run_body_global_fittest(self);
                        }
                    },
                    FittestType::LocalFittest => {
                        for _ in 0..end_iteration {
                            run_body_local_fittest(self);
                        }
                    },
                    FittestType::RandomFittest => {
                        for _ in 0..end_iteration {
                            run_body_random_fittest(self);
                        }
                    }
                }

                iteration_counter = end_iteration;
            },
            SimulationType::EndFactor(end_factor) => {
                match self.type_of_fittest {
                    FittestType::GlobalFittest => {
                        loop {
                            if self.improvement_factor <= end_factor { break }
                            run_body_global_fittest (self);
                            iteration_counter = iteration_counter + 1;
                        }
                    },
                    FittestType::LocalFittest => {
                        loop {
                            if self.improvement_factor <= end_factor { break }
                            run_body_local_fittest(self);
                            iteration_counter = iteration_counter + 1;
                        }
                    },
                    FittestType::RandomFittest => {
                        loop {
                            if self.improvement_factor <= end_factor { break }
                            run_body_random_fittest (self);
                            iteration_counter = iteration_counter + 1;
                        }
                    }
                }
            },
            SimulationType::EndFittness(end_fittness) => {
                match self.type_of_fittest {
                    FittestType::GlobalFittest => {
                        loop {
                            if self.fittest.fittness <= end_fittness { break }
                            run_body_global_fittest(self);
                            iteration_counter = iteration_counter + 1;
                        }
                    },
                    FittestType::LocalFittest => {
                        loop {
                            if self.fittest.fittness <= end_fittness { break }
                            run_body_local_fittest(self);
                            iteration_counter = iteration_counter + 1;
                        }
                    },
                    FittestType::RandomFittest => {
                        loop {
                            if self.fittest.fittness <= end_fittness { break }
                            run_body_random_fittest(self);
                            iteration_counter = iteration_counter + 1;
                        }
                    }
                }
            }
        }

        // sort all individuals by fittness
        self.population.sort_by(|a, b| a.fittness.partial_cmp(&b.fittness).unwrap());

        let end_time = precise_time_ns();

        self.total_time_in_ms = ((end_time - start_time) as f64) / (1000.0 * 1000.0);
        self.iteration_counter = iteration_counter;
    }

    pub fn print_fittness(&self) {
        for wrapper in self.population.iter() {
            println!("fittness: {}", wrapper.fittness);
        }
    }
}

#[derive(Debug,Clone)]
pub struct IndividualWrapper<T: Individual> {
    pub individual: T,
    fittness: f64,
    num_of_mutations: u32
}

pub trait Individual {
    fn new() -> Self;
    fn mutate(&mut self);
    fn calculate_fittness(&self) -> f64;
}

pub struct SimulationBuilder<T: 'static + Individual + Send> {
    simulation: Simulation<T>
}

pub enum BuilderResult<T: 'static + Individual + Send> {
        TooLowEndIterration,
        TooLowIndividuals,
        InvalidFittestCount,
        Ok(Simulation<T>)
}

impl<T: Individual + Clone + Send> SimulationBuilder<T> {
    pub fn new() -> SimulationBuilder<T> {
        SimulationBuilder {
            simulation: Simulation {
                type_of_simulation: SimulationType::EndIteration(10),
                num_of_individuals: 10,
                num_of_threads: 2,
                improvement_factor: std::f64::MAX,
                original_fittness: std::f64::MAX,
                fittest: IndividualWrapper {
                    individual: Individual::new(),
                    fittness: std::f64::MAX,
                    num_of_mutations: 1
                },
                population: Vec::new(),
                total_time_in_ms: 0.0,
                iteration_counter: 0,
                output_new_fittest: true,
                random_fittest: 1,
                run_body: Box::new(run_body_global_fittest),
                pool: simple_parallel::Pool::new(2),
                type_of_fittest: FittestType::GlobalFittest
            }
        }
    }

    pub fn iterations(mut self, iterations: u32) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndIteration(iterations);
        self
    }

    pub fn factor(mut self, factor: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFactor(factor);
        self
    }

    pub fn fittness(mut self, fittness: f64) -> SimulationBuilder<T> {
        self.simulation.type_of_simulation = SimulationType::EndFittness(fittness);
        self
    }

    pub fn individuals(mut self, individuals: u32) -> SimulationBuilder<T> {
        self.simulation.num_of_individuals = individuals;

        for _ in 0..individuals {
            self.simulation.population.push(
                IndividualWrapper {
                    individual: Individual::new(),
                    fittness: std::f64::MAX,
                    num_of_mutations: 1
                }
            );
        }

        self
    }

    pub fn threads(mut self, threads: usize) -> SimulationBuilder<T> {
        self.simulation.num_of_threads = threads;
        self
    }

    pub fn output_new_fittest(mut self, output_new_fittest: bool) -> SimulationBuilder<T> {
        self.simulation.output_new_fittest = output_new_fittest;
        self
    }

    pub fn global_fittest(mut self) -> SimulationBuilder<T> {
        self.simulation.type_of_fittest = FittestType::GlobalFittest;
        self.simulation.run_body = Box::new(run_body_global_fittest);
        self
    }

    pub fn local_fittest(mut self) -> SimulationBuilder<T> {
        self.simulation.type_of_fittest = FittestType::LocalFittest;
        self.simulation.run_body = Box::new(run_body_local_fittest);
        self
    }

    pub fn random_fittest(mut self, count: u32) -> SimulationBuilder<T> {
        self.simulation.type_of_fittest = FittestType::RandomFittest;
        self.simulation.random_fittest = count;
        self.simulation.run_body = Box::new(run_body_random_fittest);
        self
    }

    pub fn increasing_mutation_rate(mut self) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in self.simulation.population.iter_mut() {
            wrapper.num_of_mutations = mutation_rate;
            mutation_rate = mutation_rate + 1;
        }

        self
    }

    pub fn mutation_rate(mut self, mutation_rate: Vec<u32>) -> SimulationBuilder<T> {
        // TODO: better error handling
        assert!(self.simulation.population.len() == mutation_rate.len());

        for i in 0..self.simulation.population.len() {
            self.simulation.population[i].num_of_mutations = mutation_rate[i];
        }

        self
    }

    pub fn finalize(self) -> BuilderResult<T> {
        let result = Simulation {
            type_of_simulation: self.simulation.type_of_simulation.clone(),
            num_of_individuals: self.simulation.num_of_individuals,
            num_of_threads: self.simulation.num_of_threads,
            improvement_factor: self.simulation.improvement_factor,
            original_fittness: self.simulation.original_fittness,
            fittest: self.simulation.fittest,
            population: self.simulation.population,
            total_time_in_ms: self.simulation.total_time_in_ms,
            iteration_counter: self.simulation.iteration_counter,
            output_new_fittest: self.simulation.output_new_fittest,
            random_fittest: self.simulation.random_fittest,
            run_body: self.simulation.run_body,
            pool: self.simulation.pool,
            type_of_fittest: self.simulation.type_of_fittest
        };

        if self.simulation.num_of_individuals < 3 { return BuilderResult::TooLowIndividuals }

        if let SimulationType::EndIteration(end_iteration) = self.simulation.type_of_simulation {
            if end_iteration < 10 { return BuilderResult::TooLowEndIterration }
        }

        if result.type_of_fittest == FittestType::RandomFittest {
            if result.random_fittest >= result.num_of_individuals ||
               result.random_fittest == 0 { return BuilderResult::InvalidFittestCount }
        }

        BuilderResult::Ok(result)
    }
}
