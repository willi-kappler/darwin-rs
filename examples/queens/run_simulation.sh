#!/bin/bash

killall queens

rm *.log *.dat

target/release/queens -l 1.0 -s &

sleep 2

target/release/queens -m 1 &
target/release/queens -m 2 &
target/release/queens -m 4 &
target/release/queens -m 8 &
target/release/queens -m 16 &
target/release/queens -m 32 &
target/release/queens -m 64 &
target/release/queens -m 128 &
