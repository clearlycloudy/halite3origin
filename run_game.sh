#!/usr/bin/env bash

set -e

cargo build
./halite --replay-directory replays/ -vvv "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/my_bot"

# ./halite -s 200 -n 4 --replay-directory replays/ -vvv "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/my_bot"
# --width 32 --height 32
