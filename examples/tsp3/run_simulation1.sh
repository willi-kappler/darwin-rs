#!/bin/bash

killall tsp3

rm *.log *.dat

# Delay between clients to prevent overwriting log files
DELAY=0.1

# IP address of server
IP=127.0.0.1

# Population size
POPULATION=100

# Reset counter
RESET=20

# Delete method
DELETE=sort_keep

# maybe best: 89190.21485389463
target/release/tsp3 -l 1.0 -o $POPULATION -s &

# Give the server a chance to start up
sleep 2

target/release/tsp3 --ip $IP -m 1 -r $RESET -i 8000 -o $POPULATION --method simple --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 2 -r $RESET -i 4000 -o $POPULATION --method simple --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 4 -r $RESET -i 2000 -o $POPULATION --method simple --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 8 -r $RESET -i 1000 -o $POPULATION --method simple --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 10 -r $RESET -i 800 -o $POPULATION --method only_best --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 20 -r $RESET -i 400 -o $POPULATION --method only_best --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 40 -r $RESET -i 200 -o $POPULATION --method only_best --delete $DELETE &
sleep $DELAY
target/release/tsp3 --ip $IP -m 80 -r $RESET -i 100 -o $POPULATION --method only_best --delete $DELETE &
