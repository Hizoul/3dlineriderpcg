rm -rf trl-experiments
rm -rf ../linerider/trl-experiments
cargo make run
cargo make extract
cp -rf trl-experiments ../linerider

# find . -type f ! -name "highlights*" -delete