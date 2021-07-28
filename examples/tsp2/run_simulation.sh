#!/bin/bash

killall tsp2

rm *.log *.dat

# best may be: 376.3341189874508
target/release/tsp2 -l 400.0 -s &

sleep 2

target/release/tsp2 -m 1 -i 50000 &
target/release/tsp2 -m 2 -i 50000 &
target/release/tsp2 -m 4 -i 50000 &
target/release/tsp2 -m 8 -i 50000 &
#target/release/tsp2 -m 25 &
#target/release/tsp2 -m 30 &
#target/release/tsp2 -m 35 &
#target/release/tsp2 -m 40 &
