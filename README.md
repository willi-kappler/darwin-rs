[![Build Status](https://travis-ci.org/willi-kappler/darwin-rs.svg?branch=master)](https://travis-ci.org/willi-kappler/darwin-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

# darwin-rs
This library allows you to write evolutionary algorithms (EA) using the [Rust](https://www.rust-lang.org/) programming language.

Written by Willi Kappler, License: MIT - Version 0.2 (2016.08.xx)

**Documentation:** [darwin-rs](https://willi-kappler.github.io/darwin-rs)

![tsp start](tsp_start.png)

![tsp end](tsp_end.png)

The example folder contains three examples:

- TSP (traveling salesman problem): the classic type of problem for EA (see two pictures above)
- Sudoku: a sudoku solver using EA
- Queens: solving the queens problem with EA. Although not as fast as [this one](https://github.com/reem/rust-n-queens) ;-)

darwin-rs uses [semantic versioning](http://semver.org/)

# Usage:
Add the following to the Cargo.toml in your project:

```toml
[dependencies]
darwin-rs = "0.2"
```

And this in the rust source code of your application:

```rust
extern crate darwin_rs;

use darwin_rs::{Individual, SimulationBuilder, PopulationBuilder, SimError};
```

Basically you have to implement the trait ```Individual``` for your data structure:

```rust
#[derive(Debug, Clone)]
struct MyStruct {
    text: String
}

impl Individual for MyStruct {
    fn new() -> MyStruct {
        MyStruct{ text: "Some fancy data values...".to_string() }
    }

    fn mutate(&mut self) {
        // Mutate the struct here.
    }

    fn calculate_fitness(&self) -> f64 {
        // Calculate how good the data values are compared to the perfect solution
        0.0
    }
}
```

These three methods are needed:

**new()**: creates new instance of your struct.

**mutate(&mut self)**: mutates the content of the struct.

**calculate_fitness(&self) -> f64**: this calculates the fitness value, that is how close is this individual struct instance to the perfect solution ? Lower values means better fit (or less error).

Tip: Use [lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs) to share large data structure between individuals (see TSP example).

Now you have to create one or more populations that can have different properties:

```rust
let population1 = PopulationBuilder::<MyStruct>::new()
    .set_id(1)
    .individuals(100)
    .increasing_exp_mutation_rate(1.03)
    .reset_limit_increment(100)
    .reset_limit_start(100)
    .reset_limit_end(1000)
    .finalize().unwrap();


let population2 = PopulationBuilder::<MyStruct>::new()
    .set_id(1)
    .individuals(50)
    .increasing_exp_mutation_rate(1.04)
    .reset_limit_increment(200)
    .reset_limit_start(100)
    .reset_limit_end(2000)
    .finalize().unwrap();


```
**set_id()**: Sets the population ID. This can be any positive u32 integer. Currently this is only used for internal statistics, for example: which population does have the most fittest individuals ? This may help you to set the correct parameters for your simulations.

**individuals()**: How many individuals (= distinct copies of your data structure) should the population have ?

**increasing_exp_mutation_rate()**: Sets the mutation rate for each individual: Use exponential mutation rate.

**reset_limit_increment()**: Increase the reset limit by this amount every time the iteration counter reaches the limit

**reset_limit_start()**: The start value of the reset limit.

**reset_limit_end()**: The end value of the reset limit. If this end value is reached the reset limit is reset to the start value above.


After that you have to create a new instance of the simulation and provide the settings:


```rust
let my_builder = SimulationBuilder::<MyStruct>::new()
    .factor(0.34)
    .threads(2)
    .add_population(population1)
    .add_population(population2)
    .finalize();

    match my_builder {
        Err(SimError::EndIterationTooLow) => println!("more than 10 iteratons needed"),
        Ok(mut my_simulation) => {
            my_simulation.run();

            println!("total run time: {} ms", my_simulation.total_time_in_ms);
            println!("improvement factor: {}", my_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}", my_simulation.simulation_result.iteration_counter);

            my_simulation.print_fitness();
        }
    }
```


**factor()**: Sets the termination condition: if the improvement factor is better or equal to this value, the simulation stops.

**threads()**: Number of threads to use for the simulation.

**add_population()**: This adds the previously created population to the simulation.

**finalize()**: Finish setup and do sanity check. Returns ```Ok(Simulation)``` if there are no errors in the configuration.

Then just do a match on the result of ```finalize()``` and call ```simulation.run()``` to start the simulation. After the finishing it, you can access some statistics (```total_time_in_ms```, ```improvement_factor```, ```iteration_counter```) and the populations of course:

```rust
    for population in my_simulation.habitat {
      for wrapper in population.population {...}
    }
```
Each individual is wrapped inside a ```Wrapper``` struct that contains additional information needed for the simulation: **fitness** and the **number of mutations**.
See also the example folder for full working programs.

# Discussion:
- [Reddit](https://www.reddit.com/r/rust/comments/4nnajh/darwinrs_evolutionary_algorithms_with_rust/)
- [Rust User Forum](https://users.rust-lang.org/t/darwin-rs-evolutionary-algorithms-with-rust/6188)

# Used crates:
- [jobsteal](https://github.com/rphmeier/jobsteal): parallelization
- [quick-error](https://github.com/tailhook/quick-error): easy error type creation

# Similar crates:
- [genetic-files](https://github.com/vadixidav/genetic-files)
- [RsGenetic](https://github.com/m-decoster/RsGenetic)
- [evo-rs](https://github.com/mneumann/evo-rs)
- [calco-rs](https://github.com/Kerosene2000/calco-rs)
- [GenGen](https://crates.io/crates/GenGen)
- [parasailors](https://github.com/dikaiosune/parasailors)
- [random-wheel-rs](https://github.com/Kerosene2000/random-wheel-rs)
- [roulette-wheel-rs](https://github.com/Kerosene2000/roulette-wheel-rs)

TODO:
- [ ] Add more documentation comments for library
- [ ] Add test cases
- [ ] Add more examples (ocr, ...)
- [x] Split up code
- [x] Allow user to specify start and end of reset limit
- [ ] Use log file instead of println!()
- [x] Use multiple population
- [ ] Maybe use phantom type for builder pattern to detect configuration error at compile type ? (https://www.reddit.com/r/rust/comments/2pgrz7/required_parameters_utilizing_the_builder_pattern/)
- [ ] Add super optimization (only allow changes that have an improvement) ?
- [ ] Add possibility to load and save population / individuals in order to cancel / resume simulation (serde)

Any feedback is welcome!
