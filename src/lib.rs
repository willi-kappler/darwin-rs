extern crate time;
extern crate simple_parallel;

// external modules
use time::precise_time_ns;

#[derive(Debug)]
pub struct Simulation<T: Individual> {
    pub num_of_iterations: u32,
    pub num_of_individuals: u32,
    pub num_of_threads: u32,
    pub improvement_factor: f64,
    pub population: Vec<IndividualWrapper<T>>
}

impl<T: Individual + Clone> Simulation<T> {
    pub fn run(&mut self) -> f64 {
        let start_time = precise_time_ns();

        let original_fittness = self.population[0].individual.calculate_fittness();

        for _ in 0..self.num_of_iterations {
            // TODO: use simple_parallel

            // Merge information about fittest individual back to the rest of the population
            let fittest = self.population[0].clone();

            // No need to set self.population[0] since this is already the fittest
            // No need to set self.population[1] since we want keep one total random individual to
            // escape local minimum or maximum
            for i in 2..self.population.len() {
                self.population[i].individual = fittest.individual.clone();
            }

            // mutate all individuals and recalculate fittness
            for wrapper in self.population.iter_mut() {
                for _ in 0..wrapper.num_of_mutations {
                    wrapper.individual.mutate();
                }
                wrapper.fittness = wrapper.individual.calculate_fittness();
            }

            // sort all individuals by fittness
            self.population.sort_by(|a, b| a.fittness.partial_cmp(&b.fittness).unwrap());

            // If the fittness of the best has decreased, we need correct it:
            if fittest.fittness < self.population[0].fittness {
                self.population[0] = fittest.clone();
            }
        }
        let end_time = precise_time_ns();

        let best_individual = &self.population[0];
        self.improvement_factor = best_individual.fittness / original_fittness;

        let total_time_in_ms = ((end_time - start_time) as f64) / (1000.0 * 1000.0);

        total_time_in_ms
    }

    pub fn print_fittness(&self) {
        for wrapper in self.population.iter() {
            println!("fittness: {}", wrapper.fittness);
        }
    }
}

#[derive(Debug,Clone)]
pub struct IndividualWrapper<T: Individual> {
    individual: T,
    fittness: f64,
    num_of_mutations: u32
}

pub trait Individual {
    fn mutate(&mut self);
    fn calculate_fittness(&self) -> f64;
}

#[derive(Debug)]
pub struct SimulationBuilder<T: Individual> {
    simulation: Simulation<T>
}

pub enum BuilderResult<T: Individual> {
        LowIterration,
        LowIndividuals,
        Ok(Simulation<T>)
}

impl<T: Individual + Clone> SimulationBuilder<T> {
    pub fn new() -> SimulationBuilder<T> {
        SimulationBuilder {
            simulation: Simulation {
                num_of_iterations: 10,
                num_of_individuals: 10,
                num_of_threads: 1,
                improvement_factor: 0.0,
                population: Vec::new()
            }
        }
    }

    pub fn iterations(mut self, iterations: u32) -> SimulationBuilder<T> {
        self.simulation.num_of_iterations = iterations;
        self
    }

    pub fn individuals(mut self, individuals: u32) -> SimulationBuilder<T> {
        self.simulation.num_of_individuals = individuals;
        self
    }

    pub fn threads(mut self, threads: u32) -> SimulationBuilder<T> {
        self.simulation.num_of_threads = threads;
        self
    }

    pub fn initial_population(mut self, initial_population: Vec<T>) -> SimulationBuilder<T>  {
        let mut new_population = Vec::new();

        for individual in initial_population {
            new_population.push(
                IndividualWrapper {
                    individual: individual,
                    fittness: core::f64::MAX,
                    num_of_mutations: 1
                }
            )
        }

        let num_of_individuals = new_population.len() as u32;
        self.simulation.population = new_population;
        self.simulation.num_of_individuals = num_of_individuals;
        self
    }

    pub fn initial_population_num_mut(mut self, initial_population: Vec<(T, u32)>) -> SimulationBuilder<T>  {
        let mut new_population = Vec::new();

        for (individual, num_of_mutation) in initial_population {
            new_population.push(
                IndividualWrapper {
                    individual: individual,
                    fittness: core::f64::MAX,
                    num_of_mutations: num_of_mutation
                }
            )
        }

        let num_of_individuals = new_population.len() as u32;
        self.simulation.population = new_population;
        self.simulation.num_of_individuals = num_of_individuals;
        self
    }

    pub fn one_individual(mut self, individual: T) -> SimulationBuilder<T> {
        for _ in 0..self.simulation.num_of_individuals {
            self.simulation.population.push(
                IndividualWrapper {
                    individual: individual.clone(),
                    fittness: core::f64::MAX,
                    num_of_mutations: 1
                }
            );
        }
        self
    }

    pub fn one_individual_num_mut(mut self, individual: T, num_of_mutations: u32) -> SimulationBuilder<T> {
        for _ in 0..self.simulation.num_of_individuals {
            self.simulation.population.push(
                IndividualWrapper {
                    individual: individual.clone(),
                    fittness: core::f64::MAX,
                    num_of_mutations: num_of_mutations
                }
            );
        }
        self
    }

    pub fn finalize(self) -> BuilderResult<T> {
        let result = Simulation {
            num_of_iterations: self.simulation.num_of_iterations,
            num_of_individuals: self.simulation.num_of_individuals,
            num_of_threads: self.simulation.num_of_threads,
            improvement_factor: self.simulation.improvement_factor,
            population: self.simulation.population
        };

        if self.simulation.num_of_iterations < 10 { BuilderResult::LowIterration }
        else if self.simulation.num_of_individuals < 3 { BuilderResult::LowIndividuals }
        else { BuilderResult::Ok(result) }
    }
}
