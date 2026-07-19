#!/bin/sh
# Trunk may schedule hooks from the same stage concurrently. Static rendering,
# article generation, and minification all touch the staging tree, so serialize
# them explicitly to prevent a fast hook from minifying or scanning half-written
# output on cold CI machines.
set -eu

cargo run --quiet --manifest-path tools/site/Cargo.toml
cargo run --quiet --manifest-path tools/blog/Cargo.toml
sh tools/minify.sh
