#!/bin/bash

killall tsp3

rm *.log *.dat

DELAY=0.1

# best may be: 89971.50164001813
target/release/tsp3 -l 89980.0 -s &

sleep 2

target/release/tsp3 -m 10 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 20 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 30 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 40 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 50 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 60 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 70 -i 1000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 80 -i 1000 -o 100 &
