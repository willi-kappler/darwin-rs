extern crate time;
extern crate jobsteal;

// external modules
use time::precise_time_ns;
use jobsteal::{make_pool, Pool, IntoSplitIterator, SplitIterator};
use std::cmp::Ordering;

#[derive(Debug,Clone)]
pub enum SimulationType {
    EndIteration(u32),
    EndFittness(f64),
    EndFactor(f64)
}

pub struct Simulation<T: Individual + Send + Sync> {
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
    pub pool: Pool,
}

fn mutate_population<T: Individual + Send + Sync>(simulation: &mut Simulation<T>) {
    (&mut simulation.population).into_split_iter().for_each(
        &simulation.pool.spawner(), |wrapper|
        {
            for _ in 0..wrapper.num_of_mutations {
                wrapper.individual.mutate();
            }
            wrapper.fittness = wrapper.individual.calculate_fittness();
        }
    );
}

fn run_body_sorting_fittest<T: Individual + Send + Sync + Clone>(simulation: &mut Simulation<T>) {
    // Keep original population
    let orig_population = simulation.population.clone();

    // Mutate population
    mutate_population(simulation);

    // Append original (unmutated) population to new (mutated) population
    simulation.population.extend(orig_population.iter().cloned());

    // Sort by fittness
    simulation.population.sort();

    // Reduce population to original length
    simulation.population.truncate(orig_population.len());

    // Restore original number of mutation rate, since these get overwritten by .clone()
    for i in 0..simulation.population.len() {
        simulation.population[i].num_of_mutations = orig_population[i].num_of_mutations;
    }

    // Check if we have new fittest individual and store it globally
    if simulation.population[0].fittness < simulation.fittest.fittness {
        simulation.fittest = simulation.population[0].clone();
        if simulation.output_new_fittest {
            println!("{}: new fittest: {}", simulation.iteration_counter, simulation.fittest.fittness);
        }
    }

    simulation.improvement_factor = simulation.fittest.fittness / simulation.original_fittness;
}

impl<T: Individual + Send + Sync + Clone> Simulation<T> {
    pub fn run(&mut self) {
        let start_time = precise_time_ns();

        self.original_fittness = self.population[0].individual.calculate_fittness();

        // Initialize
        self.iteration_counter = 0;
        self.pool = make_pool(self.num_of_threads).unwrap();

        match self.type_of_simulation {
            SimulationType::EndIteration(end_iteration) => {
                for i in 0..end_iteration {
                    run_body_sorting_fittest(self);
                    self.iteration_counter = i
                }
            },
            SimulationType::EndFactor(end_factor) => {
                loop {
                    if self.improvement_factor <= end_factor { break }
                    run_body_sorting_fittest (self);
                    self.iteration_counter = self.iteration_counter + 1;
                }
            },
            SimulationType::EndFittness(end_fittness) => {
                loop {
                    if self.fittest.fittness <= end_fittness { break }
                    run_body_sorting_fittest(self);
                    self.iteration_counter = self.iteration_counter + 1;
                }
            }
        }

        let end_time = precise_time_ns();

        self.total_time_in_ms = ((end_time - start_time) as f64) / (1000.0 * 1000.0);
    }

    pub fn print_fittness(&self) {
        for wrapper in self.population.iter() {
            println!("fittness: {}, num_of_mutations: {}", wrapper.fittness, wrapper.num_of_mutations);
        }
    }
}

#[derive(Debug,Clone)]
pub struct IndividualWrapper<T: Individual> {
    pub individual: T,
    fittness: f64,
    num_of_mutations: u32
}

impl<T: Individual> PartialEq for IndividualWrapper<T> {
    fn eq(&self, other: &IndividualWrapper<T>) -> bool {
        self.fittness == other.fittness
    }
}

impl<T: Individual> Eq for IndividualWrapper<T>{}

impl<T: Individual> Ord for IndividualWrapper<T> {
    fn cmp(&self, other: &IndividualWrapper<T>) -> Ordering {
        if self.fittness < other.fittness { Ordering::Less }
        else if self.fittness > other.fittness { Ordering::Greater }
        else { Ordering::Equal }
    }
}

impl<T: Individual> PartialOrd for IndividualWrapper<T> {
    fn partial_cmp(&self, other: &IndividualWrapper<T>) -> Option<Ordering> {
        if self.fittness < other.fittness { Some(Ordering::Less) }
        else if self.fittness > other.fittness { Some(Ordering::Greater) }
        else { Some(Ordering::Equal) }
    }
}

pub trait Individual {
    fn new() -> Self;
    fn mutate(&mut self);
    fn calculate_fittness(&self) -> f64;
}

pub struct SimulationBuilder<T: Individual + Send + Sync> {
    simulation: Simulation<T>
}

pub enum BuilderResult<T: Individual + Send + Sync> {
        TooLowEndIterration,
        TooLowIndividuals,
        Ok(Simulation<T>)
}

impl<T: Individual + Send + Sync> SimulationBuilder<T> {
    pub fn new() -> SimulationBuilder<T> {
        SimulationBuilder {
            simulation: Simulation {
                type_of_simulation: SimulationType::EndIteration(10),
                num_of_individuals: 0,
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
                pool: make_pool(4).unwrap(),
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

    pub fn increasing_mutation_rate(mut self) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in self.simulation.population.iter_mut() {
            wrapper.num_of_mutations = mutation_rate;
            mutation_rate = mutation_rate + 1;
        }

        self
    }

    pub fn increasing_exp_mutation_rate(mut self, base: f64) -> SimulationBuilder<T> {
        let mut mutation_rate = 1;

        for wrapper in self.simulation.population.iter_mut() {
            wrapper.num_of_mutations = base.powi(mutation_rate).floor() as u32;
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
            pool: self.simulation.pool,
        };

        if self.simulation.num_of_individuals < 3 { return BuilderResult::TooLowIndividuals }

        if let SimulationType::EndIteration(end_iteration) = self.simulation.type_of_simulation {
            if end_iteration < 10 { return BuilderResult::TooLowEndIterration }
        }

        BuilderResult::Ok(result)
    }
}
