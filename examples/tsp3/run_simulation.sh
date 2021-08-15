#!/bin/bash

killall tsp3

rm *.log *.dat

DELAY=0.1

# maybe best: 89190.21485389463
target/release/tsp3 -l 1.0 -s &

sleep 2

target/release/tsp3 -m 100 -i 100 -o 100 --method only_best &
sleep $DELAY
target/release/tsp3 -m 1 -i 1000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 1 -i 10000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 10 -i 1000 -o 100 --method simple &
sleep $DELAY
target/release/tsp3 -m 1 -i 1000 -o 100 --method reset &
sleep $DELAY
target/release/tsp3 -m 1 -i 10000 -o 100 --method reset &
sleep $DELAY
target/release/tsp3 -m 10 -i 1000 -o 100 --method reset &
