#!/bin/bash

killall sudoku

rm *.log *.dat

DELAY=0.1

target/release/sudoku -l 1.0 -s &

sleep 2

target/release/sudoku -m 1 -i 50000 &
sleep $DELAY
target/release/sudoku -m 2 -i 50000 &
sleep $DELAY
target/release/sudoku -m 4 -i 50000 &
sleep $DELAY
target/release/sudoku -m 8 -i 50000 &
sleep $DELAY
# target/release/sudoku -m 16 -i 50000 &
#sleep $DELAY
# target/release/sudoku -m 32 -i 50000 &
#sleep $DELAY
# target/release/sudoku -m 64 -i 50000 &
#sleep $DELAY
# target/release/sudoku -m 128 -i 50000 &
