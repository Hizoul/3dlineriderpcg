# Scalable Procedural Content Generation via Transfer Reinforcement Learning

This repository contains the code to run the experiments conducted in our paper as well as re-generate the graphs based on the Replay traces.

To run the experiments, make a python weel of the rust environment:
```
maturin build --manylinux=off --release
```
and then install the resulting .whl file.


The experiments can be ran using `experiments/pysrc/main.py` and the graphs can be generated with `experiments/pysrc/plot.py`.

The environment implementation is found in `envs/linerider`. THe rest is mostly glue for Rust <=> Python interaction.

The replay traces to re-generate the graphs or verify can be found in the Github Releases of this repository.

To regenerate the graphs extract the files to a folder called `0exp` at the root of the experiment.