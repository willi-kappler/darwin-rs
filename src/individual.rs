
pub trait Individual {
    fn mutate(&mut self);
    fn calculate_fitness(&self) -> f64;
    fn reset(&mut self);
}

pub struct IndividualWrapper<T> {
    pub individual: T,
    pub fitness: f64,
}

impl IndividualWrapper {
    pub fn new<T: Individual>(individual: T) -> Self {
        Self {
            individual,
            fitness: f64::MAX,
        }
    }
    pub fn mutate(&mut self) {
        self.individual.mutate();
    }
    pub fn calculate_fitness(&self) -> f64 {
        self.individual.calculate_fitness()
    }
    pub fn reset(&mut self) {
        self.individual.reset();
    }
}
