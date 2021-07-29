#!/bin/bash

killall queens

rm *.log *.dat

DELAY=0.1

target/release/queens -l 1.0 -s &

sleep 2

target/release/queens -m 1 -i 50000 &
sleep $DELAY
target/release/queens -m 2 -i 50000 &
sleep $DELAY
target/release/queens -m 4 -i 50000 &
sleep $DELAY
target/release/queens -m 8 -i 50000 &
sleep $DELAY
# target/release/queens -m 16 -i 50000 &
#sleep $DELAY
# target/release/queens -m 32 -i 50000 &
#sleep $DELAY
# target/release/queens -m 64 -i 50000 &
#sleep $DELAY
# target/release/queens -m 128 -i 50000 &
