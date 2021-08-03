#!/bin/bash

killall ocr1

rm *.log *.dat *.json

DELAY=0.1

target/release/ocr1 -l 1.0 -s &

sleep 2

target/release/ocr1 -m 1 -i 50000 &
sleep $DELAY
target/release/ocr1 -m 2 -i 50000 &
sleep $DELAY
target/release/ocr1 -m 4 -i 50000 &
sleep $DELAY
target/release/ocr1 -m 8 -i 50000 &
sleep $DELAY
# target/release/ocr1 -m 16 -i 50000 &
#sleep $DELAY
# target/release/ocr1 -m 32 -i 50000 &
#sleep $DELAY
# target/release/ocr1 -m 64 -i 50000 &
#sleep $DELAY
# target/release/ocr1 -m 128 -i 50000 &
