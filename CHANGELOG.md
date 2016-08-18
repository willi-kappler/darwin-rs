# Change Log

## 0.3 - 2016-xx
- Write output into a log file instead of using print!()
- Remove new() from trait Individual, provide reset() instead.
- All mutexes removed
- User must now provide whole population but can now use shared configuration / data instead of using lazy_static
- calculate_fitness now needs (&mut self)
- add option share_fittest to share the fittest individual between all population after each iteration
- add ocr2 example

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
