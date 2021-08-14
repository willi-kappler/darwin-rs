#!/bin/bash

killall tsp3
rm *.log

git checkout --force node_crunch
git pull
cargo build --release

