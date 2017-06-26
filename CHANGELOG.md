# Change Log

## 0.4 - 2017-06-26
- Allow user to specify num_of_global_fittest, fixes https://github.com/willi-kappler/darwin-rs/issues/12 .
- Use error_chain.
- Add output_every, to only output every nth time a new fittest individual is found.
- Add share_every, to only share fittest individual after a number of iterations.

## 0.3 - 2016-08-29
- Write output into a log file instead of using print!().
- Remove new() from trait Individual, provide reset() instead.
- All mutexes removed.
- User must now provide whole population but can now use shared configuration / data instead of using lazy_static.
- calculate_fitness now needs (&mut self).
- Add option share_fittest to share the fittest individual between all population after each iteration.
- Add ocr2 example.
- Add optional method new_fittest_found() to write out some statistics. Default implementation does nothing.
- Add fitness counter statistic to population.
- Fix bug in parallelization using jobsteal.
- Fix bug in TSP2 example.

## 0.2 - 2016-08-17
- Split up code into several files
- Allow user to specify start and end value for reset limit
- Allow multiple populations
- Split threads for each population instead for each individual

## 0.1.1 - 2016-06-12

- Allow user to specify the reset limit
- Code clean up and fix typos

## 0.1 - 2016-06-11

- First public release on crates.io
