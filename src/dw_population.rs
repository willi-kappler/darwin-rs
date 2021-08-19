

use crate::{DWConfiguration, DWIndividual, dw_individual::DWIndividualWrapper};

use rand::{Rng, rngs::StdRng, SeedableRng};
use log::{debug};


pub(crate) struct DWPopulation<T> {
    collection: Vec<DWIndividualWrapper<T>>,
    max_population_size: usize,
    num_of_mutations: u64,
    fitness_limit: f64,
    new_best_fitness: f64,
    delete_method: u8,
    rng: StdRng,
}

impl<T: DWIndividual + Clone> DWPopulation<T> {
    pub(crate) fn new(initial: DWIndividualWrapper<T>, dw_configuration: &DWConfiguration) -> Self {
        let max_population_size = dw_configuration.num_of_individuals;
        let fitness_limit = dw_configuration.fitness_limit;
        let num_of_mutations = dw_configuration.num_of_mutations;

        // TODO: Use a sorted data structure
        // Maybe BTreeSet: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

        let mut collection = Vec::new();

        for _ in 0..max_population_size {
            let mut new_individual = initial.clone();
            new_individual.mutate(&initial);
            new_individual.calculate_fitness();
            collection.push(new_individual);
        }

        Self {
            collection,
            max_population_size,
            num_of_mutations,
            fitness_limit,
            new_best_fitness: f64::MAX,
            delete_method: 0,
            rng: SeedableRng::from_entropy(),
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

    fn index_of_best(&self) -> usize {
        let mut best_fitness = self.collection[0].get_fitness();
        let mut best_index = 0;

        for index in 1..self.collection.len() {
            let fitness = self.collection[index].get_fitness();
            if fitness < best_fitness {
                best_fitness = fitness;
                best_index = index;
            }
        }

        best_index
    }

    fn index_of_2_best(&self) -> (usize, usize) {
        let mut fitness1 = f64::MAX;
        let mut fitness2 = f64::MAX;
        let mut index1 = 0;
        let mut index2 = 0;

        for index in 0..self.collection.len() {
            let fitness = self.collection[index].get_fitness();

            if fitness < fitness2 {
                if fitness <= fitness1 {
                    fitness1 = fitness;
                    index1 = index;
                } else {
                    fitness2 = fitness;
                    index2 = index;
                }
            }
        }

        (index1, index2)
    }

    pub(crate) fn is_job_done(&self) -> bool {
        self.get_best_fitness() < self.fitness_limit
    }

    pub(crate) fn get_best_fitness(&self) -> f64 {
        let mut best_fitness = self.collection[0].get_fitness();

        for index in 1..self.collection.len() {
            let fitness = self.collection[index].get_fitness();
            if fitness < best_fitness {
                best_fitness = fitness;
            }
        }

        best_fitness
    }

    pub(crate) fn log_fitness(&mut self) -> () {
        self.collection.sort_unstable_by(|individual1, individual2| {
            let fitness1 = individual1.get_fitness();
            let fitness2 = individual2.get_fitness();

            fitness1.partial_cmp(&fitness2).unwrap()
        });

        for individual in self.collection.iter() {
            debug!("Fitness: '{}'", individual.get_fitness());
        }

    }

    pub(crate) fn get_new_best_fitness(&self) -> f64 {
        self.new_best_fitness
    }

    pub(crate) fn get_best_and_worst_fitness(&self) -> (f64, f64) {
        let mut best_fitness = self.collection[0].get_fitness();
        let mut worst_fitness = best_fitness;

        for index in 1..self.collection.len() {
            let fitness = self.collection[index].get_fitness();
            if fitness < best_fitness {
                best_fitness = fitness;
            } else if fitness > worst_fitness {
                worst_fitness = fitness;
            }
        }

        (best_fitness, worst_fitness)
    }

    fn get_median_fitness(&self) -> f64 {
        let (best_fitness, worst_fitness) = self.get_best_and_worst_fitness();
        (best_fitness + worst_fitness) / 2.0
    }

    pub(crate) fn calc_new_best_individual(&mut self) {
        self.new_best_fitness = self.get_best_fitness();
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
        let best_index = self.index_of_best();
        &self.collection[best_index]
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

    pub(crate) fn random_delete(&mut self) {
        let mut average_fitness = self.get_median_fitness();

        while self.collection.len() > self.max_population_size {
            let index = self.random_index();
            let fitness = self.collection[index].get_fitness();

            if fitness >= average_fitness {
                self.collection.swap_remove(index);
                average_fitness = self.get_median_fitness();
            }
        }
    }

    pub(crate) fn keep_best_delete_others(&mut self) {
        // Save best individual to first two positions
        let (index1, index2) = self.index_of_2_best();
        self.collection.swap(0, index1);
        self.collection.swap(1, index2);

        while self.collection.len() > self.max_population_size {
            let index = self.random_index_from(2);
            self.collection.swap_remove(index);
        }
    }

    pub(crate) fn delete(&mut self) {
        match self.delete_method {
            0 => {
                self.random_delete();
            }
            1 => {
                self.keep_best_delete_others()
            }
            _ => {
                panic!("Unknown delete method: {}", self.delete_method);
            }
        }
    }

    pub(crate) fn select_delete_method(&mut self) {
        self.delete_method = self.rng.gen_range(0..2);
    }

    pub(crate) fn get_delete_method(&self) -> u8 {
        self.delete_method
    }

    pub(crate) fn get_random_individual(&mut self) -> &DWIndividualWrapper<T> {
        let index = self.random_index();
        &self.collection[index]
    }

    pub(crate) fn reseed_rng(&mut self) {
        self.rng = SeedableRng::from_entropy();
    }
}
