#!/bin/bash

killall tsp3

rm *.log *.dat

# Delay between clients to prevent overwriting log files
DELAY=0.1

# IP address of server
IP=127.0.0.1

# Population size
POPULATION=100

# Mutation method
METHOD=only_best

target/release/tsp3 --ip $IP -m 2 -r 20 -i 1000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 4 -r 20 -i 1000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 8 -r 20 -i 1000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 16 -r 20 -i 1000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 2 -r 20 -i 10000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 4 -r 20 -i 10000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 8 -r 20 -i 10000 -o $POPULATION --method $METHOD &
sleep $DELAY
target/release/tsp3 --ip $IP -m 16 -r 20 -i 10000 -o $POPULATION --method $METHOD &
