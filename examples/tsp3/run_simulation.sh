#!/bin/bash

killall tsp3

rm *.log *.dat

DELAY=0.1

# best may be: 90878.5569909252
target/release/tsp3 -l 90880.0 -s &

sleep 2

target/release/tsp3 -m 10 -i 10000 -o 10 &
sleep $DELAY
target/release/tsp3 -m 10 -i 10000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 10 -i 100000 -o 10 &
sleep $DELAY
target/release/tsp3 -m 10 -i 100000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 100 -i 10000 -o 10 &
sleep $DELAY
target/release/tsp3 -m 100 -i 10000 -o 100 &
sleep $DELAY
target/release/tsp3 -m 100 -i 100000 -o 10 &
sleep $DELAY
target/release/tsp3 -m 100 -i 100000 -o 100 &
