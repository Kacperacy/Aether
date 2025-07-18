#!/bin/bash

# Build with optimizations
cargo build --release

# Run flamegraph profiling
cargo flamegraph --bench move_generation -- --bench

# Run memory profiling with valgrind
valgrind --tool=massif --stacks=yes target/release/aether

# Run CPU profiling
perf record -g target/release/aether
perf report
