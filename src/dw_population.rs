

use crate::{DWConfiguration, DWIndividual, dw_individual::DWIndividualWrapper};

use rand::{Rng, rngs::StdRng, SeedableRng};
use log::{debug};

use std::cmp::Ordering;

fn compare_individual<T: DWIndividual>(i1: &DWIndividualWrapper<T>, i2: &DWIndividualWrapper<T>) -> Ordering {
    let f1 = i1.get_fitness();
    let f2 = i2.get_fitness();

    f1.partial_cmp(&f2).unwrap()
}

pub(crate) struct DWPopulation<T> {
    collection: Vec<DWIndividualWrapper<T>>,
    max_population_size: usize,
    num_of_mutations: u64,
    fitness_limit: f64,
    new_best_fitness: f64,
    reset_counter: u64,
    reset_fitness: f64,
    max_reset: u64,
    rng: StdRng,
}

impl<T: DWIndividual + Clone> DWPopulation<T> {
    pub(crate) fn new(initial: DWIndividualWrapper<T>, dw_configuration: &DWConfiguration) -> Self {
        let max_population_size = dw_configuration.max_population_size;
        let fitness_limit = dw_configuration.fitness_limit;
        let num_of_mutations = dw_configuration.num_of_mutations;

        // TODO: Maybe use a sorted data structure
        // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

        let mut collection = Vec::new();

        for _ in 0..max_population_size {
            let mut new_individual = initial.clone();
            new_individual.mutate(&initial);
            new_individual.calculate_fitness();
            collection.push(new_individual);
        }

        collection.sort_unstable_by(compare_individual);

        Self {
            collection,
            max_population_size,
            num_of_mutations,
            fitness_limit,
            new_best_fitness: f64::MAX,
            reset_counter: 0,
            reset_fitness: 0.0,
            max_reset: 100,
            rng: SeedableRng::from_entropy(),
        }
    }

    pub(crate) fn check_reset(&mut self, individual: DWIndividualWrapper<T>) {
        let fitness = self.get_best_fitness();
        if self.reset_fitness == fitness {
            self.reset_counter += 1;
            debug!("Reset counter increased: '{}'", self.reset_counter);
            if self.reset_counter >= self.max_reset {
                debug!("Max reset reached, population will be randomly reset");
                self.reset_counter = 0;
                for individual in self.collection.iter_mut() {
                    individual.random_reset();
                    individual.calculate_fitness();
                }
            } else {
                self.add_individual(individual);
            }
        } else {
            self.reset_fitness = fitness;
            self.reset_counter = 0;
            if individual.get_fitness() > fitness {
                self.add_individual(individual);
            }
        }
    }

    pub(crate) fn from_vec(&mut self, population: &mut Vec<DWIndividualWrapper<T>>) {
        self.collection.clear();
        self.collection.append(population);
    }

    pub(crate) fn to_vec(&self) -> &Vec<DWIndividualWrapper<T>> {
        &self.collection
    }

    fn random_index_from(&mut self, start: usize) -> usize{
        self.rng.gen_range(start..self.collection.len())
    }

    fn random_index(&mut self) -> usize {
        self.random_index_from(0)
    }

    fn random_index_new(&mut self, index1: usize) -> usize {
        let mut index2 = self.random_index();

        while index1 == index2 {
            index2 = self.random_index();
        }

        index2
    }

    pub(crate) fn is_job_done(&self) -> bool {
        self.get_best_fitness() < self.fitness_limit
    }

    pub(crate) fn get_best_fitness(&self) -> f64 {
        self.collection[0].get_fitness()
    }

    pub(crate) fn log_fitness(&mut self) -> () {
        for individual in self.collection.iter() {
            debug!("Fitness: '{}'", individual.get_fitness());
        }
    }

    pub(crate) fn get_new_best_fitness(&self) -> f64 {
        self.new_best_fitness
    }

    pub(crate) fn get_best_and_worst_fitness(&self) -> (f64, f64) {
        let best_fitness = self.collection[0].get_fitness();
        let worst_fitness = self.collection[self.collection.len() - 1].get_fitness();

        (best_fitness, worst_fitness)
    }

    pub(crate) fn has_new_best_individual(&mut self) -> bool {
        let best_fitness = self.get_best_fitness();

        if best_fitness < self.new_best_fitness {
            self.new_best_fitness = best_fitness;
            true
        } else {
            false
        }
    }

    pub(crate) fn get_best_individual(&self) -> &DWIndividualWrapper<T> {
        &self.collection[0]
    }

    pub(crate) fn add_individual(&mut self, new_individual: DWIndividualWrapper<T>) {
        self.collection.push(new_individual);
    }

    pub(crate) fn mutate_random_single_clone(&mut self) {
        for _ in 0..self.num_of_mutations {
            let index1 = self.random_index();
            let index2 = self.random_index_new(index1);

            let individual = &self.collection[index2];
            let mut new_individual = self.collection[index1].clone();
            new_individual.mutate(individual);
            new_individual.calculate_fitness();
            self.collection.push(new_individual);
        }
    }

    pub(crate) fn mutate_all_clone(&mut self) {
        for index1 in 0..self.collection.len() {
            let mut new_individual = self.collection[index1].clone();

            for _ in 0..self.num_of_mutations {
                let index2 = self.random_index_new(index1);
                let individual = &self.collection[index2];
                new_individual.mutate(individual);
            }

            new_individual.calculate_fitness();
            self.collection.push(new_individual);
        }
    }

    pub(crate) fn mutate_all_only_best(&mut self) {
        for index1 in 0..self.collection.len() {
            let mut new_individual = self.collection[index1].clone();
            let old_fitness = new_individual.get_fitness();

            for _ in 0..self.num_of_mutations {
                let index2 = self.random_index_new(index1);
                let individual = &self.collection[index2];
                new_individual.mutate(individual);
                new_individual.calculate_fitness();

                if new_individual.get_fitness() < old_fitness {
                    self.collection.push(new_individual.clone());
                }
            }
        }
    }

    pub(crate) fn sort(&mut self) {
        self.collection.sort_unstable_by(compare_individual);
    }

    pub(crate) fn clean(&mut self) {
        self.sort();
        self.collection.truncate(self.max_population_size);
    }

    pub(crate) fn get_random_individual(&mut self) -> &DWIndividualWrapper<T> {
        let index = self.random_index();
        &self.collection[index]
    }

    pub(crate) fn reseed_rng(&mut self) {
        self.rng = SeedableRng::from_entropy();
    }
}
