#!/bin/bash

killall tsp3

rm *.log *.dat

DELAY=0.1

# best may be: 376.3341189874508
target/release/tsp3 -l 400.0 -s &

sleep 2

target/release/tsp3 -m 1 -i 50000 &
sleep $DELAY
target/release/tsp3 -m 2 -i 50000 &
sleep $DELAY
target/release/tsp3 -m 4 -i 50000 &
sleep $DELAY
target/release/tsp3 -m 8 -i 50000 &
sleep $DELAY
#target/release/tsp3 -m 25 &
#sleep $DELAY
#target/release/tsp3 -m 30 &
#sleep $DELAY
#target/release/tsp3 -m 35 &
#sleep $DELAY
#target/release/tsp3 -m 40 &

