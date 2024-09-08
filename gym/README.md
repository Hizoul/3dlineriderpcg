# Rustified Gym
Yet another Rust Gym API using PYO3

Packages
- `rusty-gym` A WIP rust port of the OpenAI Gym API.
- `compressed-vec` Written to allow space friendly episode replay collection (see https://arxiv.org/abs/2203.01075)
- `xp-tools` Small utility crate to ease developement between WASM and Desktops

# Code Limitations that might be blockers for some users
- rusty-gym spaces don't have bounds yet
- code that relates to actions sometimes assumes that the value range is x >= 0.0 so never negative
- some code might assume that action spaces can only be Discrete