#!/bin/bash

killall tsp3

rm *.log *.dat

DELAY=0.1

# best may be: 91379.51073418828
target/release/tsp3 -l 400.0 -s &

sleep 2

target/release/tsp3 -m 1 -i 500000 &
sleep $DELAY
target/release/tsp3 -m 2 -i 500000 &
sleep $DELAY
target/release/tsp3 -m 4 -i 500000 &
sleep $DELAY
target/release/tsp3 -m 8 -i 500000 &
sleep $DELAY
#target/release/tsp3 -m 16 -i 500000 &
#sleep $DELAY
#target/release/tsp3 -m 32 -i 500000 &
#sleep $DELAY
#target/release/tsp3 -m 64 -i 500000 &
#sleep $DELAY
#target/release/tsp3 -m 128 -i 500000 &
