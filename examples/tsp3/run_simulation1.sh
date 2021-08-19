#!/bin/bash

killall tsp3

rm *.log *.dat

# Delay between clients to prevent overwriting log files
DELAY=0.1

# IP address of server
IP=127.0.0.1

# Population size
POPULATION=100

# maybe best: 89190.21485389463
target/release/tsp3 -l 1.0 -o $POPULATION -s &

# Give the server a chance to start up
sleep 2

target/release/tsp3 --ip $IP -m 1 -i 8000 -o $POPULATION --method simple &
sleep $DELAY
target/release/tsp3 --ip $IP -m 2 -i 4000 -o $POPULATION --method simple &
sleep $DELAY
target/release/tsp3 --ip $IP -m 4 -i 2000 -o $POPULATION --method simple &
sleep $DELAY
target/release/tsp3 --ip $IP -m 8 -i 1000 -o $POPULATION --method simple &
sleep $DELAY
target/release/tsp3 --ip $IP -m 10 -i 800 -o $POPULATION --method only_best &
sleep $DELAY
target/release/tsp3 --ip $IP -m 20 -i 400 -o $POPULATION --method only_best &
sleep $DELAY
target/release/tsp3 --ip $IP -m 40 -i 200 -o $POPULATION --method only_best &
sleep $DELAY
target/release/tsp3 --ip $IP -m 80 -i 100 -o $POPULATION --method only_best &
sleep $DELAY
