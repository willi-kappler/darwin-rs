#!/bin/bash

killall queens

rm *.log *.dat

target/release/queens -l 1.0 -s &

sleep 2

target/release/queens -m 5 &
target/release/queens -m 10 &
target/release/queens -m 15 &
target/release/queens -m 20 &
target/release/queens -m 25 &
target/release/queens -m 30 &
target/release/queens -m 35 &
target/release/queens -m 40 &
