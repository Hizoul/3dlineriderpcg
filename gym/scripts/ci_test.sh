#!/bin/bash
err=0
trap 'err=1' ERR
cd compressed-vec
cargo test
cd ..
cd rusty-gym
cargo test
test $err = 0