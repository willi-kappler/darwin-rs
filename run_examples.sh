#!/bin/bash

cargo clean
cargo update
cargo test

cd examples

cd ocr
cargo clean
cargo update
cargo run --release
cd ..

cd queens
cargo clean
cargo update
cargo run --release
cd ..

cd sudoku
cargo clean
cargo update
cargo run --release
cd ..

cd tsp
cargo clean
cargo update
cargo run --release
cd ..

cd tsp2
cargo clean
cargo update
cargo run --release
cd ..

