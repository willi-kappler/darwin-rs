#!/bin/bash

cd examples

cd ocr
cargo run --release
cd ..

cd queens
cargo run --release
cd ..

cd sudoku
cargo run --release
cd ..

cd tsp
cargo run --release
cd ..

cd tsp2
cargo run --release
cd ..

