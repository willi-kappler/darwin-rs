#!/bin/bash

killall tsp1

rm *.log *.dat

DELAY=0.1

# best may be: 376.3341189874508
target/release/tsp1 -l 400.0 -s &

sleep 2

target/release/tsp1 -m 1 -i 50000 &
sleep $DELAY
target/release/tsp1 -m 2 -i 50000 &
sleep $DELAY
target/release/tsp1 -m 4 -i 50000 &
sleep $DELAY
target/release/tsp1 -m 8 -i 50000 &
sleep $DELAY
#target/release/tsp1 -m 16 -i 50000 &
#sleep $DELAY
##target/release/tsp1 -m 32 -i 50000 &
#sleep $DELAY
#target/release/tsp1 -m 64 -i 50000 &
#sleep $DELAY
#target/release/tsp1 -m 128 -i 50000 &
