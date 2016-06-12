[![Build Status](https://travis-ci.org/willi-kappler/darwin-rs.svg?branch=master)](https://travis-ci.org/willi-kappler/darwin-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

# darwin-rs
This library allows you to write evolutionary algorithms (EA) using the [Rust](https://www.rust-lang.org/) programming language.

Written by Willi Kappler, License: MIT - Version 0.1.1 (2016.06.12)

Documentation: [darwin-rs](https://willi-kappler.github.io/darwin-rs)

![tsp start](tsp_start.png)

![tsp end](tsp_end.png)

The example folder contains three examples:

- TSP (traveling salesman problem): the classic type of problem for EA (see two pictures above)
- Sudoku: a sudoku solver using EA
- Queens: solving the queens problem with EA. Although not as fast as [this one](https://github.com/reem/rust-n-queens) ;-)

# Usage:
Add the following to the Cargo.toml in your project:

```toml
[dependencies]
darwin-rs = "0.1"
```

And this in the rust source code of your application:

```rust
extern crate darwin_rs;

use darwin_rs::{Individual, SimulationBuilder, Error};
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


After that you have to create a new instance of the simulation and provide the settings:


```rust
let my_builder = SimulationBuilder::<MyStruct>::new()
    .factor(0.34)
    .threads(2)
    .individuals(100)
    .increasing_exp_mutation_rate(1.03)
    .finalize();

    match my_builder {
        Err(Error::TooLowEndIteration) => println!("more than 10 iteratons needed"),
        Err(Error::TooLowIndividuals) => println!("more than 2 individuals needed"),
        Ok(mut my_simulation) => {
            my_simulation.run();

            println!("total run time: {} ms", my_simulation.total_time_in_ms);
            println!("improvement factor: {}", my_simulation.improvement_factor);
            println!("number of iterations: {}", my_simulation.iteration_counter);

            my_simulation.print_fitness();
        }
    }
```

**factor()**: Sets the termination condition: if the improvement factor is better or equal to this value, the simulation stops.

**threads()**: Number of threads to use for the simulation.

**individuals()**: How many individuals (= distinct copies of your data structure) should the simulation have ?

**increasing_exp_mutation_rate()**: Sets the mutation rate for each individual: Use exponential mutation rate.

**finalize()**: Finish setup and do sanity check. Returns ```Ok(Simulation)``` if there are no errors in the configuration.

Then just do a match on the result of ```finalize()``` and call ```simulation.run()``` to start the simulation. After the finishing it, you can access some statistics (```total_time_in_ms```, ```improvement_factor```, ```iteration_counter```) and the population of course:

```rust
    for wrapper in my_simulation.population {...}
```
Each individual is wrapped inside a ```Wrapper``` struct that contains additional information needed for the simulation: **fitness** and the **number of mutations**.
See also the example folder for full working programs.


# Used crates:
- [time](https://doc.rust-lang.org/time/time/index.html): time usage statistics
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
- [ ] Maybe use phantom type for builder pattern to detect configuration error at compile type ? (https://www.reddit.com/r/rust/comments/2pgrz7/required_parameters_utilizing_the_builder_pattern/)
- [ ] Add super optimization (only allow changes that have an improvement) ?
- [ ] Add possibility to load and save population / individuals in order to cancel / resume simulation (serde)

Any feedback is welcome!
