#!/bin/bash

killall queens

rm *.log *.dat

target/release/queens -l 1.0 -s &

sleep 2

target/release/queens -m 1 -i 50000 &
