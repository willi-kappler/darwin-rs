#!/bin/bash

killall tsp3

rm *.log *.dat

DELAY=0.1

# maybe best: 89190.21485389463
target/release/tsp3 -l 89191.0 -s &

sleep 2

target/release/tsp3 -m 1 -i 40000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 2 -i 30000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 3 -i 20000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 4 -i 10000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 100 -i 1000 -o 100 --method only_best &
sleep $DELAY
target/release/tsp3 -m 100 -i 2000 -o 100 --method only_best &
sleep $DELAY
target/release/tsp3 -m 100 -i 3000 -o 100 --method only_best &
sleep $DELAY
target/release/tsp3 -m 100 -i 4000 -o 100 --method only_best &
sleep $DELAY
target/release/tsp3 -m 10 -i 10000 -o 100 --method keep &
sleep $DELAY
target/release/tsp3 -m 100 -i 1000 -o 100 --method keep &
